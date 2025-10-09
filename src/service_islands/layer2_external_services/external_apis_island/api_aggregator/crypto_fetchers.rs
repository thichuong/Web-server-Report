//! Crypto Price Fetchers Component
//!
//! This module contains all the cryptocurrency price fetching methods with caching.

use anyhow::Result;
use super::aggregator_core::ApiAggregator;

impl ApiAggregator {
    /// Fetch BTC data with generic caching strategy
    pub async fn fetch_btc_with_cache(&self) -> Result<serde_json::Value> {
        let cache_key = "btc_price_30s";

        // Try cache first
        if let Some(ref cache) = self.cache_system {
            if let Ok(Some(cached_data)) = cache.cache_manager.get(cache_key).await {
                return Ok(cached_data);
            }
        }

        // Fetch from API
        match self.market_api.fetch_btc_price().await {
            Ok(data) => {
                // Cache using generic RealTime strategy (30 seconds)
                if let Some(ref cache) = self.cache_system {
                    let _ = cache.cache_manager.set_with_strategy(cache_key, data.clone(),
                        crate::service_islands::layer1_infrastructure::cache_system_island::cache_manager::CacheStrategy::RealTime).await;
                    println!("💾 BTC price cached for 30 seconds (real-time strategy)");
                }
                Ok(data)
            }
            Err(e) => Err(e)
        }
    }

    /// Fetch ETH data with generic caching strategy
    pub async fn fetch_eth_with_cache(&self) -> Result<serde_json::Value> {
        let cache_key = "eth_price_30s";

        // Try cache first
        if let Some(ref cache) = self.cache_system {
            if let Ok(Some(cached_data)) = cache.cache_manager.get(cache_key).await {
                return Ok(cached_data);
            }
        }

        // Fetch from API
        match self.market_api.fetch_eth_price().await {
            Ok(data) => {
                // Cache using generic RealTime strategy (30 seconds)
                if let Some(ref cache) = self.cache_system {
                    let _ = cache.cache_manager.set_with_strategy(cache_key, data.clone(),
                        crate::service_islands::layer1_infrastructure::cache_system_island::cache_manager::CacheStrategy::RealTime).await;
                    println!("💾 ETH price cached for 30 seconds (real-time strategy)");
                }
                Ok(data)
            }
            Err(e) => Err(e)
        }
    }

    /// Fetch SOL data with generic caching strategy
    pub async fn fetch_sol_with_cache(&self) -> Result<serde_json::Value> {
        let cache_key = "sol_price_30s";

        // Try cache first
        if let Some(ref cache) = self.cache_system {
            if let Ok(Some(cached_data)) = cache.cache_manager.get(cache_key).await {
                return Ok(cached_data);
            }
        }

        // Fetch from API
        match self.market_api.fetch_sol_price().await {
            Ok(data) => {
                // Cache using generic RealTime strategy (30 seconds)
                if let Some(ref cache) = self.cache_system {
                    let _ = cache.cache_manager.set_with_strategy(cache_key, data.clone(),
                        crate::service_islands::layer1_infrastructure::cache_system_island::cache_manager::CacheStrategy::RealTime).await;
                    println!("💾 SOL price cached for 30 seconds (real-time strategy)");
                }
                Ok(data)
            }
            Err(e) => Err(e)
        }
    }

    /// Fetch XRP data with generic caching strategy
    pub async fn fetch_xrp_with_cache(&self) -> Result<serde_json::Value> {
        let cache_key = "xrp_price_30s";

        // Try cache first
        if let Some(ref cache) = self.cache_system {
            if let Ok(Some(cached_data)) = cache.cache_manager.get(cache_key).await {
                return Ok(cached_data);
            }
        }

        // Fetch from API
        match self.market_api.fetch_xrp_price().await {
            Ok(data) => {
                // Cache using generic RealTime strategy (30 seconds)
                if let Some(ref cache) = self.cache_system {
                    let _ = cache.cache_manager.set_with_strategy(cache_key, data.clone(),
                        crate::service_islands::layer1_infrastructure::cache_system_island::cache_manager::CacheStrategy::RealTime).await;
                    println!("💾 XRP price cached for 30 seconds (real-time strategy)");
                }
                Ok(data)
            }
            Err(e) => Err(e)
        }
    }

    /// Fetch ADA data with generic caching strategy
    pub async fn fetch_ada_with_cache(&self) -> Result<serde_json::Value> {
        let cache_key = "ada_price_30s";

        // Try cache first
        if let Some(ref cache) = self.cache_system {
            if let Ok(Some(cached_data)) = cache.cache_manager.get(cache_key).await {
                return Ok(cached_data);
            }
        }

        // Fetch from API
        match self.market_api.fetch_ada_price().await {
            Ok(data) => {
                // Cache using generic RealTime strategy (30 seconds)
                if let Some(ref cache) = self.cache_system {
                    let _ = cache.cache_manager.set_with_strategy(cache_key, data.clone(),
                        crate::service_islands::layer1_infrastructure::cache_system_island::cache_manager::CacheStrategy::RealTime).await;
                    println!("💾 ADA price cached for 30 seconds (real-time strategy)");
                }
                Ok(data)
            }
            Err(e) => Err(e)
        }
    }

    /// Fetch LINK data with generic caching strategy
    pub async fn fetch_link_with_cache(&self) -> Result<serde_json::Value> {
        let cache_key = "link_price_30s";

        // Try cache first
        if let Some(ref cache) = self.cache_system {
            if let Ok(Some(cached_data)) = cache.cache_manager.get(cache_key).await {
                return Ok(cached_data);
            }
        }

        // Fetch from API
        match self.market_api.fetch_link_price().await {
            Ok(data) => {
                // Cache using generic RealTime strategy (30 seconds)
                if let Some(ref cache) = self.cache_system {
                    let _ = cache.cache_manager.set_with_strategy(cache_key, data.clone(),
                        crate::service_islands::layer1_infrastructure::cache_system_island::cache_manager::CacheStrategy::RealTime).await;
                    println!("💾 LINK price cached for 30 seconds (real-time strategy)");
                }
                Ok(data)
            }
            Err(e) => Err(e)
        }
    }
}