//! Dashboard Handlers - HTTP request handlers for dashboard endpoints
//!
//! Provides Axum route handlers that integrate with Dashboard Service Island
//! components for serving dashboard pages and API endpoints.

use crate::features::dashboard::{Dashboard, TemplateRenderer, ReportManager, UiComponents};
use crate::state::AppState;
use axum::{
    extract::{Path, Query, State},
    response::{Html, IntoResponse, Response},
    routing::{get, Router},
    http::StatusCode,
};
use std::collections::HashMap;
use std::sync::{Arc, atomic::Ordering};

/// Create dashboard routes
pub fn dashboard_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(homepage))
        .route("/crypto", get(crypto_index))
        .route("/crypto/", get(crypto_index))
        .route("/crypto/reports", get(report_list))
        .route("/crypto/reports/:id", get(crypto_view_report))
        .route("/crypto/reports/:id/pdf", get(pdf_template))
        .route("/crypto/chart-modules.js", get(serve_chart_modules))
}

/// Homepage handler - Renders main dashboard page
pub async fn homepage(State(state): State<Arc<AppState>>) -> Response {
    // Increment request counter
    state.request_counter.fetch_add(1, Ordering::Relaxed);

    // Create minimal UI components for standalone operation
    let ui_components = UiComponents::default();
    
    match read_homepage_file(&ui_components).await {
        Ok(content) => ui_components.create_html_response(content),
        Err(_) => {
            let fallback_html = r#"<!DOCTYPE html>
<html lang="vi">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>AI Investment Report Dashboard</title>
</head>
<body>
    <div class="container">
        <h1>AI Investment Report Dashboard</h1>
        <p>Chào mừng bạn đến với bảng điều khiển báo cáo đầu tư AI.</p>
        <nav>
            <a href="/crypto">Crypto Dashboard</a>
            <a href="/crypto/reports">Báo cáo</a>
        </nav>
    </div>
</body>
</html>"#;
            ui_components.create_html_response(fallback_html.to_string())
        }
    }
}

/// Crypto dashboard index handler
pub async fn crypto_index(State(state): State<Arc<AppState>>) -> Response {
    // Increment request counter  
    state.request_counter.fetch_add(1, Ordering::Relaxed);

    // Create dashboard components for rendering
    let template_renderer = TemplateRenderer::default();
    let ui_components = UiComponents::default();
    
    match template_renderer.render_dashboard_view(None).await {
        Ok(html) => ui_components.create_cached_response(html, "dashboard"),
        Err(e) => {
            eprintln!("Dashboard render error: {}", e);
            ui_components.create_error_response(StatusCode::INTERNAL_SERVER_ERROR, "Dashboard render error")
        }
    }
}

/// View specific crypto report
pub async fn crypto_view_report(Path(id): Path<i32>, State(state): State<Arc<AppState>>) -> Response {
    // Increment request counter
    state.request_counter.fetch_add(1, Ordering::Relaxed);

    let template_renderer = TemplateRenderer::default();
    let ui_components = UiComponents::default();
    
    match template_renderer.render_crypto_template("crypto/routes/reports/view.html", id, None).await {
        Ok(html) => ui_components.create_cached_response(html, "report_view"),
        Err(e) => {
            eprintln!("Report render error: {}", e);
            if e.to_string().contains("not found") {
                ui_components.create_error_response(StatusCode::NOT_FOUND, "Report not found")
            } else {
                ui_components.create_error_response(StatusCode::INTERNAL_SERVER_ERROR, "Template render error")
            }
        }
    }
}

/// PDF template handler
pub async fn pdf_template(Path(id): Path<i32>, State(state): State<Arc<AppState>>) -> Response {
    // Increment request counter
    state.request_counter.fetch_add(1, Ordering::Relaxed);

    let template_renderer = TemplateRenderer::default();
    
    match template_renderer.render_pdf_template(id).await {
        Ok(html) => Html(html).into_response(),
        Err(e) => {
            eprintln!("PDF template error: {}", e);
            if e.to_string().contains("not found") {
                (StatusCode::NOT_FOUND, "Report not found").into_response()
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, "Template render error").into_response()
            }
        }
    }
}

/// Report list with pagination
pub async fn report_list(Query(params): Query<HashMap<String, String>>, State(state): State<Arc<AppState>>) -> Response {
    // Increment request counter
    state.request_counter.fetch_add(1, Ordering::Relaxed);

    let page = params.get("page").and_then(|p| p.parse().ok()).unwrap_or(1);
    
    let report_manager = ReportManager::default();
    let template_renderer = TemplateRenderer::default();
    let ui_components = UiComponents::default();

    match report_manager.get_report_list(Some(page)).await {
        Ok(reports_data) => {
            // Render report list template
            match template_renderer.render_crypto_template("crypto/routes/reports/list.html", 0, Some(reports_data)).await {
                Ok(html) => ui_components.create_html_response(html),
                Err(e) => {
                    eprintln!("Report list template error: {}", e);
                    ui_components.create_error_response(StatusCode::INTERNAL_SERVER_ERROR, "Template render error")
                }
            }
        }
        Err(e) => {
            eprintln!("Report list error: {}", e);
            ui_components.create_error_response(StatusCode::INTERNAL_SERVER_ERROR, "Database error")
        }
    }
}

/// Serve chart modules JavaScript
pub async fn serve_chart_modules(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let ui_components = UiComponents::default();
    ui_components.serve_chart_modules().await
}

/// Read homepage HTML file
async fn read_homepage_file(ui_components: &UiComponents) -> Result<String, std::io::Error> {
    // Try to read homepage file
    let homepage_paths = [
        "dashboards/home.html",
        "src/templates/home.html", 
        "templates/home.html",
    ];

    for path in &homepage_paths {
        match tokio::fs::read_to_string(path).await {
            Ok(content) => return Ok(content),
            Err(_) => continue,
        }
    }

    // Return fallback if no file found
    Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Homepage file not found"))
}
