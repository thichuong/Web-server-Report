//! Homepage Route
//!
//! This module handles the main homepage route using the Service Islands Architecture.
//! The homepage is served through the Dashboard Island.

use axum::{Router, extract::State, routing::get};
use std::sync::Arc;
use tracing::debug;

use crate::services::crypto_reports::handlers::RenderedContent;
use crate::services::shared::{error::Layer5Result, try_get_cached_compressed};
use crate::state::AppState;

/// Configure homepage route
pub fn configure_homepage_route() -> Router<Arc<AppState>> {
    Router::new().route("/", get(homepage))
}

async fn homepage(State(state): State<Arc<AppState>>) -> Layer5Result<RenderedContent> {
    // ⚡ IMMEDIATE CACHE CHECK: Global multi-tier cache check (L1 -> L2)
    let cache_key = "dashboard_homepage_compressed";
    if let Some(cached_data) = try_get_cached_compressed(&state.cache_manager, cache_key).await {
        debug!("⚡ [Route] Immediate cache HIT for homepage");
        return Ok(RenderedContent {
            data: cached_data,
            cache_control: "public, max-age=300",
            cache_status: "HIT",
        });
    }

    // Fallback: Use the dashboard island's homepage handler for lazy init/rendering
    state.dashboard_handlers.homepage_with_tera(&state).await
}
