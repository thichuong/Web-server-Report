//! Routes Module
//!
//! This module organizes all route handlers for the Service Islands Architecture.
//! Routes are split into logical modules for better maintainability and organization.

pub mod api;
pub mod crypto_reports;
pub mod homepage;
pub mod rss_feed;
pub mod seo;
pub mod static_files;
pub mod system;
// WebSocket module moved to separate Web-server-Report-websocket service
// pub mod websocket;

use axum::Router;
use std::sync::Arc;

use crate::service_islands::ServiceIslands;

/// Create the main Service Islands router by combining all route modules
pub fn create_service_islands_router(service_islands: Arc<ServiceIslands>) -> Router {
    Router::new()
        // Static file serving
        .merge(static_files::configure_static_routes())
        // Homepage
        .merge(homepage::configure_homepage_route())
        // Health and system endpoints
        .merge(system::configure_system_routes())
        // Crypto Reports routes
        .merge(crypto_reports::configure_crypto_reports_routes())
        // API endpoints
        .merge(api::configure_api_routes())
        // SEO endpoints (sitemap.xml)
        .merge(seo::configure_seo_routes())
        // RSS feed endpoint
        .merge(rss_feed::configure_rss_routes())
        // Note: WebSocket endpoint has been moved to Web-server-Report-websocket service
        // Client should connect to separate websocket service (port 8081)
        .with_state(service_islands)
}
