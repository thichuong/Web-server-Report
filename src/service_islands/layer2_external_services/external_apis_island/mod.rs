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

use market_data_api::MarketDataApi;
use api_aggregator::ApiAggregator;
use circuit_breaker::CircuitBreaker;
use crate::service_islands::layer1_infrastructure::CacheSystemIsland;

/// External APIs Island
/// 
/// Central coordinator for all external API interactions.
/// Manages market data fetching and error resilience with caching.
#[allow(dead_code)]
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
    /// Initialize External APIs Island with Cache System dependency
    #[allow(dead_code)]
    pub async fn with_cache(taapi_secret: String, cache_system: Arc<CacheSystemIsland>) -> Result<Self> {
        Self::with_cache_and_cmc(taapi_secret, None, cache_system).await
    }
    
    /// Initialize External APIs Island with Cache System dependency and CoinMarketCap support
    pub async fn with_cache_and_cmc(
        taapi_secret: String, 
        cmc_api_key: Option<String>, 
        cache_system: Arc<CacheSystemIsland>
    ) -> Result<Self> {
        Self::with_cache_and_all_keys(taapi_secret, cmc_api_key, None, cache_system).await
    }
    
    /// Initialize External APIs Island with Cache System dependency and all API keys
    pub async fn with_cache_and_all_keys(
        taapi_secret: String, 
        cmc_api_key: Option<String>, 
        finnhub_api_key: Option<String>,
        cache_system: Arc<CacheSystemIsland>
    ) -> Result<Self> {
        println!("🏝️ Initializing External APIs Island with Cache System...");
        
        // Initialize components with cache integration
        let market_data_api = Arc::new(MarketDataApi::with_all_keys(taapi_secret.clone(), cmc_api_key.clone(), finnhub_api_key.clone()).await?);
        let api_aggregator = Arc::new(ApiAggregator::with_cache_and_all_keys(taapi_secret, cmc_api_key, finnhub_api_key, cache_system.clone()).await?);
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

    /// Fetch dashboard summary V2 using aggregator
    pub async fn fetch_dashboard_summary_v2(&self) -> Result<serde_json::Value> {
        self.api_aggregator.fetch_dashboard_data().await
    }
}

// Re-export commonly used types
// Note: DashboardSummary removed as it's not used outside this module
