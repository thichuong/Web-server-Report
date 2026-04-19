//! SEO Routes Module
//!
//! Handles SEO-related endpoints including:
//! - sitemap.xml generation
//!
//! These routes are designed for search engine optimization and follow
//! the Service Islands architecture (Layer 5 -> Layer 3 -> Layer 1).
//! ✅ OPTIMIZED: Server-side L1/L2 cache with `MediumTerm` strategy (1 hour)

use axum::{
    Router,
    body::Body,
    extract::State,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use std::sync::Arc;
use tracing::{error, info};

use crate::services::data_communication::CryptoDataService;
use crate::services::shared::{
    SitemapCreator, build_standard_compressed_response, cache_compressed_data, compress_data,
    try_get_cached_compressed,
};
use crate::state::AppState;

/// Configure SEO routes
pub fn configure_seo_routes() -> Router<Arc<AppState>> {
    Router::new().route("/sitemap.xml", get(sitemap_xml))
}

/// Generate and serve sitemap.xml with L1/L2 cache
///
/// Uses `MediumTerm` cache strategy (1 hour) since sitemap changes infrequently.
async fn sitemap_xml(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    info!("Generating sitemap.xml");

    let cache_key = "sitemap_xml_compressed";
    let cache_manager = &state.cache_manager;

    // Step 1: Check L1/L2 cache first
    if let Some(cached_bytes) = try_get_cached_compressed(cache_manager, cache_key).await {
        info!("🔥 SEO: sitemap.xml Cache HIT - serving from cache");
        return build_standard_compressed_response(
            cached_bytes,
            "application/xml; charset=utf-8",
            3600,
            "HIT",
        );
    }

    info!("🔍 SEO: sitemap.xml Cache MISS - generating from database");

    // Step 2: Cache MISS - generate from database
    let data_service = CryptoDataService::new();
    let reports_result = data_service.fetch_all_report_ids_for_sitemap(&state).await;

    match reports_result {
        Ok(reports) => {
            let report_data: Vec<(i32, chrono::DateTime<chrono::Utc>)> =
                reports.into_iter().map(|r| (r.id, r.created_at)).collect();

            match SitemapCreator::generate_sitemap_xml(report_data) {
                Ok(xml) => {
                    info!("Sitemap.xml generated successfully ({} bytes)", xml.len());

                    match compress_data(&xml) {
                        Ok(compressed_data) => {
                            cache_compressed_data(
                                cache_manager,
                                cache_key,
                                &compressed_data,
                                multi_tier_cache::CacheStrategy::MediumTerm,
                                "sitemap.xml",
                            )
                            .await;
                            build_standard_compressed_response(
                                compressed_data,
                                "application/xml; charset=utf-8",
                                3600,
                                "MISS",
                            )
                        }
                        Err(e) => {
                            error!("Failed to compress sitemap XML: {}", e);
                            Response::builder()
                                .status(StatusCode::OK)
                                .header(header::CONTENT_TYPE, "application/xml; charset=utf-8")
                                .header(header::CACHE_CONTROL, "public, max-age=3600")
                                .header("X-Robots-Tag", "noindex")
                                .body(Body::from(xml))
                                .unwrap_or_else(|e| {
                                    error!("Failed to build fallback sitemap response: {}", e);
                                    Response::new(Body::from("Failed to generate sitemap"))
                                })
                                .into_response()
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to generate sitemap XML: {}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Failed to generate sitemap",
                    )
                        .into_response()
                }
            }
        }
        Err(e) => {
            error!("Failed to fetch reports for sitemap: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to fetch sitemap data",
            )
                .into_response()
        }
    }
}
