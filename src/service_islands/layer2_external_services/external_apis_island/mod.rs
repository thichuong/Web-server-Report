//! External APIs Island - Layer 2: External Services (Optimized for Maximum Performance)
//! 
//! This island manages all external API interactions including:
//! - Market data fetching from cryptocurrency APIs
//! - Circuit breaker protection for service resilience
//! - Data aggregation and normalization
//! - Error handling for external service calls
//! 
//! PERFORMANCE OPTIMIZATION: Rate limiting completely removed for maximum throughput.
//! Cache logic handled by Layer 1, this layer focuses on pure API business logic.

pub mod market_data_api;
pub mod api_aggregator;
pub mod circuit_breaker;

use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;

use market_data_api::MarketDataApi;
use api_aggregator::ApiAggregator;
use circuit_breaker::CircuitBreaker;
use crate::service_islands::layer1_infrastructure::CacheSystemIsland;

/// External APIs Island
/// 
/// Central coordinator for all external API interactions.
/// Manages market data fetching and error resilience with caching.
pub struct ExternalApisIsland {
    /// Market data API component
    pub market_data_api: Arc<MarketDataApi>,
    /// API aggregation component
    pub api_aggregator: Arc<ApiAggregator>,
    /// Circuit breaker component
    pub circuit_breaker: Arc<CircuitBreaker>,
    /// Cache system for API response caching
    pub cache_system: Option<Arc<CacheSystemIsland>>,
}

impl ExternalApisIsland {
    /// Initialize the External APIs Island
    /// 
    /// Creates all components for external service management.
    pub async fn new(taapi_secret: String) -> Result<Self> {
        println!("🏝️ Initializing External APIs Island (Layer 2 External Services)...");
        
        // Initialize components
        let market_data_api = Arc::new(MarketDataApi::new(taapi_secret.clone()).await?);
        let api_aggregator = Arc::new(ApiAggregator::new(taapi_secret).await?);
        let circuit_breaker = Arc::new(CircuitBreaker::new());
        
        println!("✅ External APIs Island initialized successfully (without cache)");
        
        Ok(Self {
            market_data_api,
            api_aggregator,
            circuit_breaker,
            cache_system: None,
        })
    }
    
    /// Initialize External APIs Island with Cache System dependency
    pub async fn with_cache(taapi_secret: String, cache_system: Arc<CacheSystemIsland>) -> Result<Self> {
        println!("🏝️ Initializing External APIs Island with Cache System...");
        
        // Initialize components with cache integration
        let market_data_api = Arc::new(MarketDataApi::new(taapi_secret.clone()).await?);
        let api_aggregator = Arc::new(ApiAggregator::with_cache(taapi_secret, cache_system.clone()).await?);
        let circuit_breaker = Arc::new(CircuitBreaker::new());
        
        println!("✅ External APIs Island initialized with Cache System!");
        
        Ok(Self {
            market_data_api,
            api_aggregator,
            circuit_breaker,
            cache_system: Some(cache_system),
        })
    }
    
    /// Health check for the entire External APIs Island
    /// 
    /// Validates that all external service components are operational.
    pub async fn health_check(&self) -> Result<()> {
        println!("🏥 Checking External APIs Island health...");
        
        // Check all components
        let checks = vec![
            ("Market Data API", self.market_data_api.health_check().await),
            ("API Aggregator", self.api_aggregator.health_check().await),
            ("Circuit Breaker", self.circuit_breaker.health_check().await),
        ];
        
        let mut all_healthy = true;
        for (component, healthy) in checks {
            if healthy {
                println!("  ✅ {} - Healthy", component);
            } else {
                println!("  ❌ {} - Unhealthy", component);
                all_healthy = false;
            }
        }
        
        if all_healthy {
            println!("✅ External APIs Island - All components healthy");
            Ok(())
        } else {
            Err(anyhow::anyhow!("External APIs Island - Some components unhealthy"))
        }
    }
    
    /// Fetch comprehensive dashboard data with cache support
    /// 
    /// Aggregates data from multiple external APIs with maximum performance optimization.
    /// Features circuit breaker protection and intelligent caching.
    pub async fn fetch_dashboard_summary(&self) -> Result<serde_json::Value> {
        let cache_key = "dashboard_summary";
        
        // Try cache first if available
        if let Some(ref cache) = self.cache_system {
            if let Ok(Some(cached_data)) = cache.get(cache_key).await {
                println!("✅ Dashboard data served from cache");
                return Ok(cached_data);
            }
        }
        
        // Check circuit breakers
        if !self.circuit_breaker.can_proceed("global").await {
            // Try to serve stale cache data if available
            if let Some(ref cache) = self.cache_system {
                if let Ok(Some(stale_data)) = cache.get(&format!("{}_stale", cache_key)).await {
                    println!("🔄 Serving stale cache data due to circuit breaker");
                    return Ok(stale_data);
                }
            }
            return Err(anyhow::anyhow!("Circuit breaker is open for global APIs and no cached data available"));
        }
        
        // Rate limiting removed for maximum performance optimization
        
        // Delegate to API aggregator
        match self.api_aggregator.fetch_dashboard_data().await {
            Ok(data) => {
                // Cache successful response if cache is available - Short TTL for CoinGecko APIs
                if let Some(ref cache) = self.cache_system {
                    let _ = cache.set(cache_key, data.clone(), Some(Duration::from_secs(30))).await; // 30 sec cache for CoinGecko
                    let _ = cache.set(&format!("{}_stale", cache_key), data.clone(), Some(Duration::from_secs(1800))).await; // 30 min stale backup
                    println!("💾 Dashboard data cached for 30 seconds (CoinGecko rate limit optimization)");
                }
                
                self.circuit_breaker.record_success("global").await;
                Ok(data)
            }
            Err(e) => {
                // Try to serve stale cache data
                if let Some(ref cache) = self.cache_system {
                    if let Ok(Some(stale_data)) = cache.get(&format!("{}_stale", cache_key)).await {
                        println!("🔄 Serving stale cache data due to API error");
                        return Ok(stale_data);
                    }
                }
                
                self.circuit_breaker.record_failure("global").await;
                Err(e)
            }
        }
    }
    
    /// DEPRECATED: Individual BTC fetch with cache creates redundant API calls
    /// Use fetch_dashboard_summary() which includes BTC data in aggregated response
    pub async fn fetch_btc_price(&self) -> Result<serde_json::Value> {
        println!("⚠️ DEPRECATED: fetch_btc_price() - Use fetch_dashboard_summary() instead");
        Err(anyhow::anyhow!("DEPRECATED: Use fetch_dashboard_summary() for BTC data to prevent redundant API calls"))
    }
    
    /// Get circuit breaker status
    /// 
    /// Returns current circuit breaker status.
    pub async fn get_circuit_breaker_status(&self) -> Result<serde_json::Value> {
        let circuit_status = self.circuit_breaker.get_status().await;
        
        Ok(serde_json::json!({
            "circuit_breaker": circuit_status,
            "last_updated": chrono::Utc::now().to_rfc3339()
        }))
    }
    
    /// Get external API statistics
    /// 
    /// Returns comprehensive statistics about external API usage.
    pub async fn get_api_statistics(&self) -> Result<serde_json::Value> {
        let market_stats = self.market_data_api.get_statistics().await;
        let circuit_stats = self.circuit_breaker.get_statistics().await;
        
        Ok(serde_json::json!({
            "market_data_api": market_stats,
            "circuit_breaker": circuit_stats,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    }
    
    // ===== NEW CACHE-FREE METHODS (Phase 2 Refactoring) =====
    
    /// Intelligent dashboard fetch - Maximum performance optimization
    /// 
    /// This method prioritizes cache for optimal performance.
    /// All API calls are immediate with no rate limiting delays.
    pub async fn fetch_dashboard_summary_v2(&self) -> Result<serde_json::Value> {
        println!("📊 [V2] Fetching dashboard data (maximum performance mode)...");
        
        // Step 1: Try cache first - Immediate response for cached data
        if let Some(ref cache) = self.cache_system {
            let cache_key = "dashboard_summary";
            if let Ok(Some(cached_data)) = cache.get(cache_key).await {
                println!("💨 [V2] Dashboard served from cache (maximum speed)");
                return Ok(cached_data);
            }
        }
        
        // Step 2: Direct API calls with maximum performance
        println!("🔄 [V2] Cache miss - fetching fresh data immediately...");
        
        // Check circuit breaker only - no cache fallback
        if !self.circuit_breaker.can_proceed("global").await {
            return Err(anyhow::anyhow!("Circuit breaker is open for global APIs"));
        }
        
        // Maximum performance - all API calls are immediate with no delays
        match self.api_aggregator.fetch_dashboard_data().await {
            Ok(data) => {
                // Cache the result for future fast access
                if let Some(ref cache) = self.cache_system {
                    let _ = cache.set("dashboard_summary", data.clone(), Some(Duration::from_secs(30))).await;
                    println!("💾 [V2] Fresh data cached for 30 seconds");
                }
                
                println!("✅ [V2] Dashboard data fetched - ready for Layer 1 processing");
                self.circuit_breaker.record_success("global").await;
                Ok(data)
            }
            Err(e) => {
                println!("❌ [V2] Dashboard data fetch failed: {}", e);
                self.circuit_breaker.record_failure("global").await;
                Err(e)
            }
        }
    }
    
    // ===== DEPRECATED INDIVIDUAL METHODS - Use fetch_dashboard_summary_v2() instead =====
    
    /// DEPRECATED: Individual BTC fetch creates redundant API calls
    /// Use fetch_dashboard_summary_v2() which includes BTC data in aggregated response
    pub async fn fetch_btc_price_v2(&self) -> Result<serde_json::Value> {
        Err(anyhow::anyhow!("DEPRECATED: Use fetch_dashboard_summary_v2() for BTC data to prevent redundant API calls"))
    }
    
    /// DEPRECATED: Individual Fear & Greed fetch creates redundant API calls  
    /// Use fetch_dashboard_summary_v2() which includes Fear & Greed data in aggregated response
    pub async fn fetch_fear_greed_index_v2(&self) -> Result<serde_json::Value> {
        Err(anyhow::anyhow!("DEPRECATED: Use fetch_dashboard_summary_v2() for Fear & Greed data to prevent redundant API calls"))
    }
    
    /// Check if this instance uses cache-free mode
    pub fn is_cache_free_mode(&self) -> bool {
        self.cache_system.is_none()
    }
    
    /// Get API-only statistics (no cache stats)
    pub async fn get_api_only_statistics(&self) -> Result<serde_json::Value> {
        Ok(serde_json::json!({
            "island": "external_apis_v2",
            "mode": "maximum_performance",
            "responsibilities": [
                "External API calls",
                "Circuit breaker protection", 
                "Data validation and normalization"
            ],
            "removed_responsibilities": [
                "Data caching (moved to Layer 1)",
                "Cache invalidation (moved to Layer 1)",
                "Stale data serving (moved to Layer 1)",
                "Rate limiting (removed for maximum performance)"
            ],
            "components": {
                "market_data_api": "operational",
                "api_aggregator": "operational", 
                "circuit_breaker": "operational"
            }
        }))
    }
}
