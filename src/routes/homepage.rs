//! Homepage Route
//!
//! This module handles the main homepage route using the Service Islands Architecture.
//! The homepage is served through the Dashboard Island.

use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use std::sync::Arc;
use tracing::{error, info};

use crate::service_islands::layer5_business_logic::dashboard::handlers::DashboardHandlers;
use crate::service_islands::ServiceIslands;

/// Configure homepage route
pub fn configure_homepage_route() -> Router<Arc<ServiceIslands>> {
    Router::new().route("/", get(homepage))
}

/// Homepage handler - delegates to Dashboard Island
async fn homepage(State(service_islands): State<Arc<ServiceIslands>>) -> Response {
    // Use the dashboard island's homepage handler with Tera rendering
    match service_islands
        .dashboard
        .handlers
        .homepage_with_tera(&service_islands.get_legacy_app_state())
        .await
    {
        Ok(compressed_data) => {
            info!("✅ [Route] Compressed homepage rendered successfully from Layer 5");
            // Use create_compressed_response for compressed data with proper headers
            DashboardHandlers::create_compressed_response(compressed_data)
        }
        Err(e) => {
            error!(
                "❌ Homepage rendering failed, falling back to simple handler: {}",
                e
            );
            // Fallback to simple homepage method
            match service_islands.dashboard.handlers.homepage().await {
                Ok(html) => Html(html).into_response(),
                Err(_) => (StatusCode::NOT_FOUND, "Home page not found").into_response(),
            }
        }
    }
}
