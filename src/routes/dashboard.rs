//! Dashboard Routes
//! 
//! This module handles all dashboard-related routes including main dashboard,
//! crypto dashboard, and stock dashboard pages.

use axum::{
    routing::get,
    Router,
    response::Html,
    extract::State
};
use std::sync::Arc;

use crate::service_islands::ServiceIslands;

/// Configure dashboard routes
pub fn configure_dashboard_routes() -> Router<Arc<ServiceIslands>> {
    Router::new()
        .route("/dashboard", get(dashboard_index))
        .route("/dashboard/crypto", get(crypto_dashboard))
        .route("/dashboard/stock", get(stock_dashboard))
}

/// Main dashboard index page
async fn dashboard_index(
    State(_service_islands): State<Arc<ServiceIslands>>
) -> Html<&'static str> {
    Html(r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>Investment Dashboard</title>
        <style>
            body { font-family: Arial, sans-serif; margin: 20px; }
            .dashboard { display: grid; grid-template-columns: 1fr 1fr; gap: 20px; }
            .card { padding: 20px; border: 1px solid #ddd; border-radius: 8px; }
        </style>
    </head>
    <body>
        <h1>📊 Investment Dashboard</h1>
        <div class="dashboard">
            <div class="card">
                <h3>🪙 Crypto Dashboard</h3>
                <p>Real-time cryptocurrency analysis</p>
                <a href="/dashboard/crypto">Open Crypto Dashboard</a>
            </div>
            <div class="card">
                <h3>📈 Stock Dashboard</h3>
                <p>Stock market analysis and reports</p>
                <a href="/dashboard/stock">Open Stock Dashboard</a>
            </div>
        </div>
    </body>
    </html>
    "#)
}

/// Crypto dashboard page
async fn crypto_dashboard(
    State(_service_islands): State<Arc<ServiceIslands>>
) -> Html<&'static str> {
    Html(r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>Crypto Dashboard</title>
        <link rel="stylesheet" href="/shared_assets/css/style.css">
    </head>
    <body>
        <h1>🪙 Crypto Dashboard</h1>
        <div id="crypto-dashboard">
            <p>Loading crypto dashboard...</p>
        </div>
        <script src="/shared_assets/js/chart_modules/crypto-charts.js"></script>
    </body>
    </html>
    "#)
}

/// Stock dashboard page
async fn stock_dashboard(
    State(_service_islands): State<Arc<ServiceIslands>>
) -> Html<&'static str> {
    Html(r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>Stock Dashboard</title>
        <link rel="stylesheet" href="/shared_assets/css/style.css">
    </head>
    <body>
        <h1>📈 Stock Dashboard</h1>
        <div id="stock-dashboard">
            <p>Loading stock dashboard...</p>
        </div>
    </body>
    </html>
    "#)
}
