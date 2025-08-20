//! Legacy Compatibility Routes
//! 
//! This module provides backward compatibility for old route patterns.
//! These routes redirect or serve content for legacy URLs to maintain compatibility.

use axum::Router;
use tower_http::services::ServeDir;
use std::sync::Arc;

use crate::service_islands::ServiceIslands;

/// Configure legacy compatibility routes
/// 
/// Provides backward compatibility for:
/// - Old /assets paths
/// - Old /static paths
/// - Any other deprecated route patterns
pub fn configure_legacy_routes() -> Router<Arc<ServiceIslands>> {
    Router::new()
        // Legacy compatibility routes - redirect old paths to new crypto dashboard assets
        .nest_service("/assets", ServeDir::new("dashboards/crypto_dashboard/assets"))
        .nest_service("/static", ServeDir::new("dashboards/crypto_dashboard/assets"))
}
