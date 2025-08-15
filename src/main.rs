use axum::{extract::{Path, Query, State}, http::StatusCode, response::{Html, IntoResponse, Response}, routing::get, Json, Router};
use tower_http::services::ServeDir;
use dotenvy::dotenv;
use serde::Serialize;
use sqlx::FromRow;
use sqlx::PgPool;
use std::{env, net::SocketAddr, sync::Arc};
use serde_json::json;
use tokio::sync::RwLock;
use tokio::fs;
use tera::{Tera, Context};

struct AppState {
    db: PgPool,
    auto_update_secret: Option<String>,
    // cache for concatenated chart modules JS (None until populated)
    chart_modules_cache: RwLock<Option<String>>,
    // Tera template engine
    tera: Tera,
}

#[derive(FromRow, Serialize, Debug)]
struct Report {
    id: i32,
    html_content: String,
    css_content: Option<String>,
    js_content: Option<String>,
    html_content_en: Option<String>,
    js_content_en: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
    let auto_update_secret = env::var("AUTO_UPDATE_SECRET_KEY").ok();

    let pool = PgPool::connect(&database_url).await?;

    // Initialize Tera template engine - only load specific templates we need
    let mut tera = Tera::default();
    tera.add_template_file("templates/index.html", Some("index.html")).expect("Failed to load index.html");
    tera.add_template_file("templates/components/theme_toggle.html", Some("components/theme_toggle.html")).expect("Failed to load theme_toggle.html");
    tera.add_template_file("templates/components/language_toggle.html", Some("components/language_toggle.html")).expect("Failed to load language_toggle.html");
    tera.add_template_file("templates/report_list.html", Some("report_list.html")).expect("Failed to load report_list.html");
    tera.autoescape_on(vec![]); // Disable auto-escaping for safe content

    let state = AppState { 
        db: pool, 
        auto_update_secret, 
        chart_modules_cache: RwLock::new(None),
        tera,
    };
    let shared_state = Arc::new(state);

    let app = Router::new()
        .nest_service("/static", ServeDir::new("static"))
        .route("/health", get(health))
        // serve a simple homepage at / that links to the report index
        .route("/", get(homepage))
        // keep the existing index behaviour available at /index.html
        .route("/index.html", get(index))
        .route("/report/:id", get(view_report))
        .route("/pdf-template/:id", get(pdf_template))
        .route("/reports", get(report_list))
        .route("/upload", get(upload_page))
        .route("/auto-update-system-:secret", get(auto_update))
    .with_state(shared_state);

    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port: u16 = env::var("PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(8000);
    let addr: SocketAddr = format!("{}:{}", host, port).parse().unwrap();

    println!("Starting server on http://{}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

async fn health() -> impl IntoResponse {
    Json(serde_json::json!({"status": "healthy", "message": "Crypto Dashboard Rust server is running"}))
}

async fn index(State(state): State<Arc<AppState>>) -> Response {
    let rec = sqlx::query_as::<_, Report>(
            "SELECT id, html_content, css_content, js_content, html_content_en, js_content_en, created_at FROM report ORDER BY created_at DESC LIMIT 1",
    )
    .fetch_optional(&state.db)
    .await;

    // Get chart modules content
    let chart_modules_content = get_chart_modules_content(&state).await;

    // Create Tera context
    let mut context = Context::new();
    
    match rec {
        Ok(Some(report)) => {
            context.insert("report", &report);
        }
        Ok(None) => {
            // Create empty report for template
            let empty_report = serde_json::json!({
                "html_content": "",
                "html_content_en": "",
                "css_content": "",
                "js_content": ""
            });
            context.insert("report", &empty_report);
        }
        Err(e) => {
            eprintln!("DB error: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
        }
    }

    // Add chart modules function result
    context.insert("chart_modules_content", &chart_modules_content);

    // Render template with Tera
    match state.tera.render("index.html", &context) {
        Ok(html) => Html(html).into_response(),
        Err(e) => {
            eprintln!("Template render error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Template render error").into_response()
        }
    }
}

async fn homepage() -> Response {
    match fs::read_to_string("static/home.html").await {
        Ok(s) => Html(s).into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "Home page not found").into_response(),
    }
}

async fn view_report(Path(id): Path<i32>, State(state): State<Arc<AppState>>) -> Response {
    let rec = sqlx::query_as::<_, Report>(
        "SELECT id, html_content, css_content, js_content, html_content_en, js_content_en, created_at FROM report WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await;

    match rec {
        Ok(Some(report)) => {
            // Build full page using the same index template so the report is shown with site chrome
            let chart_modules_content = get_chart_modules_content(&state).await;
            let mut context = Context::new();
            context.insert("report", &report);
            context.insert("chart_modules_content", &chart_modules_content);

            match state.tera.render("index.html", &context) {
                Ok(html) => Html(html).into_response(),
                Err(e) => {
                    eprintln!("Template render error: {}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, "Template render error").into_response()
                }
            }
        }
        Ok(None) => (StatusCode::NOT_FOUND, "Report not found").into_response(),
        Err(e) => {
            eprintln!("DB error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response()
        }
    }
}

async fn pdf_template(Path(id): Path<i32>, State(state): State<Arc<AppState>>) -> Response {
    // For simplicity return same html_content. You can wrap with PDF template if needed.
    view_report(Path(id), State(state)).await
}

#[derive(FromRow, Serialize)]
struct ReportSummary {
    id: i32,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize)]
struct ReportListItem {
    id: i32,
    created_date: String,
    created_time: String,
}

async fn report_list(Query(params): Query<std::collections::HashMap<String, String>>, State(state): State<Arc<AppState>>) -> Response {
    // Pagination params
    let page: i64 = params.get("page").and_then(|p| p.parse().ok()).unwrap_or(1);
    let per_page: i64 = 10;
    let offset = (page - 1) * per_page;

    // Get total count
    let total_res = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM report").fetch_one(&state.db).await;
    let total = match total_res {
        Ok(t) => t,
        Err(e) => {
            eprintln!("DB error: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
        }
    };

    // Fetch page rows
    let rows = sqlx::query_as::<_, ReportSummary>(
        "SELECT id, created_at FROM report ORDER BY created_at DESC LIMIT $1 OFFSET $2",
    )
    .bind(per_page as i64)
    .bind(offset as i64)
    .fetch_all(&state.db)
    .await;

    let list = match rows {
        Ok(list) => list,
        Err(e) => {
            eprintln!("DB error: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
        }
    };

    // Build items with formatted dates (UTC+7)
    let mut items: Vec<ReportListItem> = Vec::new();
    for r in list {
        let dt = r.created_at + chrono::Duration::hours(7);
        let created_date = dt.format("%d/%m/%Y").to_string();
        let created_time = format!("{} UTC+7", dt.format("%H:%M:%S"));
        items.push(ReportListItem { id: r.id, created_date, created_time });
    }

    // Compute pages
    let pages = if total == 0 { 1 } else { ((total as f64) / (per_page as f64)).ceil() as i64 };

    // Build simple page numbers similar to Flask pagination.iter_pages
    let mut page_numbers: Vec<Option<i64>> = Vec::new();
    if pages <= 10 {
        for p in 1..=pages { page_numbers.push(Some(p)); }
    } else {
        // always show first 1-2, last 1-2, and current +/-2 with ellipses
        let mut added = std::collections::HashSet::new();
        let push = |vec: &mut Vec<Option<i64>>, v: i64, added: &mut std::collections::HashSet<i64>| {
            if !added.contains(&v) {
                vec.push(Some(v));
                added.insert(v);
            }
        };
        push(&mut page_numbers, 1, &mut added);
        push(&mut page_numbers, 2, &mut added);
        for v in (page-2)..=(page+2) { if v>2 && v<pages-1 { push(&mut page_numbers, v, &mut added); } }
        push(&mut page_numbers, pages-1, &mut added);
        push(&mut page_numbers, pages, &mut added);

        // sort and insert None where gaps >1
        let mut nums: Vec<i64> = page_numbers.iter().filter_map(|o| *o).collect();
        nums.sort();
        page_numbers.clear();
        let mut last: Option<i64> = None;
        for n in nums {
            if let Some(l) = last {
                if n - l > 1 {
                    page_numbers.push(None);
                }
            }
            page_numbers.push(Some(n));
            last = Some(n);
        }
    }

    // Build reports context
    let display_start = if total == 0 { 0 } else { offset + 1 };
    let display_end = offset + (items.len() as i64);

    let reports = json!({
        "items": items,
        "total": total,
        "per_page": per_page,
        "page": page,
        "pages": pages,
        "has_prev": page > 1,
        "has_next": page < pages,
        "prev_num": if page > 1 { page - 1 } else { 1 },
        "next_num": if page < pages { page + 1 } else { pages },
        "page_numbers": page_numbers,
        "display_start": display_start,
        "display_end": display_end,
    });

    // Render template
    let mut context = Context::new();
    context.insert("reports", &reports);

    match state.tera.render("report_list.html", &context) {
        Ok(html) => Html(html).into_response(),
        Err(e) => {
            eprintln!("Template render error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Template render error").into_response()
        }
    }
}

async fn upload_page() -> Response {
    match fs::read_to_string("static/upload.html").await {
        Ok(s) => Html(s).into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "Upload page not found").into_response(),
    }
}

async fn auto_update(Path(secret): Path<String>, State(state): State<Arc<AppState>>) -> Response {
    match &state.auto_update_secret {
        None => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "Auto update system not configured", "message": "Set AUTO_UPDATE_SECRET_KEY in .env"})),
        )
            .into_response(),
        Some(required) => {
            if &secret != required {
                eprintln!("Unauthorized access attempt with key: {}", secret);
                (StatusCode::FORBIDDEN, Json(serde_json::json!({"error": "Access denied", "message": "Invalid secret key"}))).into_response()
            } else {
                match fs::read_to_string("static/auto_update.html").await {
                    Ok(s) => Html(s).into_response(),
                    Err(_) => (StatusCode::NOT_FOUND, "Auto update page not found").into_response(),
                }
            }
        }
    }
}

async fn get_chart_modules_content(state: &AppState) -> String {
    use tokio::fs::read_dir;
    use std::path::Path;

    // If not in debug mode, try cache first
    let debug = env::var("DEBUG").unwrap_or_default() == "1";
    if !debug {
        if let Some(cached) = state.chart_modules_cache.read().await.clone() {
            return cached;
        }
    }

    let source_dir = Path::new("static").join("js").join("chart_modules");

    let priority_order = vec!["gauge.js", "bar.js", "line.js", "doughnut.js"];

    let mut entries = match read_dir(&source_dir).await {
        Ok(rd) => rd,
        Err(_) => return "// No chart modules found".to_string(),
    };

    let mut all_files = Vec::new();
    while let Ok(Some(entry)) = entries.next_entry().await {
        if let Ok(ft) = entry.file_type().await {
            if ft.is_file() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.ends_with(".js") {
                        all_files.push(name.to_string());
                    }
                }
            }
        }
    }

    // Order files: priority first, then alphabetically
    let mut ordered = Vec::new();
    for p in &priority_order {
        if let Some(idx) = all_files.iter().position(|f| f == p) {
            ordered.push(all_files.remove(idx));
        }
    }
    all_files.sort();
    ordered.extend(all_files);

    let mut parts: Vec<String> = Vec::new();
    for filename in ordered {
        let path = source_dir.join(&filename);
        match tokio::fs::read_to_string(&path).await {
            Ok(content) => {
                let wrapped = format!("// ==================== {name} ====================\ntry {{\n{code}\n}} catch (error) {{\n    console.error('Error loading chart module {name}:', error);\n}}\n// ==================== End {name} ====================", name=filename, code=content);
                parts.push(wrapped);
            }
            Err(_) => {
                parts.push(format!("// Warning: {name} not found", name=filename));
            }
        }
    }

    let final_content = parts.join("\n\n");

    // Cache if not debug
    if !debug {
        let mut w = state.chart_modules_cache.write().await;
        *w = Some(final_content.clone());
    }

    final_content
}
