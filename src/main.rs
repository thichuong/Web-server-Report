use axum::{extract::{Path, Query, State, ws::WebSocketUpgrade}, http::StatusCode, response::{Html, IntoResponse, Response}, routing::get, Json, Router};
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
use std::error::Error as StdError;

mod data_service;
mod websocket_service;

use data_service::DataService;

struct AppState {
    db: PgPool,
    auto_update_secret: Option<String>,
    // cache for concatenated chart modules JS (None until populated)
    chart_modules_cache: RwLock<Option<String>>,
    // cache reports by id to avoid querying DB on every view request
    cached_reports: RwLock<std::collections::HashMap<i32, Report>>,
    // cached latest report id for quick lookup of the most recent report
    cached_latest_id: RwLock<Option<i32>>,
    // Tera template engine
    tera: Tera,
    // WebSocket service for real-time updates
    websocket_service: Arc<crate::websocket_service::WebSocketService>,
}

#[derive(FromRow, Serialize, Debug, Clone)]
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
    let taapi_secret = env::var("TAAPI_SECRET").expect("TAAPI_SECRET must be set in .env");
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

    let pool = PgPool::connect(&database_url).await?;

    // Initialize data service
    let data_service = DataService::new(taapi_secret);
    
    // Initialize WebSocket service
    let websocket_service = Arc::new(crate::websocket_service::WebSocketService::new(&redis_url, data_service)?);
    
    // Start background data updates
    websocket_service.start_background_updates().await;

    // Initialize Tera template engine with new architecture
    let mut tera = Tera::default();
    
    // Load shared components (global)
    tera.add_template_file("shared_components/theme_toggle.html", Some("shared/components/theme_toggle.html")).expect("Failed to load shared theme_toggle.html");
    tera.add_template_file("shared_components/language_toggle.html", Some("shared/components/language_toggle.html")).expect("Failed to load shared language_toggle.html");
    
    // Load route-specific templates for crypto_dashboard
    tera.add_template_file("dashboards/crypto_dashboard/routes/reports/view.html", Some("crypto/routes/reports/view.html")).expect("Failed to load crypto reports view template");
    tera.add_template_file("dashboards/crypto_dashboard/routes/reports/pdf.html", Some("crypto/routes/reports/pdf.html")).expect("Failed to load crypto reports pdf template");
    tera.add_template_file("dashboards/crypto_dashboard/routes/reports/list.html", Some("crypto/routes/reports/list.html")).expect("Failed to load crypto reports list template");
    
    // Load legacy templates for backwards compatibility (keeping for fallback)
    // Add legacy components as well for backwards compatibility
    tera.add_template_file("shared_components/theme_toggle.html", Some("crypto/components/theme_toggle.html")).expect("Failed to load legacy crypto theme_toggle.html");
    tera.add_template_file("shared_components/language_toggle.html", Some("crypto/components/language_toggle.html")).expect("Failed to load legacy crypto language_toggle.html");
    
    tera.autoescape_on(vec![]); // Disable auto-escaping for safe content

    let state = AppState { 
        db: pool, 
        auto_update_secret, 
        chart_modules_cache: RwLock::new(None),
    cached_reports: RwLock::new(std::collections::HashMap::new()),
    cached_latest_id: RwLock::new(None),
        tera,
        websocket_service,
    };
    let shared_state = Arc::new(state);

    // Prime the latest-report cache once at startup to reduce first-request latency
    // (best-effort; failure won't stop the server)
    {
        let s = Arc::clone(&shared_state);
        let pool_ref = s.db.clone();
        tokio::spawn(async move {
        if let Ok(Some(report)) = sqlx::query_as::<_, Report>(
                "SELECT id, html_content, css_content, js_content, html_content_en, js_content_en, created_at FROM report ORDER BY created_at DESC LIMIT 1",
            )
            .fetch_optional(&pool_ref)
            .await
            {
            // insert into per-id cache and mark as latest
            let mut mapw = s.cached_reports.write().await;
            mapw.insert(report.id, report.clone());
            let mut lid = s.cached_latest_id.write().await;
            *lid = Some(report.id);
            }
        });
    }

    // Serve crypto_dashboard assets at /crypto_dashboard/assets and keep a compatibility
    // mount for /static (optional) to avoid breaking external links. We also serve the
    // new asset path for the reorganized structure.
    let app = Router::new()
    // Serve crypto_dashboard assets
    .nest_service("/crypto_dashboard/shared", ServeDir::new("dashboards/crypto_dashboard/shared"))
    .nest_service("/crypto_dashboard/routes", ServeDir::new("dashboards/crypto_dashboard/routes"))
    .nest_service("/crypto_dashboard/assets", ServeDir::new("dashboards/crypto_dashboard/assets"))
    .nest_service("/crypto_dashboard/pages", ServeDir::new("dashboards/crypto_dashboard/pages"))
    
    // Serve stock_dashboard assets
    .nest_service("/stock_dashboard/shared", ServeDir::new("dashboards/stock_dashboard/shared"))
    .nest_service("/stock_dashboard/routes", ServeDir::new("dashboards/stock_dashboard/routes"))
    .nest_service("/stock_dashboard/assets", ServeDir::new("dashboards/stock_dashboard/assets"))
    .nest_service("/stock_dashboard/pages", ServeDir::new("dashboards/stock_dashboard/pages"))
    
    // Serve shared components and assets
    .nest_service("/shared_components", ServeDir::new("shared_components"))
    .nest_service("/shared_assets", ServeDir::new("shared_assets"))
    
    // Legacy compatibility routes
    .nest_service("/assets", ServeDir::new("dashboards/crypto_dashboard/assets"))
    .nest_service("/static", ServeDir::new("dashboards/crypto_dashboard/assets"))
        .route("/health", get(health))
        .route("/", get(homepage))
        .route("/crypto_report", get(crypto_index))
        .route("/crypto_report/:id", get(crypto_view_report))
        .route("/pdf-template/:id", get(pdf_template))
        .route("/crypto_reports_list", get(report_list))
        .route("/upload", get(upload_page))
        .route("/auto-update-system-:secret", get(auto_update))
        // New WebSocket and API routes
        .route("/ws", get(websocket_handler))
        .route("/api/crypto/dashboard-summary", get(dashboard_summary_api))
        .route("/api/crypto/dashboard-summary/refresh", get(force_refresh_dashboard))
        .route("/shared_assets/js/chart_modules.js", get(serve_chart_modules))
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

async fn crypto_index(State(state): State<Arc<AppState>>) -> Response {
    // Try cache first (fast path) using cached_latest_id -> cached_reports map
    if let Some(latest_id_opt) = *state.cached_latest_id.read().await {
        if let Some(cached) = state.cached_reports.read().await.get(&latest_id_opt).cloned() {
            let chart_modules_content = get_chart_modules_content(&state).await;
            let mut context = Context::new();
            context.insert("current_route", "dashboard");
            context.insert("current_lang", "vi");
            context.insert("current_time", &chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string());
            context.insert("report", &cached);
            context.insert("chart_modules_content", &chart_modules_content);
            let pdf_url = format!("/pdf-template/{}", cached.id);
            context.insert("pdf_url", &pdf_url);

            match state.tera.render("crypto/routes/reports/view.html", &context) {
                Ok(html) => {
                    return Response::builder()
                        .status(StatusCode::OK)
                        .header("cache-control", "public, max-age=15") // short client cache to reduce repeat hits
                        .header("content-type", "text/html; charset=utf-8")
                        .body(html)
                        .unwrap()
                        .into_response();
                }
                Err(e) => {
                    eprintln!("Template render error: {:#?}", e);
                    let mut src = e.source();
                    while let Some(s) = src {
                        eprintln!("Template render error source: {:#?}", s);
                        src = s.source();
                    }
                    return (StatusCode::INTERNAL_SERVER_ERROR, "Template render error").into_response();
                }
            }
        }
    }

    // Cache miss: fetch DB and chart modules concurrently to reduce latency
    let db_fut = sqlx::query_as::<_, Report>(
        "SELECT id, html_content, css_content, js_content, html_content_en, js_content_en, created_at FROM report ORDER BY created_at DESC LIMIT 1",
    ).fetch_optional(&state.db);
    let chart_fut = get_chart_modules_content(&state);

    let (db_res, chart_modules_content) = tokio::join!(db_fut, chart_fut);

    // Create Tera context
    let mut context = Context::new();
    context.insert("current_route", "dashboard");
    context.insert("current_lang", "vi");
    context.insert("current_time", &chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string());

    match db_res {
        Ok(Some(report)) => {
            // prime per-id cache and latest id
            {
                let mut mapw = state.cached_reports.write().await;
                mapw.insert(report.id, report.clone());
            }
            let mut lid = state.cached_latest_id.write().await;
            *lid = Some(report.id);
            context.insert("report", &report);
            let pdf_url = format!("/pdf-template/{}", report.id);
            context.insert("pdf_url", &pdf_url);
        }
        Ok(None) => {
            let empty_report = serde_json::json!({
                "html_content": "",
                "html_content_en": "",
                "css_content": "",
                "js_content": ""
            });
            context.insert("report", &empty_report);
            context.insert("pdf_url", &"#");
        }
        Err(e) => {
            eprintln!("DB error: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
        }
    }

    context.insert("chart_modules_content", &chart_modules_content);

    match state.tera.render("crypto/routes/reports/view.html", &context) {
        Ok(html) => Response::builder()
                .status(StatusCode::OK)
                .header("cache-control", "public, max-age=15")
                .header("content-type", "text/html; charset=utf-8")
                .body(html)
                .unwrap()
                .into_response(),
        Err(e) => {
            eprintln!("Template render error: {:#?}", e);
            let mut src = e.source();
            while let Some(s) = src {
                eprintln!("Template render error source: {:#?}", s);
                src = s.source();
            }
            (StatusCode::INTERNAL_SERVER_ERROR, "Template render error").into_response()
        }
    }
}

async fn homepage() -> Response {
    match fs::read_to_string("dashboards/home.html").await {
        Ok(s) => Html(s).into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "Home page not found").into_response(),
    }
}

async fn crypto_view_report(Path(id): Path<i32>, State(state): State<Arc<AppState>>) -> Response {
    // Fast path: check per-id cache
    if let Some(cached) = state.cached_reports.read().await.get(&id).cloned() {
        let chart_modules_content = get_chart_modules_content(&state).await;
        let mut context = Context::new();
        context.insert("report", &cached);
        context.insert("chart_modules_content", &chart_modules_content);
        let pdf_url = format!("/pdf-template/{}", cached.id);
        context.insert("pdf_url", &pdf_url);

        match state.tera.render("crypto/routes/reports/view.html", &context) {
            Ok(html) => return Response::builder()
                .status(StatusCode::OK)
                .header("cache-control", "public, max-age=15")
                .header("content-type", "text/html; charset=utf-8")
                .body(html)
                .unwrap()
                .into_response(),
            Err(e) => {
                eprintln!("Template render error: {:#?}", e);
                let mut src = e.source();
                while let Some(s) = src {
                    eprintln!("Template render error source: {:#?}", s);
                    src = s.source();
                }
                return (StatusCode::INTERNAL_SERVER_ERROR, "Template render error").into_response();
            }
        }
    }

    // Cache miss or different id: fetch DB and chart modules concurrently
    let db_fut = sqlx::query_as::<_, Report>(
        "SELECT id, html_content, css_content, js_content, html_content_en, js_content_en, created_at FROM report WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db);

    let chart_fut = get_chart_modules_content(&state);

    let (db_res, chart_modules_content) = tokio::join!(db_fut, chart_fut);

    match db_res {
        Ok(Some(report)) => {
            // Insert into per-id cache and maybe update latest id
            {
                let mut mapw = state.cached_reports.write().await;
                mapw.insert(report.id, report.clone());
            }
            // update latest id if this report is newer
            {
                let mut lid = state.cached_latest_id.write().await;
                match *lid {
                    None => {
                        *lid = Some(report.id);
                    }
                    Some(existing_id) => {
                        // compare timestamps using cached_reports map
                        let compare = {
                            let read_map = state.cached_reports.read().await;
                            read_map.get(&existing_id).map(|r| r.created_at)
                        };
                        if compare.map_or(true, |t| report.created_at > t) {
                            *lid = Some(report.id);
                        }
                    }
                }
            }

            // Build context and render
            let mut context = Context::new();
            context.insert("report", &report);
            context.insert("chart_modules_content", &chart_modules_content);
            let pdf_url = format!("/pdf-template/{}", report.id);
            context.insert("pdf_url", &pdf_url);

            match state.tera.render("crypto/routes/reports/view.html", &context) {
                Ok(html) => Response::builder()
                        .status(StatusCode::OK)
                        .header("cache-control", "public, max-age=15")
                        .header("content-type", "text/html; charset=utf-8")
                        .body(html)
                        .unwrap()
                        .into_response(),
                Err(e) => {
                    eprintln!("Template render error: {:#?}", e);
                    let mut src = e.source();
                    while let Some(s) = src {
                        eprintln!("Template render error source: {:#?}", s);
                        src = s.source();
                    }
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
    // Fast path: return cached report if present
    if let Some(cached) = state.cached_reports.read().await.get(&id).cloned() {
        let chart_modules_content = get_chart_modules_content(&state).await;

        let mut context = Context::new();
        context.insert("report", &cached);
        context.insert("chart_modules_content", &chart_modules_content);

        // formatted created date in UTC+7 for display
        let created_display = (cached.created_at + chrono::Duration::hours(7)).format("%d-%m-%Y %H:%M").to_string();
        context.insert("created_at_display", &created_display);

        match state.tera.render("crypto/routes/reports/pdf.html", &context) {
            Ok(html) => return Html(html).into_response(),
            Err(e) => {
                eprintln!("Template render error: {:#?}", e);
                let mut src = e.source();
                while let Some(s) = src {
                    eprintln!("Template render error source: {:#?}", s);
                    src = s.source();
                }
                return (StatusCode::INTERNAL_SERVER_ERROR, "Template render error").into_response();
            }
        }
    }

    // Cache miss: fetch DB and chart modules concurrently
    let db_fut = sqlx::query_as::<_, Report>(
        "SELECT id, html_content, css_content, js_content, html_content_en, js_content_en, created_at FROM report WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db);

    let chart_fut = get_chart_modules_content(&state);

    let (db_res, chart_modules_content) = tokio::join!(db_fut, chart_fut);

    match db_res {
        Ok(Some(report)) => {
            // insert into per-id cache
            {
                let mut mapw = state.cached_reports.write().await;
                mapw.insert(report.id, report.clone());
            }

            // optionally update latest id if this is newer
            {
                let mut lid = state.cached_latest_id.write().await;
                match *lid {
                    None => *lid = Some(report.id),
                    Some(existing_id) => {
                        let compare = {
                            let read_map = state.cached_reports.read().await;
                            read_map.get(&existing_id).map(|r| r.created_at)
                        };
                        if compare.map_or(true, |t| report.created_at > t) {
                            *lid = Some(report.id);
                        }
                    }
                }
            }

            let mut context = Context::new();
            context.insert("report", &report);
            context.insert("chart_modules_content", &chart_modules_content);

            // formatted created date in UTC+7 for display
            let created_display = (report.created_at + chrono::Duration::hours(7)).format("%d-%m-%Y %H:%M").to_string();
            context.insert("created_at_display", &created_display);

            match state.tera.render("crypto/routes/reports/pdf.html", &context) {
                Ok(html) => Html(html).into_response(),
                Err(e) => {
                    eprintln!("Template render error: {:#?}", e);
                    let mut src = e.source();
                    while let Some(s) = src {
                        eprintln!("Template render error source: {:#?}", s);
                        src = s.source();
                    }
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

    match state.tera.render("crypto/routes/reports/list.html", &context) {
        Ok(html) => Html(html).into_response(),
        Err(e) => {
            eprintln!("Template render error: {:#?}", e);
            let mut src = e.source();
            while let Some(s) = src {
                eprintln!("Template render error source: {:#?}", s);
                src = s.source();
            }
            // Dump the reports JSON context to help diagnose template issues
            match serde_json::to_string_pretty(&reports) {
                Ok(s) => eprintln!("reports context: {}", s),
                Err(_) => eprintln!("Failed to serialize reports context for debugging"),
            }
            (StatusCode::INTERNAL_SERVER_ERROR, "Template render error").into_response()
        }
    }
}

async fn upload_page() -> Response {
    match fs::read_to_string("crypto_dashboard/templates/upload.html").await {
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
                match fs::read_to_string("crypto_dashboard/templates/auto_update.html").await {
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

    let source_dir = Path::new("shared_assets").join("js").join("chart_modules");

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

async fn serve_chart_modules(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let content = get_chart_modules_content(&state).await;
    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/javascript")
        .header("cache-control", "public, max-age=3600")
        .body(content)
        .unwrap()
}

// WebSocket handler for real-time updates
async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| async move {
        state.websocket_service.handle_websocket(socket).await;
    })
}

// API endpoint to get cached dashboard summary
async fn dashboard_summary_api(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match state.websocket_service.get_cached_dashboard_data().await {
        Ok(Some(data)) => Json(data).into_response(),
        Ok(None) => {
            // No cached data, try to fetch fresh data
            match state.websocket_service.force_update_dashboard().await {
                Ok(data) => Json(data).into_response(),
                Err(e) => {
                    eprintln!("Failed to fetch dashboard data: {}", e);
                    (
                        StatusCode::SERVICE_UNAVAILABLE,
                        Json(json!({
                            "error": "Service temporarily unavailable",
                            "message": "Unable to fetch dashboard data"
                        }))
                    ).into_response()
                }
            }
        }
        Err(e) => {
            eprintln!("Redis error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Internal server error",
                    "message": "Database connection failed"
                }))
            ).into_response()
        }
    }
}

// API endpoint to force refresh dashboard data
async fn force_refresh_dashboard(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match state.websocket_service.force_update_dashboard().await {
        Ok(data) => Json(json!({
            "status": "success",
            "message": "Dashboard data refreshed",
            "data": data
        })).into_response(),
        Err(e) => {
            eprintln!("Failed to refresh dashboard data: {}", e);
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({
                    "error": "Service temporarily unavailable",
                    "message": format!("Unable to refresh dashboard data: {}", e)
                }))
            ).into_response()
        }
    }
}
