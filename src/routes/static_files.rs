//! Static File Serving Routes
//! 
//! This module handles all static file serving routes for the Service Islands Architecture.
//! It serves assets, shared components, and dashboard-specific files.

use axum::Router;
use tower_http::services::ServeDir;
use std::sync::Arc;

use crate::service_islands::ServiceIslands;

/// Configure static file serving routes
/// 
/// Sets up all static file serving including:
/// - Crypto dashboard assets
/// - Stock dashboard assets  
/// - Shared components and assets
/// - Legacy compatibility routes
pub fn configure_static_routes(service_islands: Arc<ServiceIslands>) -> Router<Arc<ServiceIslands>> {
    Router::new()
        // Crypto Dashboard static files
        .nest_service("/crypto_dashboard/shared", ServeDir::new("dashboards/crypto_dashboard/shared"))
        .nest_service("/crypto_dashboard/routes", ServeDir::new("dashboards/crypto_dashboard/routes"))
        .nest_service("/crypto_dashboard/assets", ServeDir::new("dashboards/crypto_dashboard/assets"))
        .nest_service("/crypto_dashboard/pages", ServeDir::new("dashboards/crypto_dashboard/pages"))
        
        // Stock Dashboard static files
        .nest_service("/stock_dashboard/shared", ServeDir::new("dashboards/stock_dashboard/shared"))
        .nest_service("/stock_dashboard/routes", ServeDir::new("dashboards/stock_dashboard/routes"))
        .nest_service("/stock_dashboard/assets", ServeDir::new("dashboards/stock_dashboard/assets"))
        .nest_service("/stock_dashboard/pages", ServeDir::new("dashboards/stock_dashboard/pages"))
        
        // Shared components and assets
        .nest_service("/shared_components", ServeDir::new("shared_components"))
        .nest_service("/shared_assets", ServeDir::new("shared_assets"))
}
