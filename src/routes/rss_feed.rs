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
use flate2::{Compression, write::GzEncoder};
use std::io::Write;
use std::sync::Arc;
use tracing::{error, info, warn};

use crate::services::data_communication::CryptoDataService;
use crate::services::shared::RssCreator;
use crate::state::AppState;

/// Default number of reports to include in RSS feed
const RSS_FEED_LIMIT: i64 = 20;

/// Configure RSS feed routes
pub fn configure_rss_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/rss.xml", get(rss_feed))
        .route("/rss", get(rss_feed)) // Alternative path without .xml extension
}

/// Build a gzip-compressed RSS XML response with cache headers
fn build_compressed_rss_response(compressed_data: Vec<u8>, cache_status: &str) -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/rss+xml; charset=utf-8")
        .header(header::CACHE_CONTROL, "public, max-age=3600")
        .header(header::CONTENT_ENCODING, "gzip")
        .header("X-Robots-Tag", "index, follow")
        .header("x-cache", cache_status)
        .body(Body::from(compressed_data))
        .unwrap_or_else(|e| {
            error!("Failed to build RSS response: {}", e);
            Response::new(Body::from("Failed to serve RSS feed"))
        })
        .into_response()
}

/// Try to read compressed data from cache (Base64 or legacy `Vec<u8>` format)
async fn try_get_cached_compressed(
    cache_manager: &multi_tier_cache::CacheManager,
    cache_key: &str,
) -> Option<Vec<u8>> {
    let cached_value = cache_manager.get(cache_key).await.ok()??;

    // In v0.6.1, it's already Bytes.
    // To support transition from legacy Base64 JSON format:
    if let Ok(base64_string) = serde_json::from_slice::<String>(&cached_value)
        && let Ok(bytes) = base64::Engine::decode(&base64::prelude::BASE64_STANDARD, base64_string)
    {
        return Some(bytes);
    }

    // Fallback for legacy Vec<u8> JSON format:
    if let Ok(bytes) = serde_json::from_slice::<Vec<u8>>(&cached_value) {
        return Some(bytes);
    }

    // New way: raw bytes
    Some(cached_value.to_vec())
}

/// Compress XML string to gzip format
fn compress_xml(xml: &str) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(xml.as_bytes())?;
    let compressed = encoder.finish()?;
    info!(
        "🗜️ RSS: XML compressed - Original: {}KB, Compressed: {}KB",
        xml.len() / 1024,
        compressed.len() / 1024
    );
    Ok(compressed)
}

/// Cache compressed data with Base64 encoding and `MediumTerm` strategy
async fn cache_compressed_data(
    cache_manager: &multi_tier_cache::CacheManager,
    cache_key: &str,
    compressed_data: &[u8],
    label: &str,
) {
    let bytes = multi_tier_cache::Bytes::from(compressed_data.to_vec());
    match cache_manager
        .set_with_strategy(
            cache_key,
            bytes,
            multi_tier_cache::CacheStrategy::MediumTerm,
        )
        .await
    {
        Err(e) => {
            warn!("⚠️ RSS: Failed to cache {label}: {e}");
        }
        _ => {
            info!("💾 RSS: {label} cached with MediumTerm strategy (1 hour)");
        }
    }
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
        return build_compressed_rss_response(cached_bytes, "HIT");
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

                    match compress_xml(&xml) {
                        Ok(compressed_data) => {
                            cache_compressed_data(
                                cache_manager,
                                cache_key,
                                &compressed_data,
                                "RSS feed",
                            )
                            .await;
                            build_compressed_rss_response(compressed_data, "MISS")
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
