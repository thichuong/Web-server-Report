//! Refactored External APIs Island - Layer 2 Without Cache Logic
//! 
//! This is the refactored version of Layer 2 External APIs Island with:
//! - REMOVED: All cache logic (moved to Layer 1)
//! - FOCUS: Pure API business logic, rate limiting, circuit breaker
//! - STREAM: Direct data streaming to Layer 1 Redis Streams
//! - CLEAN: Single responsibility - external API management only

pub mod market_data_api;
pub mod rate_limiter;
pub mod api_aggregator;
pub mod circuit_breaker;

use anyhow::Result;
use std::sync::Arc;

use market_data_api::MarketDataApi;
use rate_limiter::RateLimiter;
use api_aggregator::ApiAggregator;
use circuit_breaker::CircuitBreaker;
// REMOVED: Cache system dependency - cache is now handled in Layer 1
// use crate::service_islands::layer1_infrastructure::CacheSystemIsland;

/// External APIs Island - Refactored for Clean Architecture
/// 
/// Central coordinator for external API interactions with focus on:
/// - API business logic and data processing
/// - Rate limiting and circuit breaker protection  
/// - Error handling and resilience patterns
/// - Direct streaming to Layer 1 (no cache management)
pub struct ExternalApisIslandV2 {
    /// Market data API component
    pub market_data_api: Arc<MarketDataApi>,
    /// Rate limiting component
    pub rate_limiter: Arc<RateLimiter>,
    /// API aggregation component  
    pub api_aggregator: Arc<ApiAggregator>,
    /// Circuit breaker component
    pub circuit_breaker: Arc<CircuitBreaker>,
    // REMOVED: cache_system field - Layer 1 handles all caching
}

impl ExternalApisIslandV2 {
    /// Initialize the External APIs Island without cache dependencies
    /// 
    /// Creates all components for external service management.
    /// Cache logic is completely handled by Layer 1.
    pub async fn new(taapi_secret: String) -> Result<Self> {
        println!("🏝️ Initializing External APIs Island V2 (Cache-Free)...");
        
        // Initialize components without cache integration
        let market_data_api = Arc::new(MarketDataApi::new(taapi_secret.clone()).await?);
        let rate_limiter = Arc::new(RateLimiter::new());
        let api_aggregator = Arc::new(ApiAggregator::new_stream_mode(taapi_secret).await?); // New stream mode
        let circuit_breaker = Arc::new(CircuitBreaker::new());
        
        println!("✅ External APIs Island V2 initialized (pure business logic)");
        
        Ok(Self {
            market_data_api,
            rate_limiter, 
            api_aggregator,
            circuit_breaker,
            // No cache system field
        })
    }
    
    /// Health check for the External APIs Island
    pub async fn health_check(&self) -> Result<()> {
        println!("🏥 Checking External APIs Island V2 health...");
        
        // Check all components (no cache checks)
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
            println!("✅ External APIs Island V2 - All components healthy");
            Ok(())
        } else {
            Err(anyhow::anyhow!("External APIs Island V2 - Some components unhealthy"))
        }
    }
    
    /// Fetch dashboard data and stream to Layer 1
    /// 
    /// REFACTORED: No cache logic - just fetch and return raw data.
    /// Layer 1 handles caching and streaming automatically.
    pub async fn fetch_dashboard_summary(&self) -> Result<serde_json::Value> {
        println!("📊 Fetching dashboard data (cache-free)...");
        
        // Check circuit breaker only
        if !self.circuit_breaker.can_proceed("global").await {
            return Err(anyhow::anyhow!("Circuit breaker is open for global APIs"));
        }
        
        // Use rate limiter
        self.rate_limiter.wait_for_limit("dashboard").await?;
        
        // Fetch data without any cache logic
        match self.api_aggregator.fetch_dashboard_data().await {
            Ok(data) => {
                println!("✅ Dashboard data fetched - ready for Layer 1 streaming");
                self.circuit_breaker.record_success("global").await;
                
                // REMOVED: All cache operations
                // Layer 1 will handle caching/streaming when it receives this data
                
                Ok(data)
            }
            Err(e) => {
                println!("❌ Dashboard data fetch failed: {}", e);
                self.circuit_breaker.record_failure("global").await;
                Err(e)
            }
        }
    }
    
    /// Fetch BTC price data without cache logic
    /// 
    /// REFACTORED: Pure API fetch, no caching concerns.
    pub async fn fetch_btc_price(&self) -> Result<serde_json::Value> {
        println!("₿ Fetching BTC price (cache-free)...");
        
        // Business logic checks only
        if !self.circuit_breaker.can_proceed("btc").await {
            return Err(anyhow::anyhow!("Circuit breaker is open for BTC API"));
        }
        
        self.rate_limiter.wait_for_limit("btc").await?;
        
        // Pure API fetch
        match self.market_data_api.get_btc_price().await {
            Ok(data) => {
                println!("✅ BTC price fetched - ready for Layer 1 streaming");
                self.circuit_breaker.record_success("btc").await;
                Ok(data)
            }
            Err(e) => {
                println!("❌ BTC price fetch failed: {}", e);
                self.circuit_breaker.record_failure("btc").await;
                Err(e)
            }
        }
    }
    
    /// Fetch fear and greed index without cache logic
    pub async fn fetch_fear_greed_index(&self) -> Result<serde_json::Value> {
        println!("😨😤 Fetching Fear & Greed Index (cache-free)...");
        
        if !self.circuit_breaker.can_proceed("fear_greed").await {
            return Err(anyhow::anyhow!("Circuit breaker is open for Fear & Greed API"));
        }
        
        self.rate_limiter.wait_for_limit("fear_greed").await?;
        
        match self.market_data_api.fetch_fear_greed_index().await {
            Ok(data) => {
                println!("✅ Fear & Greed Index fetched - ready for Layer 1 streaming");
                self.circuit_breaker.record_success("fear_greed").await;
                Ok(data)
            }
            Err(e) => {
                println!("❌ Fear & Greed Index fetch failed: {}", e);
                self.circuit_breaker.record_failure("fear_greed").await;
                Err(e)
            }
        }
    }
    
    /// Get API statistics (no cache stats)
    pub async fn get_api_statistics(&self) -> Result<serde_json::Value> {
        Ok(serde_json::json!({
            "island": "external_apis_v2",
            "version": "cache_free",
            "components": {
                "market_data_api": "operational",
                "rate_limiter": "operational", 
                "api_aggregator": "operational",
                "circuit_breaker": "operational"
            },
            "responsibilities": [
                "External API calls",
                "Rate limiting", 
                "Circuit breaker protection",
                "Data validation and normalization"
            ],
            "removed_responsibilities": [
                "Data caching (moved to Layer 1)",
                "Cache invalidation (moved to Layer 1)",
                "Cache statistics (moved to Layer 1)",
                "Stale data serving (moved to Layer 1)"
            ]
        }))
    }
}

// Example of how the API flow changes:
// 
// OLD FLOW (Layer 2 with cache):
// External API → Layer 2 → Check Cache → Return/Store → Layer 3
// 
// NEW FLOW (Layer 2 cache-free):
// External API → Layer 2 → Return Raw Data → Layer 3 → Layer 1 (Stream/Cache)
//
// Benefits:
// ✅ Single responsibility: Layer 2 = API business logic only
// ✅ Clean separation: Layer 1 = All storage concerns  
// ✅ Better testability: API logic isolated from caching
// ✅ Easier scaling: Cache scaling independent of API scaling
// ✅ Event sourcing: All data changes flow through streams
