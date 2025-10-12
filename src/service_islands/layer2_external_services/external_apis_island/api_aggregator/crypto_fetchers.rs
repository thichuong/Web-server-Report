//! Crypto Price Fetchers Component
//!
//! This module contains all the cryptocurrency price fetching methods with caching.

use anyhow::Result;
use std::collections::HashMap;
use super::aggregator_core::ApiAggregator;

impl ApiAggregator {
    /// Fetch all crypto prices with a single API call (OPTIMIZED)
    /// 
    /// Returns HashMap with coin symbols as keys: BTC, ETH, SOL, XRP, ADA, LINK, BNB
    /// Each value is a JSON object with price_usd and change_24h
    /// 
    /// force_refresh: If true, skips cache check and forces API fetch (for streaming)
    pub async fn fetch_all_crypto_prices_with_cache(&self, force_refresh: bool) -> Result<HashMap<String, serde_json::Value>> {
        let cache_key = "multi_crypto_prices_realtime";

        // Try cache first (unless force refresh)
        if !force_refresh {
            if let Some(ref cache) = self.cache_system {
                if let Ok(Some(cached_data)) = cache.cache_manager.get(cache_key).await {
                    // Deserialize from JSON to HashMap
                    if let Ok(prices) = serde_json::from_value::<HashMap<String, serde_json::Value>>(cached_data) {
                        println!("💾 Retrieved all crypto prices from cache");
                        return Ok(prices);
                    }
                }
            }
        }

        // Fetch from API
        match self.market_api.fetch_multi_crypto_prices().await {
            Ok(raw_data) => {
                // Convert HashMap<String, (f64, f64)> to HashMap<String, serde_json::Value>
                let mut result = HashMap::new();
                for (coin, (price_usd, change_24h)) in raw_data {
                    result.insert(coin.clone(), serde_json::json!({
                        "price_usd": price_usd,
                        "change_24h": change_24h,
                        "source": "binance",
                        "last_updated": chrono::Utc::now().to_rfc3339()
                    }));
                }

                // Cache using RealTime strategy - 10s TTL
                if let Some(ref cache) = self.cache_system {
                    let cache_value = serde_json::to_value(&result).unwrap_or(serde_json::json!({}));
                    let _ = cache.cache_manager.set_with_strategy(cache_key, cache_value,
                        crate::service_islands::layer1_infrastructure::cache_system_island::cache_manager::CacheStrategy::RealTime).await;
                    println!("💾 All crypto prices cached (RealTime strategy - 10s TTL)");
                }
                Ok(result)
            }
            Err(e) => Err(e)
        }
    }
}
