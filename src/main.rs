use axum::{extract::{Path, Query, State}, http::StatusCode, response::{Html, IntoResponse, Response}, routing::get, Json, Router};
use dotenvy::dotenv;
use serde::Serialize;
use sqlx::FromRow;
use sqlx::PgPool;
use std::{env, net::SocketAddr, sync::Arc};
use tokio::fs;

#[derive(Clone)]
struct AppState {
    db: PgPool,
    auto_update_secret: Option<String>,
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

    let state = AppState { db: pool, auto_update_secret };
    let shared_state = Arc::new(state);

    let app = Router::new()
        .route("/health", get(health))
        .route("/", get(index))
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

    match rec {
        Ok(Some(report)) => Html(report.html_content).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "No reports found").into_response(),
        Err(e) => {
            eprintln!("DB error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response()
        }
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
        Ok(Some(report)) => Html(report.html_content).into_response(),
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

async fn report_list(Query(params): Query<std::collections::HashMap<String, String>>, State(state): State<Arc<AppState>>) -> Response {
    let page: i64 = params.get("page").and_then(|p| p.parse().ok()).unwrap_or(1);
    let per_page: i64 = 10;
    let offset = (page - 1) * per_page;

    let rows = sqlx::query_as::<_, ReportSummary>(
        "SELECT id, created_at FROM report ORDER BY created_at DESC LIMIT $1 OFFSET $2",
    )
    .bind(per_page as i64)
    .bind(offset as i64)
    .fetch_all(&state.db)
    .await;

    match rows {
        Ok(list) => Json(list).into_response(),
        Err(e) => {
            eprintln!("DB error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response()
        }
    }
}

async fn upload_page() -> Response {
    match fs::read_to_string("Web-server-Report/static/upload.html").await {
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
                match fs::read_to_string("Web-server-Report/static/auto_update.html").await {
                    Ok(s) => Html(s).into_response(),
                    Err(_) => (StatusCode::NOT_FOUND, "Auto update page not found").into_response(),
                }
            }
        }
    }
}
