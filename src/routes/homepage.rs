//! Homepage Route
//!
//! This module handles the main homepage route using the Service Islands Architecture.
//! The homepage is served through the Dashboard Island.

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
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
    // Directly await the async method from Layer 5
    match service_islands
        .dashboard
        .handlers
        .homepage_with_tera(&service_islands.get_legacy_app_state())
    {
        Ok(compressed_data) => {
            info!("✅ [Route] Compressed homepage rendered successfully from Layer 5");
            DashboardHandlers::create_compressed_response(compressed_data)
        }
        Err(e) => {
            error!("❌ Homepage rendering failed: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response()
        }
    }
}
