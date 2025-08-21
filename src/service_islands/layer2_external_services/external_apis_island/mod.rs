//! External APIs Island - Layer 2: External Services (Refactored)
//! 
//! This island manages all external API interactions including:
//! - Market data fetching from cryptocurrency APIs
//! - Rate limiting and circuit breaker protection  
//! - Data aggregation and normalization
//! - Error handling for external service calls
//! 
//! REFACTORING: Cache logic moved to Layer 1, this layer focuses on pure API business logic.
//! Maintains backward compatibility during transition.

pub mod market_data_api;
pub mod rate_limiter;
pub mod api_aggregator;
pub mod circuit_breaker;

use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;

use market_data_api::MarketDataApi;
use rate_limiter::RateLimiter;
use api_aggregator::ApiAggregator;
use circuit_breaker::CircuitBreaker;
use crate::service_islands::layer1_infrastructure::CacheSystemIsland;

/// External APIs Island
/// 
/// Central coordinator for all external API interactions.
/// Manages market data fetching, rate limiting, and error resilience with caching.
pub struct ExternalApisIsland {
    /// Market data API component
    pub market_data_api: Arc<MarketDataApi>,
    /// Rate limiting component
    pub rate_limiter: Arc<RateLimiter>,
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
        let rate_limiter = Arc::new(RateLimiter::new());
        let api_aggregator = Arc::new(ApiAggregator::new(taapi_secret).await?);
        let circuit_breaker = Arc::new(CircuitBreaker::new());
        
        println!("✅ External APIs Island initialized successfully (without cache)");
        
        Ok(Self {
            market_data_api,
            rate_limiter,
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
        let rate_limiter = Arc::new(RateLimiter::new());
        let api_aggregator = Arc::new(ApiAggregator::with_cache(taapi_secret, cache_system.clone()).await?);
        let circuit_breaker = Arc::new(CircuitBreaker::new());
        
        println!("✅ External APIs Island initialized with Cache System!");
        
        Ok(Self {
            market_data_api,
            rate_limiter,
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
            ("Rate Limiter", self.rate_limiter.health_check().await),
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
    /// Aggregates data from multiple external APIs with rate limiting, 
    /// circuit breaker protection, and intelligent caching.
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
        
        // Use rate limiter to control API calls
        self.rate_limiter.wait_for_limit("dashboard").await?;
        
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
    
    /// Fetch Bitcoin price data with cache support
    /// 
    /// Gets real-time BTC price with optimized caching and rate limiting.
    pub async fn fetch_btc_price(&self) -> Result<serde_json::Value> {
        let cache_key = "btc_price";
        
        // Try cache first if available
        if let Some(ref cache) = self.cache_system {
            if let Ok(Some(cached_data)) = cache.get(cache_key).await {
                println!("✅ BTC price served from cache");
                return Ok(cached_data);
            }
        }
        
        // Check BTC-specific circuit breaker
        if !self.circuit_breaker.can_proceed("btc").await {
            // Try to serve stale cache data
            if let Some(ref cache) = self.cache_system {
                if let Ok(Some(stale_data)) = cache.get(&format!("{}_stale", cache_key)).await {
                    println!("🔄 Serving stale BTC data due to circuit breaker");
                    return Ok(stale_data);
                }
            }
            return Err(anyhow::anyhow!("Circuit breaker is open for BTC API and no cached data available"));
        }
        
        // BTC has more frequent updates, use dedicated rate limiter
        self.rate_limiter.wait_for_limit("btc").await?;
        
        match self.market_data_api.fetch_btc_price().await {
            Ok(price_data) => {
                // Cache successful response if cache is available - Use price data strategy (5 min TTL)
                if let Some(ref cache) = self.cache_system {
                    // Use CacheStrategy::PriceData for 5-minute TTL
                    let cache_manager = cache.get_cache_manager();
                    let _ = cache_manager.set_with_strategy(cache_key, price_data.clone(), 
                        Duration::from_secs(300), // 5 minutes as requested
                        crate::service_islands::layer1_infrastructure::cache_system_island::cache_manager::CacheStrategy::PriceData).await;
                    
                    // Keep stale backup for circuit breaker fallback (15 min)
                    let _ = cache.set(&format!("{}_stale", cache_key), price_data.clone(), Some(Duration::from_secs(900))).await;
                    println!("💾 BTC price cached for 5 minutes (price data strategy)");
                }
                
                self.circuit_breaker.record_success("btc").await;
                Ok(price_data)
            }
            Err(e) => {
                // Try to serve stale cache data
                if let Some(ref cache) = self.cache_system {
                    if let Ok(Some(stale_data)) = cache.get(&format!("{}_stale", cache_key)).await {
                        println!("🔄 Serving stale BTC data due to API error");
                        return Ok(stale_data);
                    }
                }
                
                self.circuit_breaker.record_failure("btc").await;
                Err(e)
            }
        }
    }
    
    /// Get rate limit status
    /// 
    /// Returns current rate limiting and circuit breaker status.
    pub async fn get_rate_limit_status(&self) -> Result<serde_json::Value> {
        let rate_status = self.rate_limiter.get_status().await;
        let circuit_status = self.circuit_breaker.get_status().await;
        
        Ok(serde_json::json!({
            "rate_limiter": rate_status,
            "circuit_breaker": circuit_status,
            "last_updated": chrono::Utc::now().to_rfc3339()
        }))
    }
    
    /// Get external API statistics
    /// 
    /// Returns comprehensive statistics about external API usage.
    pub async fn get_api_statistics(&self) -> Result<serde_json::Value> {
        let market_stats = self.market_data_api.get_statistics().await;
        let rate_stats = self.rate_limiter.get_statistics().await;
        let circuit_stats = self.circuit_breaker.get_statistics().await;
        
        Ok(serde_json::json!({
            "market_data_api": market_stats,
            "rate_limiter": rate_stats,
            "circuit_breaker": circuit_stats,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    }
    
    // ===== NEW CACHE-FREE METHODS (Phase 2 Refactoring) =====
    
    /// Fetch dashboard data without cache logic (NEW - Cache-free)
    /// 
    /// Pure API business logic - no cache management.
    /// Layer 1 handles all caching and streaming.
    pub async fn fetch_dashboard_summary_v2(&self) -> Result<serde_json::Value> {
        println!("📊 [V2] Fetching dashboard data (cache-free)...");
        
        // Check circuit breaker only - no cache fallback
        if !self.circuit_breaker.can_proceed("global").await {
            return Err(anyhow::anyhow!("Circuit breaker is open for global APIs"));
        }
        
        // Use rate limiter
        self.rate_limiter.wait_for_limit("dashboard").await?;
        
        // Pure API fetch - no cache logic
        match self.api_aggregator.fetch_dashboard_data().await {
            Ok(data) => {
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
    
    /// Fetch BTC price without cache logic (NEW - Cache-free)
    /// 
    /// Pure API business logic focused on data fetching and validation.
    pub async fn fetch_btc_price_v2(&self) -> Result<serde_json::Value> {
        println!("₿ [V2] Fetching BTC price (cache-free)...");
        
        // Business logic checks only
        if !self.circuit_breaker.can_proceed("btc").await {
            return Err(anyhow::anyhow!("Circuit breaker is open for BTC API"));
        }
        
        self.rate_limiter.wait_for_limit("btc").await?;
        
        // Pure API fetch
        match self.market_data_api.fetch_btc_price().await {
            Ok(data) => {
                println!("✅ [V2] BTC price fetched - ready for Layer 1 processing");
                self.circuit_breaker.record_success("btc").await;
                Ok(data)
            }
            Err(e) => {
                println!("❌ [V2] BTC price fetch failed: {}", e);
                self.circuit_breaker.record_failure("btc").await;
                Err(e)
            }
        }
    }
    
    /// Fetch fear and greed index without cache logic (NEW - Cache-free)
    pub async fn fetch_fear_greed_index_v2(&self) -> Result<serde_json::Value> {
        println!("😨😤 [V2] Fetching Fear & Greed Index (cache-free)...");
        
        if !self.circuit_breaker.can_proceed("fear_greed").await {
            return Err(anyhow::anyhow!("Circuit breaker is open for Fear & Greed API"));
        }
        
        self.rate_limiter.wait_for_limit("fear_greed").await?;
        
        match self.market_data_api.fetch_fear_greed_index().await {
            Ok(data) => {
                println!("✅ [V2] Fear & Greed Index fetched - ready for Layer 1 processing");
                self.circuit_breaker.record_success("fear_greed").await;
                Ok(data)
            }
            Err(e) => {
                println!("❌ [V2] Fear & Greed Index fetch failed: {}", e);
                self.circuit_breaker.record_failure("fear_greed").await;
                Err(e)
            }
        }
    }
    
    /// Check if this instance uses cache-free mode
    pub fn is_cache_free_mode(&self) -> bool {
        self.cache_system.is_none()
    }
    
    /// Get API-only statistics (no cache stats)
    pub async fn get_api_only_statistics(&self) -> Result<serde_json::Value> {
        Ok(serde_json::json!({
            "island": "external_apis_v2",
            "mode": "cache_free",
            "responsibilities": [
                "External API calls",
                "Rate limiting",
                "Circuit breaker protection", 
                "Data validation and normalization"
            ],
            "removed_responsibilities": [
                "Data caching (moved to Layer 1)",
                "Cache invalidation (moved to Layer 1)",
                "Stale data serving (moved to Layer 1)"
            ],
            "components": {
                "market_data_api": "operational",
                "rate_limiter": "operational",
                "api_aggregator": "operational", 
                "circuit_breaker": "operational"
            }
        }))
    }
}
