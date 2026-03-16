//! Homepage Route
//!
//! This module handles the main homepage route using the Service Islands Architecture.
//! The homepage is served through the Dashboard Island.

use axum::{
    Router,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use std::sync::Arc;
use tracing::{debug, error, info};

use crate::services::dashboard::DashboardHandlers;
use crate::state::AppState;

/// Configure homepage route
pub fn configure_homepage_route() -> Router<Arc<AppState>> {
    Router::new().route("/", get(homepage))
}

async fn homepage(State(state): State<Arc<AppState>>) -> Response {
    // ⚡ IMMEDIATE CACHE CHECK: Optimized RAM caching
    // Check if homepage is already pre-rendered and cached in RAM
    if let Some(cached) = state.dashboard_handlers.cached_homepage.get() {
        debug!("⚡ [Route] Serving homepage from immediate RAM cache");
        return DashboardHandlers::create_compressed_response(cached.clone());
    }

    // Fallback: Use the dashboard island's homepage handler for lazy init/rendering
    match state.dashboard_handlers.homepage_with_tera(&state) {
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
