//! Redis Stream Reader - Layer 3 Communication
//!
//! This module provides functionality to read market data from Redis Streams.
//! Replaces direct Layer 2 API calls with stream consumption for the main service.

use crate::service_islands::layer1_infrastructure::cache_system_island::CacheSystemIsland;
use anyhow::Result;
use serde_json::Value;
use std::sync::Arc;
use tracing::info;

/// Redis Stream Reader
///
/// Reads market data from Redis Streams published by the websocket service.
pub struct RedisStreamReader {
    cache_system: Arc<CacheSystemIsland>,
    stream_key: String,
}

impl RedisStreamReader {
    /// Create a new Redis Stream Reader
    #[must_use] 
    pub fn new(cache_system: Arc<CacheSystemIsland>) -> Self {
        Self {
            cache_system,
            stream_key: "market_data_stream".to_string(),
        }
    }

    /// Read the latest market data using cache-first strategy with automatic fallback
    ///
    /// Uses `get_or_compute_typed` for automatic cache management:
    /// - Cache hit: Returns immediately (L1 <1ms, L2 2-5ms)
    /// - Cache miss: Reads from Redis Stream, caches result, then returns
    /// - Built-in cache stampede protection prevents multiple concurrent stream reads
    pub async fn read_latest_market_data(&self) -> Result<Option<Value>> {
        info!("📖 Reading latest market data (cache-first with auto-fallback)...");

        use crate::service_islands::layer1_infrastructure::cache_system_island::CacheStrategy;

        // ✅ IDIOMATIC: Use get_or_compute_typed for automatic cache management
        // This provides cache-first behavior + stampede protection + automatic caching
        self.cache_system
            .cache_manager()
            .get_or_compute_typed(
                "latest_market_data",
                CacheStrategy::RealTime, // 5 minutes TTL
                || async {
                    // Compute function: only called on cache miss
                    info!("💾 Cache miss - reading from Redis Stream...");

                    match self.read_from_stream().await {
                        Ok(Some(data)) => {
                            info!("✅ Market data read from Redis Stream (will be cached)");
                            Ok(Some(data))
                        }
                        Ok(None) => {
                            info!("⚠️ No data in Redis Stream");
                            Ok(None)
                        }
                        Err(e) => {
                            info!("❌ Failed to read from Redis Stream: {}", e);
                            Err(anyhow::anyhow!("Failed to read market data: {}", e))
                        }
                    }
                },
            )
            .await
    }

    /// Read from Redis Stream
    ///
    /// Reads the latest entry from the `market_data_stream`.
    async fn read_from_stream(&self) -> Result<Option<Value>> {
        // Use cache_manager's stream reading functionality
        let entries = self
            .cache_system
            .cache_manager()
            .read_stream_latest(&self.stream_key, 1)
            .await?;

        if entries.is_empty() {
            return Ok(None);
        }

        // Get the first (and only) entry
        let (entry_id, fields) = entries.first().ok_or_else(|| anyhow::anyhow!("Stream entry missing"))?;
        info!("📨 Stream entry ID: {}", entry_id);

        // Convert stream fields back to JSON
        let json_data = Self::stream_fields_to_json(fields);

        Ok(Some(json_data))
    }

    /// Convert Redis Stream fields to JSON
    ///
    /// Transforms the flat key-value pairs from Redis Streams back into JSON.
    /// Special handling: If there's a single "data" field containing JSON, unwrap it.
    fn stream_fields_to_json(fields: &Vec<(String, String)>) -> Value {
        // Special case: If there's only one field named "data" containing JSON string
        if fields.len() == 1 {
            if let Some((key, value)) = fields.first() {
                if key == "data" {
                    if let Ok(data) = serde_json::from_str::<Value>(value) {
                        info!("📦 Unwrapped nested 'data' field from Redis Stream");
                        return data;
                    }
                }
            }
        }

        // General case: parse each field
        let mut map = serde_json::Map::new();

        for (key, value) in fields {
            // Try to parse value as different types
            let json_value = if value == "null" {
                Value::Null
            } else if value == "true" {
                Value::Bool(true)
            } else if value == "false" {
                Value::Bool(false)
            } else if let Ok(int) = value.parse::<i64>() {
                Value::Number(serde_json::Number::from(int))
            } else if let Ok(num) = value.parse::<f64>() {
                Value::Number(
                    serde_json::Number::from_f64(num)
                        .unwrap_or_else(|| serde_json::Number::from(0)),
                )
            } else if value.starts_with('{') || value.starts_with('[') {
                // Try to parse as JSON object or array
                serde_json::from_str(value).unwrap_or_else(|_| Value::String(value.clone()))
            } else {
                Value::String(value.clone())
            };

            map.insert(key.clone(), json_value);
        }

        Value::Object(map)
    }

    /// Health check
    ///
    /// # Errors
    ///
    /// Returns error if Redis connection fails (though currently it returns Ok(false) on error)
    pub async fn health_check(&self) -> Result<bool> {
        // Try to connect to Redis via cache_system
        match self.cache_system.cache_manager().get("_health_check").await {
            Ok(_) => Ok(true),
            Err(e) => {
                info!("❌ Redis Stream Reader health check failed: {}", e);
                Ok(false)
            }
        }
    }

    /// Cleanup resources on shutdown
    ///
    /// ✅ PRODUCTION-READY: Graceful cleanup for Redis Stream Reader
    /// Note: Redis connections are managed by the `cache_system` library and will
    /// be automatically cleaned up when `cache_system` is dropped.
    ///
    /// # Errors
    ///
    /// Returns error if cleanup fails
    pub async fn cleanup(&self) -> Result<()> {
        info!("🧹 RedisStreamReader: Starting cleanup...");

        // Note: If using Redis consumer groups, we would acknowledge pending messages here
        // Currently using simple XREAD without consumer groups, so no pending messages to handle

        // Redis connections managed by cache_system - no manual cleanup needed
        info!("   ✅ Redis connections managed by cache_system library");

        info!("✅ RedisStreamReader: Cleanup complete");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_fields_to_json() {
        let fields = vec![
            ("btc_price_usd".to_string(), "45000.5".to_string()),
            ("btc_change_24h".to_string(), "2.5".to_string()),
            ("fng_value".to_string(), "75".to_string()),
            ("partial_failure".to_string(), "false".to_string()),
        ];

        let result = RedisStreamReader::stream_fields_to_json(&fields);

        assert_eq!(result["btc_price_usd"], 45000.5);
        assert_eq!(result["btc_change_24h"], 2.5);
        assert_eq!(result["fng_value"], 75);
        assert_eq!(result["partial_failure"], false);
    }
}
