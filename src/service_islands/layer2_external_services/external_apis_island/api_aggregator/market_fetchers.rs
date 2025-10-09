//! Market Data Fetchers Component
//!
//! This module contains all the market data fetching methods with caching
//! for global market data, Fear & Greed Index, RSI, and US stock indices.

use anyhow::Result;
use super::aggregator_core::ApiAggregator;

impl ApiAggregator {
    /// Fetch global data with generic caching strategy
    pub async fn fetch_global_with_cache(&self) -> Result<serde_json::Value> {
        let cache_key = "global_coingecko_1h";

        // Try cache first
        if let Some(ref cache) = self.cache_system {
            if let Ok(Some(cached_data)) = cache.cache_manager.get(cache_key).await {
                return Ok(cached_data);
            }
        }

        // Fetch from API
        match self.market_api.fetch_global_data().await {
            Ok(data) => {
                // Cache using generic MediumTerm strategy (1 hour)
                if let Some(ref cache) = self.cache_system {
                    let _ = cache.cache_manager.set_with_strategy(cache_key, data.clone(),
                        crate::service_islands::layer1_infrastructure::cache_system_island::cache_manager::CacheStrategy::MediumTerm).await;
                    println!("💾 Global data cached for 1 hour (MediumTerm strategy)");
                }
                Ok(data)
            }
            Err(e) => Err(e)
        }
    }

    /// Fetch Fear & Greed with generic caching strategy
    pub async fn fetch_fng_with_cache(&self) -> Result<serde_json::Value> {
        let cache_key = "fng_alternative_5m";

        // Try cache first
        if let Some(ref cache) = self.cache_system {
            if let Ok(Some(cached_data)) = cache.cache_manager.get(cache_key).await {
                return Ok(cached_data);
            }
        }

        // Fetch from API
        match self.market_api.fetch_fear_greed_index().await {
            Ok(data) => {
                // Cache using generic ShortTerm strategy (5 minutes)
                if let Some(ref cache) = self.cache_system {
                    let _ = cache.cache_manager.set_with_strategy(cache_key, data.clone(),
                        crate::service_islands::layer1_infrastructure::cache_system_island::cache_manager::CacheStrategy::ShortTerm).await;
                    println!("💾 Fear & Greed cached for 5 minutes (short-term strategy)");
                }
                Ok(data)
            }
            Err(e) => Err(e)
        }
    }

    /// Fetch RSI with generic caching strategy
    pub async fn fetch_rsi_with_cache(&self) -> Result<serde_json::Value> {
        let cache_key = "rsi_taapi_3h";

        // Try cache first
        if let Some(ref cache) = self.cache_system {
            if let Ok(Some(cached_data)) = cache.cache_manager.get(cache_key).await {
                return Ok(cached_data);
            }
        }

        // Fetch from API
        match self.market_api.fetch_rsi().await {
            Ok(data) => {
                // Cache using generic LongTerm strategy (3 hours)
                if let Some(ref cache) = self.cache_system {
                    let cache_manager = cache.get_cache_manager();
                    let _ = cache_manager.set_with_strategy(cache_key, data.clone(),
                        crate::service_islands::layer1_infrastructure::cache_system_island::cache_manager::CacheStrategy::LongTerm).await;
                    println!("💾 RSI cached for 3 hours (long-term strategy)");
                }
                Ok(data)
            }
            Err(e) => Err(e)
        }
    }

    /// Fetch US Stock Indices with generic caching strategy
    pub async fn fetch_us_indices_with_cache(&self) -> Result<serde_json::Value> {
        let cache_key = "us_indices_finnhub_5m";

        // Try cache first
        if let Some(ref cache) = self.cache_system {
            if let Ok(Some(cached_data)) = cache.cache_manager.get(cache_key).await {
                return Ok(cached_data);
            }
        }

        // Fetch from API
        match self.market_api.fetch_us_stock_indices().await {
            Ok(data) => {
                // Cache using generic ShortTerm strategy (5 minutes)
                if let Some(ref cache) = self.cache_system {
                    let _ = cache.cache_manager.set_with_strategy(cache_key, data.clone(),
                        crate::service_islands::layer1_infrastructure::cache_system_island::cache_manager::CacheStrategy::ShortTerm).await;
                    println!("💾 US Stock Indices cached for 5 minutes (short-term strategy)");
                }
                Ok(data)
            }
            Err(e) => Err(e)
        }
    }
}