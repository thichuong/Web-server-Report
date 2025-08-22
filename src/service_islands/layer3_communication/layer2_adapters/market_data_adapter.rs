//! Market Data Adapter - Layer 3 to Layer 2 Market Data Bridge
//! 
//! This adapter handles all market data fetching operations from Layer 2.
//! It provides a clean abstraction for Layer 3 components to access
//! Layer 2 External APIs Island market data services.

use anyhow::Result;
use serde_json;
use std::sync::Arc;
use std::time::Duration;

use crate::service_islands::layer2_external_services::external_apis_island::ExternalApisIsland;

/// Market Data Adapter
/// 
/// Handles all Layer 3 → Layer 2 market data communication.
/// Provides methods for fetching various types of market data
/// while maintaining proper Service Islands Architecture.
pub struct MarketDataAdapter {
    /// Reference to Layer 2 External APIs Island
    external_apis: Option<Arc<ExternalApisIsland>>,
}

impl MarketDataAdapter {
    /// Create new Market Data Adapter without Layer 2 dependency
    pub fn new() -> Self {
        Self {
            external_apis: None,
        }
    }
    
    /// Set Layer 2 External APIs dependency
    pub fn with_external_apis(mut self, external_apis: Arc<ExternalApisIsland>) -> Self {
        self.external_apis = Some(external_apis);
        self
    }
    
    /// Fetch dashboard summary data from Layer 2
    /// 
    /// Main method for getting comprehensive market data for dashboards.
    /// Used by Layer 5 business logic via Layer 3.
    pub async fn fetch_dashboard_summary(&self) -> Result<serde_json::Value> {
        if let Some(external_apis) = &self.external_apis {
            println!("🔄 [Layer 3 → Layer 2] Fetching dashboard summary...");
            external_apis.fetch_dashboard_summary().await
        } else {
            Err(anyhow::anyhow!("Layer 2 External APIs not configured in MarketDataAdapter"))
        }
    }
    
    /// Fetch BTC price data from Layer 2
    /// 
    /// Specialized method for getting Bitcoin price information.
    pub async fn fetch_btc_data(&self) -> Result<serde_json::Value> {
        if let Some(external_apis) = &self.external_apis {
            println!("🔄 [Layer 3 → Layer 2] Fetching BTC data...");
            external_apis.fetch_btc_price().await
        } else {
            Err(anyhow::anyhow!("Layer 2 External APIs not configured in MarketDataAdapter"))
        }
    }
    
    /// Fetch crypto fear and greed index from Layer 2
    /// 
    /// Specialized method for getting market sentiment data.
    pub async fn fetch_fear_greed_index(&self) -> Result<serde_json::Value> {
        if let Some(external_apis) = &self.external_apis {
            println!("🔄 [Layer 3 → Layer 2] Fetching Fear & Greed Index...");
            external_apis.market_data_api.fetch_fear_greed_index().await
        } else {
            Err(anyhow::anyhow!("Layer 2 External APIs not configured in MarketDataAdapter"))
        }
    }
    
    /// Fetch market data with timeout
    /// 
    /// Fetch dashboard data with configurable timeout for reliability.
    pub async fn fetch_dashboard_summary_with_timeout(&self, timeout: Duration) -> Result<serde_json::Value> {
        if let Some(external_apis) = &self.external_apis {
            println!("🔄 [Layer 3 → Layer 2] Fetching dashboard summary ({}s timeout)...", timeout.as_secs());
            
            tokio::time::timeout(timeout, external_apis.fetch_dashboard_summary()).await
                .map_err(|_| anyhow::anyhow!("Market data fetch timed out after {}s", timeout.as_secs()))?
        } else {
            Err(anyhow::anyhow!("Layer 2 External APIs not configured in MarketDataAdapter"))
        }
    }
    
    /// Normalize market data for Layer 5 consumption
    /// 
    /// Converts raw Layer 2 data into a format suitable for Layer 5 business logic.
    pub async fn fetch_normalized_market_data(&self) -> Result<serde_json::Value> {
        let raw_data = self.fetch_dashboard_summary().await?;
        
        // Extract and normalize key metrics
        let btc_price = raw_data.get("btc_price_usd").cloned().unwrap_or(serde_json::Value::Null);
        let market_cap = raw_data.get("market_cap_usd").cloned().unwrap_or(serde_json::Value::Null);
        let volume_24h = raw_data.get("volume_24h_usd").cloned().unwrap_or(serde_json::Value::Null);
        let btc_change_24h = raw_data.get("btc_change_24h").cloned().unwrap_or(serde_json::Value::Null);
        let fear_greed = raw_data.get("fng_value").cloned().unwrap_or(serde_json::Value::Number(serde_json::Number::from(50)));
        let rsi_14 = raw_data.get("rsi_14").cloned().unwrap_or(serde_json::Value::Number(serde_json::Number::from(50)));
        
        println!("  🔍 [Layer 5 via Layer 3] BTC Price received: ${:?}", btc_price);
        println!("  🔍 [Layer 5 via Layer 3] Market Cap received: ${:?}", market_cap);
        println!("  🔍 [Layer 5 via Layer 3] Fear & Greed received: {:?}", fear_greed);
        
        let normalized_data = serde_json::json!({
            "btc_price_usd": btc_price,
            "market_cap_usd": market_cap,
            "volume_24h_usd": volume_24h,
            "btc_change_24h": btc_change_24h,
            "fear_greed_index": fear_greed,
            "fng_value": fear_greed,
            "rsi_14": rsi_14,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "source": "layer2_external_apis",
            "normalized_by": "layer3_market_data_adapter"
        });
        
        println!("🔧 [Layer 5 via Layer 3] Data normalized for client compatibility");
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
    
    /// Fetch dashboard summary using cache-free Layer 2 method (NEW)
    /// 
    /// Uses the new cache-free APIs in Layer 2 for pure business logic.
    /// Layer 1 handles all caching and streaming after this call.
    pub async fn fetch_dashboard_summary_v2(&self) -> Result<serde_json::Value> {
        if let Some(external_apis) = &self.external_apis {
            println!("🔄 [Layer 3 → Layer 2 V2] Fetching dashboard summary (cache-free)...");
            external_apis.fetch_dashboard_summary_v2().await
        } else {
            Err(anyhow::anyhow!("Layer 2 External APIs not configured in MarketDataAdapter"))
        }
    }
    
    /// Fetch BTC data using cache-free Layer 2 method (NEW)
    pub async fn fetch_btc_data_v2(&self) -> Result<serde_json::Value> {
        if let Some(external_apis) = &self.external_apis {
            println!("🔄 [Layer 3 → Layer 2 V2] Fetching BTC data (cache-free)...");
            external_apis.fetch_btc_price_v2().await
        } else {
            Err(anyhow::anyhow!("Layer 2 External APIs not configured in MarketDataAdapter"))
        }
    }
    
    /// Fetch fear & greed index using cache-free Layer 2 method (NEW)
    pub async fn fetch_fear_greed_index_v2(&self) -> Result<serde_json::Value> {
        if let Some(external_apis) = &self.external_apis {
            println!("🔄 [Layer 3 → Layer 2 V2] Fetching Fear & Greed Index (cache-free)...");
            external_apis.fetch_fear_greed_index_v2().await
        } else {
            Err(anyhow::anyhow!("Layer 2 External APIs not configured in MarketDataAdapter"))
        }
    }
    
    /// Fetch and stream dashboard data to Layer 1 (NEW - Integrated approach)
    /// 
    /// This method fetches from Layer 2 (cache-free) and returns raw data.
    /// Note: Streaming to Layer 1 will be implemented when AppState includes cache_system.
    pub async fn fetch_and_stream_dashboard(&self, _state: &std::sync::Arc<crate::state::AppState>) -> Result<serde_json::Value> {
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
    
    /// Check if adapter supports cache-free mode
    pub fn supports_cache_free_mode(&self) -> bool {
        if let Some(external_apis) = &self.external_apis {
            external_apis.is_cache_free_mode()
        } else {
            false
        }
    }
}
