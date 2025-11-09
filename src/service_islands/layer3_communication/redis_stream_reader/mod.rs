//! Redis Stream Reader - Layer 3 Communication
//!
//! This module provides functionality to read market data from Redis Streams.
//! Replaces direct Layer 2 API calls with stream consumption for the main service.

use anyhow::Result;
use serde_json::Value;
use std::sync::Arc;
use crate::service_islands::layer1_infrastructure::cache_system_island::CacheSystemIsland;

/// Redis Stream Reader
///
/// Reads market data from Redis Streams published by the websocket service.
pub struct RedisStreamReader {
    cache_system: Arc<CacheSystemIsland>,
    stream_key: String,
}

impl RedisStreamReader {
    /// Create a new Redis Stream Reader
    pub fn new(cache_system: Arc<CacheSystemIsland>) -> Self {
        Self {
            cache_system,
            stream_key: "market_data_stream".to_string(),
        }
    }

    /// Read the latest market data from Redis Stream
    ///
    /// Returns the most recent entry from the stream.
    /// Falls back to cache if stream is empty.
    pub async fn read_latest_market_data(&self) -> Result<Option<Value>> {
        println!("📖 Reading latest market data from Redis Stream...");

        // Try to read from Redis Stream first
        match self.read_from_stream().await {
            Ok(Some(data)) => {
                println!("✅ Market data read from Redis Stream");
                return Ok(Some(data));
            }
            Ok(None) => {
                println!("⚠️ No data in Redis Stream, trying cache...");
            }
            Err(e) => {
                println!("❌ Failed to read from Redis Stream: {}, trying cache...", e);
            }
        }

        // Fallback to reading from cache
        match self.cache_system.cache_manager().get("latest_market_data").await {
            Ok(Some(data)) => {
                println!("✅ Market data read from cache (fallback)");
                Ok(Some(data))
            }
            Ok(None) => {
                println!("⚠️ No data in cache either");
                Ok(None)
            }
            Err(e) => {
                println!("❌ Failed to read from cache: {}", e);
                Err(anyhow::anyhow!("Failed to read market data: {}", e))
            }
        }
    }

    /// Read from Redis Stream
    ///
    /// Reads the latest entry from the market_data_stream.
    async fn read_from_stream(&self) -> Result<Option<Value>> {
        // Use cache_manager's stream reading functionality
        let entries = self.cache_system
            .cache_manager()
            .read_stream_latest(&self.stream_key, 1)
            .await?;

        if entries.is_empty() {
            return Ok(None);
        }

        // Get the first (and only) entry
        let (entry_id, fields) = &entries[0];
        println!("📨 Stream entry ID: {}", entry_id);

        // Convert stream fields back to JSON
        let json_data = self.stream_fields_to_json(fields)?;

        Ok(Some(json_data))
    }

    /// Convert Redis Stream fields to JSON
    ///
    /// Transforms the flat key-value pairs from Redis Streams back into JSON.
    fn stream_fields_to_json(&self, fields: &Vec<(String, String)>) -> Result<Value> {
        let mut map = serde_json::Map::new();

        for (key, value) in fields {
            // Try to parse value as different types
            let json_value = if value == "null" {
                Value::Null
            } else if value == "true" {
                Value::Bool(true)
            } else if value == "false" {
                Value::Bool(false)
            } else if let Ok(num) = value.parse::<f64>() {
                Value::Number(serde_json::Number::from_f64(num).unwrap_or_else(|| {
                    serde_json::Number::from(0)
                }))
            } else if let Ok(int) = value.parse::<i64>() {
                Value::Number(serde_json::Number::from(int))
            } else if value.starts_with('{') || value.starts_with('[') {
                // Try to parse as JSON object or array
                serde_json::from_str(value).unwrap_or_else(|_| Value::String(value.clone()))
            } else {
                Value::String(value.clone())
            };

            map.insert(key.clone(), json_value);
        }

        Ok(Value::Object(map))
    }

    /// Health check
    pub async fn health_check(&self) -> Result<bool> {
        // Try to connect to Redis via cache_system
        match self.cache_system.cache_manager().get("_health_check").await {
            Ok(_) => Ok(true),
            Err(e) => {
                println!("❌ Redis Stream Reader health check failed: {}", e);
                Ok(false)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_fields_to_json() {
        let reader = RedisStreamReader {
            cache_system: Arc::new(unsafe { std::mem::zeroed() }), // Mock for test
            stream_key: "test_stream".to_string(),
        };

        let fields = vec![
            ("btc_price_usd".to_string(), "45000.5".to_string()),
            ("btc_change_24h".to_string(), "2.5".to_string()),
            ("fng_value".to_string(), "75".to_string()),
            ("partial_failure".to_string(), "false".to_string()),
        ];

        let result = reader.stream_fields_to_json(&fields).unwrap();

        assert_eq!(result["btc_price_usd"], 45000.5);
        assert_eq!(result["btc_change_24h"], 2.5);
        assert_eq!(result["fng_value"], 75);
        assert_eq!(result["partial_failure"], false);
    }
}
