//! SEO Routes Module
//!
//! Handles SEO-related endpoints including:
//! - sitemap.xml generation
//!
//! These routes are designed for search engine optimization and follow
//! the Service Islands architecture (Layer 5 -> Layer 3 -> Layer 1).
//! ✅ OPTIMIZED: Server-side L1/L2 cache with `MediumTerm` strategy (1 hour)

use axum::{
    body::Body,
    extract::State,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use flate2::{write::GzEncoder, Compression};
use std::io::Write;
use std::sync::Arc;
use tracing::{error, info, warn};

use crate::services::data_communication::CryptoDataService;
use crate::services::shared::SitemapCreator;
use crate::state::AppState;

/// Configure SEO routes
pub fn configure_seo_routes() -> Router<Arc<AppState>> {
    Router::new().route("/sitemap.xml", get(sitemap_xml))
}

/// Build a gzip-compressed XML response with cache headers
fn build_compressed_xml_response(
    compressed_data: Vec<u8>,
    cache_status: &str,
) -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/xml; charset=utf-8")
        .header(header::CACHE_CONTROL, "public, max-age=3600")
        .header(header::CONTENT_ENCODING, "gzip")
        .header("X-Robots-Tag", "noindex")
        .header("x-cache", cache_status)
        .body(Body::from(compressed_data))
        .unwrap_or_else(|e| {
            error!("Failed to build sitemap response: {}", e);
            Response::new(Body::from("Failed to generate sitemap"))
        })
        .into_response()
}

/// Try to read compressed data from cache (Base64 or legacy `Vec<u8>` format)
async fn try_get_cached_compressed(
    cache_manager: &multi_tier_cache::CacheManager,
    cache_key: &str,
) -> Option<Vec<u8>> {
    let cached_value = cache_manager.get(cache_key).await.ok()??;

    // Try Base64 string format first (new, memory-optimized)
    if let Ok(base64_string) = serde_json::from_value::<String>(cached_value.clone())
        && let Ok(bytes) = base64::Engine::decode(&base64::prelude::BASE64_STANDARD, base64_string)
        {
            return Some(bytes);
        }
    // Fallback: try Vec<u8> format (legacy)
    serde_json::from_value::<Vec<u8>>(cached_value).ok()
}

/// Compress XML string to gzip format
fn compress_xml(xml: &str) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(xml.as_bytes())?;
    let compressed = encoder.finish()?;
    info!(
        "🗜️ SEO: XML compressed - Original: {}KB, Compressed: {}KB",
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
    let base64_string =
        base64::Engine::encode(&base64::prelude::BASE64_STANDARD, compressed_data);
    let json_value = serde_json::Value::String(base64_string);
    match cache_manager
        .set_with_strategy(cache_key, json_value, multi_tier_cache::CacheStrategy::MediumTerm)
        .await
    { Err(e) => {
        warn!("⚠️ SEO: Failed to cache {label}: {e}");
    } _ => {
        info!("💾 SEO: {label} cached with MediumTerm strategy (1 hour)");
    }}
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
        return build_compressed_xml_response(cached_bytes, "HIT");
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

                    match compress_xml(&xml) {
                        Ok(compressed_data) => {
                            cache_compressed_data(
                                cache_manager,
                                cache_key,
                                &compressed_data,
                                "sitemap.xml",
                            )
                            .await;
                            build_compressed_xml_response(compressed_data, "MISS")
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
                    (StatusCode::INTERNAL_SERVER_ERROR, "Failed to generate sitemap").into_response()
                }
            }
        }
        Err(e) => {
            error!("Failed to fetch reports for sitemap: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch sitemap data").into_response()
        }
    }
}
