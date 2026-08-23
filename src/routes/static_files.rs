//! Static File Serving Routes
//!
//! This module handles all static file serving routes for the Service Islands Architecture.
//! It serves assets, shared components, and dashboard-specific files.
//! ✅ OPTIMIZED: Cache-Control headers for browser caching of static assets

use axum::Router;
use axum::http::{HeaderValue, header};
use std::sync::Arc;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::set_header::SetResponseHeaderLayer;

use crate::state::AppState;

/// Configure static file serving routes
///
/// Sets up all static file serving including:
/// - Crypto dashboard assets
/// - Stock dashboard assets (minimal)
/// - Shared components and assets
/// - Test files
///
/// ✅ OPTIMIZED: All static files get `Cache-Control: public, max-age=86400` (24 hours)
pub fn configure_static_routes() -> Router<Arc<AppState>> {
    Router::new()
        // SEO files
        .route_service("/robots.txt", ServeFile::new("robots.txt"))
        // Main Frontend directory
        .nest_service("/frontend", ServeDir::new("frontend"))
        // Backward compatibility / convenience aliases
        .nest_service("/shared_assets", ServeDir::new("frontend/shared/assets"))
        .nest_service("/shared_components", ServeDir::new("frontend/shared/components"))
        // Test file
        .nest_service("/test", ServeDir::new("."))
        // Add Cache-Control header for all static files (24 hours)
        .layer(SetResponseHeaderLayer::if_not_present(
            header::CACHE_CONTROL,
            HeaderValue::from_static("public, max-age=86400"),
        ))
}
