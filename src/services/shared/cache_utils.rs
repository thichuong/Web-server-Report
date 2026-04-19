//! Cache Utilities for Layer 5
//!
//! Standardized utilities for caching compressed binary data (Vec<u8>)
//! with support for legacy Base64/JSON formats and new Raw Bytes.

use axum::{
    body::Body,
    http::{Response, StatusCode, header},
    response::IntoResponse,
};
use base64::{Engine, prelude::BASE64_STANDARD};
use flate2::{Compression, write::GzEncoder};
use multi_tier_cache::{Bytes, CacheManager, CacheStrategy};
use std::io::Write;
use tracing::{debug, info, warn};

/// Compress XML/JSON string to gzip format
///
/// # Errors
///
/// Returns error if compression fails
pub fn compress_data(data: &str) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data.as_bytes())?;
    let compressed = encoder.finish()?;
    Ok(compressed)
}

/// Try to read compressed data from cache
/// Supports legacy Base64 JSON format, legacy Vec<u8> JSON format, and new Raw Bytes.
pub async fn try_get_cached_compressed(
    cache_manager: &CacheManager,
    cache_key: &str,
) -> Option<Vec<u8>> {
    let cached_value = cache_manager.get(cache_key).await.ok()??;

    // To support transition from legacy Base64 JSON format:
    if let Ok(base64_string) = serde_json::from_slice::<String>(&cached_value)
        && let Ok(bytes) = BASE64_STANDARD.decode(base64_string)
    {
        debug!("🔥 Cache: HIT (Base64 legacy) for {}", cache_key);
        return Some(bytes);
    }

    // Fallback for legacy Vec<u8> JSON format:
    if let Ok(bytes) = serde_json::from_slice::<Vec<u8>>(&cached_value) {
        debug!("🔥 Cache: HIT (JSON legacy) for {}", cache_key);
        return Some(bytes);
    }

    // New way: raw bytes
    debug!("🔥 Cache: HIT (Raw Bytes) for {}", cache_key);
    Some(cached_value.to_vec())
}

/// Cache compressed data using the specified strategy
pub async fn cache_compressed_data(
    cache_manager: &CacheManager,
    cache_key: &str,
    compressed_data: &[u8],
    strategy: CacheStrategy,
    label: &str,
) {
    let bytes = Bytes::from(compressed_data.to_vec());
    match cache_manager
        .set_with_strategy(cache_key, bytes, strategy)
        .await
    {
        Err(e) => {
            warn!("⚠️ Cache: Failed to cache {label} at {cache_key}: {e}");
        }
        _ => {
            info!("💾 Cache: {label} cached at {cache_key} successfully");
        }
    }
}

/// Build a gzip-compressed response with standard headers
pub fn build_standard_compressed_response(
    compressed_data: Vec<u8>,
    content_type: &'static str,
    max_age: u32,
    cache_status: &str,
) -> Response<Body> {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CACHE_CONTROL, format!("public, max-age={max_age}"))
        .header(header::CONTENT_ENCODING, "gzip")
        .header("x-cache", cache_status)
        .body(Body::from(compressed_data))
        .unwrap_or_else(|e| {
            warn!("⚠️ Failed to build compressed response: {}", e);
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from("Response build error"))
                .unwrap_or_else(|_| Response::new(Body::from("Response build error")))
        })
        .into_response()
}
