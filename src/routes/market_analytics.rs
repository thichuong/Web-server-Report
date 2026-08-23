//! Market Analytics Route
//!
//! This module handles the market analytics route (`/market_analytics` and `/analytics`).
//! Renders client-side calculation & charting dashboard for Spot vs Futures Volume & Open Interest.

use axum::{Router, extract::State, routing::get};
use flate2::{Compression, write::GzEncoder};
use multi_tier_cache::CacheStrategy;
use std::io::Write;
use std::sync::Arc;
use tera::Context;
use tracing::debug;

use crate::services::crypto_reports::handlers::RenderedContent;
use crate::services::shared::{
    cache_compressed_data,
    error::{Layer5Error, Layer5Result},
    try_get_cached_compressed,
};
use crate::state::AppState;

/// Configure market analytics routes
pub fn configure_market_analytics_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/market_analytics", get(market_analytics_page))
        .route("/analytics", get(market_analytics_page))
}

async fn market_analytics_page(
    State(state): State<Arc<AppState>>,
) -> Layer5Result<RenderedContent> {
    let cache_key = "dashboard_market_analytics_compressed";

    // ⚡ IMMEDIATE CACHE CHECK: Global multi-tier cache check (L1 -> L2)
    if let Some(cached_data) = try_get_cached_compressed(&state.cache_manager, cache_key).await {
        debug!("⚡ [Route] Immediate cache HIT for market analytics");
        return Ok(RenderedContent {
            data: cached_data,
            cache_control: "public, max-age=300",
            cache_status: "HIT",
        });
    }

    // Render template using Tera
    let context = Context::new();
    let rendered_html = state
        .tera
        .render("crypto/routes/analytics/market_analytics.html", &context)
        .map_err(|e| Layer5Error::TemplateRender(format!("Failed to render market analytics: {e}")))?;

    // Compress HTML with Gzip
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(rendered_html.as_bytes())
        .map_err(|e| Layer5Error::Compression(format!("Gzip write error: {e}")))?;
    let compressed_data = encoder
        .finish()
        .map_err(|e| Layer5Error::Compression(format!("Gzip finish error: {e}")))?;

    // Cache the compressed result
    cache_compressed_data(
        &state.cache_manager,
        cache_key,
        &compressed_data,
        CacheStrategy::Default,
        "Market Analytics",
    )
    .await;

    Ok(RenderedContent {
        data: compressed_data,
        cache_control: "public, max-age=300",
        cache_status: "MISS",
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tera::Tera;

    #[test]
    fn test_market_analytics_template_compiles() {
        let mut tera = Tera::default();
        let template_path = "frontend/pages/market_analytics/market_analytics.html";
        let template_name = "crypto/routes/analytics/market_analytics.html";

        // Load and register required components for include
        let _ = tera.add_template_file(
            "frontend/shared/components/theme_toggle.html",
            Some("crypto/components/theme_toggle.html"),
        );
        let _ = tera.add_template_file(
            "frontend/shared/components/language_toggle.html",
            Some("crypto/components/language_toggle.html"),
        );

        let add_res = tera.add_template_file(template_path, Some(template_name));
        assert!(add_res.is_ok(), "Failed to add template: {:?}", add_res.err());

        let context = Context::new();
        let render_res = tera.render(template_name, &context);
        assert!(render_res.is_ok(), "Failed to render template: {:?}", render_res.err());
    }
}
