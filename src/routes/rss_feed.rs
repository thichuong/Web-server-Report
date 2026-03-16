//! RSS Feed Routes Module
//!
//! Handles RSS feed endpoint for search engines, AI bots, and feed readers:
//! - /rss.xml - RSS 2.0 feed with latest crypto reports
//!
//! These routes follow the Service Islands architecture (Layer 5 -> Layer 3 -> Layer 1)
//! and are optimized for daily content discovery by bots and crawlers.
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
    RssCreator, build_standard_compressed_response, cache_compressed_data, compress_data,
    try_get_cached_compressed,
};
use crate::state::AppState;

/// Default number of reports to include in RSS feed
const RSS_FEED_LIMIT: i64 = 20;

/// Configure RSS feed routes
pub fn configure_rss_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/rss.xml", get(rss_feed))
        .route("/rss", get(rss_feed)) // Alternative path without .xml extension
}


/// Generate and serve RSS 2.0 feed with L1/L2 cache
///
/// Uses `MediumTerm` cache strategy (1 hour) since RSS changes infrequently.
async fn rss_feed(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    info!("📡 Generating RSS feed");

    let cache_key = "rss_feed_xml_compressed";
    let cache_manager = &state.cache_manager;

    // Step 1: Check L1/L2 cache first
    if let Some(cached_bytes) = try_get_cached_compressed(cache_manager, cache_key).await {
        info!("🔥 RSS: Cache HIT - serving from cache");
        return build_standard_compressed_response(
            cached_bytes,
            "application/rss+xml; charset=utf-8",
            3600,
            "HIT",
        );
    }

    info!("🔍 RSS: Cache MISS - generating from database");

    // Step 2: Cache MISS - generate from database
    let data_service = CryptoDataService::new();
    let reports_result = data_service.fetch_rss_reports(&state, RSS_FEED_LIMIT).await;

    match reports_result {
        Ok(reports) => {
            let report_count = reports.len();

            match RssCreator::generate_rss_xml(&reports) {
                Ok(xml) => {
                    info!(
                        "✅ RSS feed generated: {} items, {} bytes",
                        report_count,
                        xml.len()
                    );

                    match compress_data(&xml) {
                        Ok(compressed_data) => {
                            cache_compressed_data(
                                cache_manager,
                                cache_key,
                                &compressed_data,
                                multi_tier_cache::CacheStrategy::MediumTerm,
                                "RSS feed",
                            )
                            .await;
                            build_standard_compressed_response(
                                compressed_data,
                                "application/rss+xml; charset=utf-8",
                                3600,
                                "MISS",
                            )
                        }
                        Err(e) => {
                            error!("Failed to compress RSS XML: {}", e);
                            Response::builder()
                                .status(StatusCode::OK)
                                .header(header::CONTENT_TYPE, "application/rss+xml; charset=utf-8")
                                .header(header::CACHE_CONTROL, "public, max-age=3600")
                                .header("X-Robots-Tag", "index, follow")
                                .body(Body::from(xml))
                                .unwrap_or_else(|e| {
                                    error!("Failed to build fallback RSS response: {}", e);
                                    Response::new(Body::from("Failed to generate RSS feed"))
                                })
                                .into_response()
                        }
                    }
                }
                Err(e) => {
                    error!("❌ Failed to generate RSS XML: {}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Failed to generate RSS feed",
                    )
                        .into_response()
                }
            }
        }
        Err(e) => {
            error!("❌ Failed to fetch reports for RSS feed: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to fetch RSS data",
            )
                .into_response()
        }
    }
}
