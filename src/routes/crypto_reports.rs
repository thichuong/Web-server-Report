//! Crypto Reports Routes
//!
//! This module defines all HTTP routes for crypto reports functionality including
//! report viewing and listing.

use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    routing::get,
    Router,
};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::debug;

use crate::service_islands::layer5_business_logic::{
    crypto_reports::handlers::RenderedContent, shared::error::Layer5Result,
};
use crate::service_islands::ServiceIslands;

/// Configure crypto reports routes
pub fn configure_crypto_reports_routes() -> Router<Arc<ServiceIslands>> {
    Router::new()
        .route("/crypto_report", get(crypto_index))
        .route("/crypto_report/:id", get(crypto_view_report))
        .route("/crypto_reports_list", get(crypto_reports_list))
}

/// List all crypto reports with pagination
async fn crypto_reports_list(
    Query(params): Query<HashMap<String, String>>,
    State(service_islands): State<Arc<ServiceIslands>>,
) -> Layer5Result<RenderedContent> {
    debug!("🚀 [Route] crypto_reports_list called - fetching from Service Islands Layer 5");

    // Parse pagination parameter
    let page: i64 = params.get("page").and_then(|p| p.parse().ok()).unwrap_or(1);
    debug!("📄 [Route] Requesting page: {}", page);

    // Use Service Islands architecture to get reports list (compressed)
    service_islands
        .crypto_reports
        .handlers
        .crypto_reports_list_with_tera(&service_islands.get_legacy_app_state(), page)
        .await
}

/// Crypto reports index page using Declarative Shadow DOM
/// Modern primary route for crypto reports
/// ✅ OPTIMIZED: Full caching support with language-specific cache keys
async fn crypto_index(
    State(service_islands): State<Arc<ServiceIslands>>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
) -> Layer5Result<RenderedContent> {
    debug!("🌓 [Route] crypto_index called - delegating to Service Islands Layer 5");

    // Check if specific report ID is requested via query param
    let report_id = params.get("id");
    let report_id_value = if let Some(id_str) = report_id {
        if let Ok(id) = id_str.parse::<i32>() {
            Some(id)
        } else {
            return Err(crate::service_islands::layer5_business_logic::shared::error::Layer5Error::InvalidInput(format!("Invalid report ID format: {id_str}")));
        }
    } else {
        None // Latest report
    };

    // Get chart modules content
    let chart_modules_content = service_islands.get_chart_modules_content();

    // Delegate to handlers

    service_islands
        .crypto_reports
        .handlers
        .render_crypto_index_dsd(
            &service_islands.get_legacy_app_state(),
            &params,
            &headers,
            chart_modules_content,
            report_id_value,
        )
        .await
}

/// View specific crypto report by ID using Declarative Shadow DOM
/// Modern primary route for viewing specific reports
/// ✅ OPTIMIZED: Full caching support with language-specific cache keys
async fn crypto_view_report(
    Path(id): Path<String>,
    State(service_islands): State<Arc<ServiceIslands>>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
) -> Layer5Result<RenderedContent> {
    debug!("🌓 [Route] crypto_view_report called for ID: {}", id);

    // Parse report ID
    let report_id: i32 = id.parse().map_err(|_| {
        crate::service_islands::layer5_business_logic::shared::error::Layer5Error::InvalidInput(
            format!("Invalid report ID format: {id}"),
        )
    })?;

    // Get chart modules content
    let chart_modules_content = service_islands.get_chart_modules_content();

    // Delegate to handlers
    service_islands
        .crypto_reports
        .handlers
        .render_crypto_report_dsd(
            &service_islands.get_legacy_app_state(),
            report_id,
            &params,
            &headers,
            chart_modules_content,
        )
        .await
}
