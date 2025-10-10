//! Market Data Adapter - Layer 3 to Layer 2 Market Data Bridge (Cache-Optimized)
//! 
//! This adapter handles all market data fetching operations from Layer 2.
//! It provides a clean abstraction for Layer 3 components to access
//! Layer 2 External APIs Island market data services.
//! 
//! OPTIMIZATION: Layer 3 cache check for latest_market_data to minimize Layer 2 calls.

use anyhow::Result;
use serde_json;
use std::sync::Arc;

use crate::service_islands::layer2_external_services::external_apis_island::ExternalApisIsland;
use crate::service_islands::layer1_infrastructure::cache_system_island::{CacheSystemIsland, CacheStrategy};

/// Market Data Adapter (Cache-Optimized)
/// 
/// Handles all Layer 3 → Layer 2 market data communication.
/// Provides methods for fetching various types of market data
/// while maintaining proper Service Islands Architecture.
/// 
/// OPTIMIZATION: Performs Layer 3 cache checks for latest_market_data
/// to minimize unnecessary Layer 2 API calls.
pub struct MarketDataAdapter {
    /// Reference to Layer 2 External APIs Island
    external_apis: Option<Arc<ExternalApisIsland>>,
    /// Reference to Layer 1 Cache System for direct cache access
    cache_system: Option<Arc<CacheSystemIsland>>,
}

#[allow(dead_code)]
impl MarketDataAdapter {
    /// Create new Market Data Adapter without dependencies
    pub fn new() -> Self {
        Self {
            external_apis: None,
            cache_system: None,
        }
    }
    
    /// Set Layer 2 External APIs dependency
    pub fn with_external_apis(mut self, external_apis: Arc<ExternalApisIsland>) -> Self {
        self.external_apis = Some(external_apis);
        self
    }
    
    /// Set Layer 1 Cache System dependency for direct cache access
    pub fn with_cache_system(mut self, cache_system: Arc<CacheSystemIsland>) -> Self {
        self.cache_system = Some(cache_system);
        self
    }
    
    /// Fetch dashboard summary data from Layer 2
    /// 
    /// Main method for getting comprehensive market data for dashboards.
    /// Used by Layer 5 business logic via Layer 3.
    pub async fn fetch_dashboard_summary(&self) -> Result<serde_json::Value> {
        if let Some(external_apis) = &self.external_apis {
            println!("🔄 [Layer 3 → Layer 2] Fetching dashboard summary...");
            external_apis.fetch_dashboard_summary_v2().await
        } else {
            Err(anyhow::anyhow!("Layer 2 External APIs not configured in MarketDataAdapter"))
        }
    }
    
    // DEPRECATED: Use fetch_normalized_market_data() instead for all market data
    // Individual methods removed to prevent redundant API calls
    
    // DEPRECATED: Use cache-free v2 methods instead for better separation of concerns
    // Layer 1 handles all caching, Layer 2 focuses on pure API business logic
    
    /// Normalize market data for Layer 5 consumption (OPTIMIZED)
    /// 
    /// This is the PRIMARY method for getting market data.
    /// Uses cache-free Layer 2 v2 methods to avoid redundant API calls.
    /// Converts raw Layer 2 data into a format suitable for Layer 5 business logic.
    pub async fn fetch_normalized_market_data(&self) -> Result<serde_json::Value> {
        // Use cache-free v2 method to avoid redundant cache logic
        let raw_data = self.fetch_dashboard_summary_v2().await?;
        
        // Extract and normalize key metrics including new fields
        let btc_price = raw_data.get("btc_price_usd").cloned().unwrap_or(serde_json::Value::Null);
        let eth_price = raw_data.get("eth_price_usd").cloned().unwrap_or(serde_json::Value::Null);
        let sol_price = raw_data.get("sol_price_usd").cloned().unwrap_or(serde_json::Value::Null);
        let xrp_price = raw_data.get("xrp_price_usd").cloned().unwrap_or(serde_json::Value::Null);
        let ada_price = raw_data.get("ada_price_usd").cloned().unwrap_or(serde_json::Value::Null);
        let link_price = raw_data.get("link_price_usd").cloned().unwrap_or(serde_json::Value::Null);
        let bnb_price = raw_data.get("bnb_price_usd").cloned().unwrap_or(serde_json::Value::Null);
        let market_cap = raw_data.get("market_cap_usd").cloned().unwrap_or(serde_json::Value::Null);
        let volume_24h = raw_data.get("volume_24h_usd").cloned().unwrap_or(serde_json::Value::Null);
        let btc_change_24h = raw_data.get("btc_change_24h").cloned().unwrap_or(serde_json::Value::Null);
        let eth_change_24h = raw_data.get("eth_change_24h").cloned().unwrap_or(serde_json::Value::Null);
        let sol_change_24h = raw_data.get("sol_change_24h").cloned().unwrap_or(serde_json::Value::Null);
        let xrp_change_24h = raw_data.get("xrp_change_24h").cloned().unwrap_or(serde_json::Value::Null);
        let ada_change_24h = raw_data.get("ada_change_24h").cloned().unwrap_or(serde_json::Value::Null);
        let link_change_24h = raw_data.get("link_change_24h").cloned().unwrap_or(serde_json::Value::Null);
        let bnb_change_24h = raw_data.get("bnb_change_24h").cloned().unwrap_or(serde_json::Value::Null);
        let market_cap_change_24h = raw_data.get("market_cap_change_percentage_24h_usd").cloned().unwrap_or(serde_json::Value::Number(serde_json::Number::from(0)));
        let btc_dominance = raw_data.get("btc_market_cap_percentage").cloned().unwrap_or(serde_json::Value::Number(serde_json::Number::from(0)));
        let eth_dominance = raw_data.get("eth_market_cap_percentage").cloned().unwrap_or(serde_json::Value::Number(serde_json::Number::from(0)));
        let fear_greed = raw_data.get("fng_value").cloned().unwrap_or(serde_json::Value::Number(serde_json::Number::from(50)));
        let rsi_14 = raw_data.get("rsi_14").cloned().unwrap_or(serde_json::Value::Number(serde_json::Number::from(50)));
        
        // Extract US stock indices data from Layer 2
        let us_stock_indices = raw_data.get("us_stock_indices").cloned().unwrap_or(serde_json::json!({}));
        let data_sources = raw_data.get("data_sources").cloned().unwrap_or(serde_json::json!({}));
        let partial_failure = raw_data.get("partial_failure").cloned().unwrap_or(serde_json::Value::Bool(false));
        let fetch_duration_ms = raw_data.get("fetch_duration_ms").cloned().unwrap_or(serde_json::Value::Number(serde_json::Number::from(0)));
        
        println!("  🔍 [Layer 5 via Layer 3] BTC Price received: ${:?}", btc_price);
        println!("  🔍 [Layer 5 via Layer 3] ETH Price received: ${:?}", eth_price);
        println!("  🔍 [Layer 5 via Layer 3] SOL Price received: ${:?}", sol_price);
        println!("  🔍 [Layer 5 via Layer 3] XRP Price received: ${:?}", xrp_price);
        println!("  🔍 [Layer 5 via Layer 3] ADA Price received: ${:?}", ada_price);
        println!("  🔍 [Layer 5 via Layer 3] LINK Price received: ${:?}", link_price);
        println!("  🔍 [Layer 5 via Layer 3] BNB Price received: ${:?}", bnb_price);
        println!("  🔍 [Layer 5 via Layer 3] Market Cap received: ${:?}", market_cap);
        println!("  🔍 [Layer 5 via Layer 3] Market Cap Change 24h: {:?}%", market_cap_change_24h);
        println!("  🔍 [Layer 5 via Layer 3] BTC Dominance: {:?}%", btc_dominance);
        println!("  🔍 [Layer 5 via Layer 3] ETH Dominance: {:?}%", eth_dominance);
        println!("  🔍 [Layer 5 via Layer 3] Fear & Greed received: {:?}", fear_greed);
        println!("  🔍 [Layer 5 via Layer 3] US Stock Indices: {:?}", us_stock_indices.as_object().map_or(0, |obj| obj.len()));
        
        let normalized_data = serde_json::json!({
            "btc_price_usd": btc_price,
            "btc_change_24h": btc_change_24h,
            "eth_price_usd": eth_price,
            "eth_change_24h": eth_change_24h,
            "sol_price_usd": sol_price,
            "sol_change_24h": sol_change_24h,
            "xrp_price_usd": xrp_price,
            "xrp_change_24h": xrp_change_24h,
            "ada_price_usd": ada_price,
            "ada_change_24h": ada_change_24h,
            "link_price_usd": link_price,
            "link_change_24h": link_change_24h,
            "bnb_price_usd": bnb_price,
            "bnb_change_24h": bnb_change_24h,
            "market_cap_usd": market_cap,
            "volume_24h_usd": volume_24h,
            "market_cap_change_percentage_24h_usd": market_cap_change_24h,
            "btc_market_cap_percentage": btc_dominance,
            "eth_market_cap_percentage": eth_dominance,
            "fear_greed_index": fear_greed,
            "fng_value": fear_greed,
            "rsi_14": rsi_14,
            "us_stock_indices": us_stock_indices,
            "data_sources": data_sources,
            "partial_failure": partial_failure,
            "fetch_duration_ms": fetch_duration_ms,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "source": "layer2_external_apis",
            "normalized_by": "layer3_market_data_adapter"
        });
        
        println!("🔧 [Layer 5 via Layer 3] Data normalized for client compatibility with enhanced fields + US stock indices");
        Ok(normalized_data)
    }
    
    /// Health check for market data adapter
    pub async fn health_check(&self) -> bool {
        if let Some(external_apis) = &self.external_apis {
            match external_apis.health_check().await {
                Ok(_) => {
                    println!("  ✅ Market Data Adapter - Layer 2 connection healthy");
                    true
                }
                Err(e) => {
                    // Be tolerant of rate limiting and temporary issues
                    let error_msg = e.to_string();
                    if error_msg.contains("429") || error_msg.contains("Circuit breaker") || error_msg.contains("rate limit") {
                        println!("  ⚠️ Market Data Adapter - Layer 2 rate limited (functional)");
                        true
                    } else {
                        println!("  ❌ Market Data Adapter - Layer 2 connection unhealthy: {}", e);
                        false
                    }
                }
            }
        } else {
            println!("  ⚠️ Market Data Adapter - Layer 2 not configured");
            true // Not configured is not an error
        }
    }
    
    /// Check if Layer 2 is configured
    pub fn is_layer2_configured(&self) -> bool {
        self.external_apis.is_some()
    }
    
    // ===== NEW CACHE-FREE METHODS (Phase 2 Refactoring) =====
    
    /// Fetch dashboard summary using cache-free Layer 2 method (CACHE-OPTIMIZED)
    /// 
    /// OPTIMIZATION: Checks latest_market_data cache in Layer 3 first before calling Layer 2.
    /// This prevents unnecessary round-trips to Layer 2 when fresh data is available.
    /// Layer 1 handles all caching and streaming after this call.
    pub async fn fetch_dashboard_summary_v2(&self) -> Result<serde_json::Value> {
        println!("🔄 [Layer 3 → Cache Check] Checking latest_market_data cache first...");
        
        // STEP 1: Layer 3 Cache Check for latest_market_data (with Cache Stampede protection)
        if let Some(cache_system) = &self.cache_system {
            match cache_system.cache_manager().get("latest_market_data").await {
                Ok(Some(cached_data)) => {
                    println!("💨 [Layer 3] Cache HIT for latest_market_data - skipping Layer 2 call (Cache Stampede protected)");
                    println!("  🚀 Performance: Avoided Layer 2 round-trip");
                    return Ok(cached_data);
                }
                Ok(None) => {
                    println!("🔍 [Layer 3] Cache MISS for latest_market_data - proceeding to Layer 2");
                }
                Err(e) => {
                    println!("⚠️ [Layer 3] Cache system error, falling back to Layer 2: {}", e);
                }
            }
        } else {
            println!("⚠️ [Layer 3] No cache system configured, calling Layer 2 directly");
        }
        
        // STEP 2: Call Layer 2 if no cache or cache miss
        if let Some(external_apis) = &self.external_apis {
            println!("🔄 [Layer 3 → Layer 2 V2] Fetching dashboard summary (cache-free)...");
            let layer2_data = external_apis.fetch_dashboard_summary_v2().await?;
            
            // STEP 3: Store in Layer 3 cache for future requests
            if let Some(cache_system) = &self.cache_system {
                match cache_system.cache_manager().set_with_strategy("latest_market_data", layer2_data.clone(), CacheStrategy::RealTime).await {
                    Ok(_) => println!("💾 [Layer 3] Stored latest_market_data in cache (RealTime strategy - 10s TTL)"),
                    Err(e) => println!("⚠️ [Layer 3] Failed to cache latest_market_data: {}", e),
                }
            }
            
            println!("✅ [Layer 3] Fresh data fetched from Layer 2 and cached");
            Ok(layer2_data)
        } else {
            Err(anyhow::anyhow!("Layer 2 External APIs not configured in MarketDataAdapter"))
        }
    }
    
    // DEPRECATED v2 Methods - Replaced by unified fetch_normalized_market_data()
    // Individual v2 methods removed to prevent API call fragmentation
    // Use fetch_normalized_market_data() for all market data needs
    
    /// Fetch and stream dashboard data to Layer 1 (NEW - Integrated approach)
    /// 
    /// This method fetches from Layer 2 (cache-free) and returns raw data.
    /// Note: Streaming to Layer 1 will be implemented when AppState includes cache_system.
    pub async fn fetch_and_stream_dashboard(&self, _state: &std::sync::Arc<crate::service_islands::layer1_infrastructure::AppState>) -> Result<serde_json::Value> {
        println!("🌊 [Layer 3] Fetch dashboard data (streaming to Layer 1 pending)...");
        
        // Step 1: Fetch from Layer 2 (cache-free)
        let raw_data = self.fetch_dashboard_summary_v2().await?;
        
        // TODO: Step 2 - Stream to Layer 1 when AppState includes cache_system
        // let cache_system = &state.cache_system;
        // if let Some(cache) = cache_system {
        //     if let Ok(event_id) = cache.store_dashboard_summary(raw_data.clone()).await {
        //         println!("  ✅ Dashboard data streamed to Layer 1 (event: {})", event_id);
        //     }
        // }
        
        println!("✅ [Layer 3] Dashboard data fetched (cache-free) - ready for Layer 1 integration");
        Ok(raw_data)
    }
    
    /// Check if adapter supports cache-free mode and Layer 3 cache optimization
    pub fn supports_cache_free_mode(&self) -> bool {
        let layer2_support = if let Some(_external_apis) = &self.external_apis {
            // Replaced removed is_cache_free_mode method with direct logic
            true // ExternalApisIsland supports cache-free mode
        } else {
            false
        };
        
        let layer3_cache = self.cache_system.is_some();
        
        println!("🔧 [Layer 3] Cache optimization status:");
        println!("  - Layer 2 cache-free mode: {}", layer2_support);
        println!("  - Layer 3 cache system: {}", layer3_cache);
        
        layer2_support || layer3_cache
    }
    
    /// Check if Layer 3 cache system is configured
    pub fn is_cache_system_configured(&self) -> bool {
        self.cache_system.is_some()
    }
    
    /// Get cache optimization statistics
    pub async fn get_cache_statistics(&self) -> Result<serde_json::Value> {
        let mut stats = serde_json::json!({
            "layer3_cache_configured": self.is_cache_system_configured(),
            "layer2_configured": self.is_layer2_configured(),
            "optimization_mode": "layer3_cache_check"
        });
        
        if let Some(_cache_system) = &self.cache_system {
            // Cache statistics not available
            stats["layer3_cache_stats"] = serde_json::json!({"status": "configured"});
        }
        
        Ok(stats)
    }
}
