use axum::{routing::get, Router};
use tower_http::services::ServeDir;
use std::sync::Arc;

use crate::{handlers::*, state::AppState};

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
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
        
        // Health and monitoring endpoints
        .route("/health", get(health))
        .route("/metrics", get(performance_metrics))
        .route("/admin/cache/clear", get(clear_cache))
        .route("/admin/cache/stats", get(cache_stats))
        
        // Main application routes
        .route("/", get(homepage))
        .route("/crypto_report", get(crypto_index))
        .route("/crypto_report/:id", get(crypto_view_report))
        .route("/pdf-template/:id", get(pdf_template))
        .route("/crypto_reports_list", get(report_list))
        
        // WebSocket and API routes
        .route("/ws", get(websocket_handler))
        .route("/api/crypto/dashboard-summary", get(api_dashboard_summary)) // New cached endpoint
        .route("/api/crypto/dashboard-summary/cached", get(dashboard_summary_api))
        .route("/api/crypto/dashboard-summary/refresh", get(force_refresh_dashboard))
        .route("/api/crypto/rate-limit-status", get(api_rate_limit_status)) // Rate limit monitoring
        .route("/shared_assets/js/chart_modules.js", get(serve_chart_modules))
        
        .with_state(state)
}
