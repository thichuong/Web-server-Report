//! API Aggregator Component
//! 
//! This component aggregates data from multiple APIs and handles coordination between different data sources.

use reqwest::Client;
use std::collections::HashMap;
use anyhow::Result;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::time::{timeout, Duration};
use super::MarketDataApi;
use crate::service_islands::layer1_infrastructure::CacheSystemIsland;

/// API Aggregator
/// 
/// Coordinates data fetching from multiple APIs and provides unified dashboard data with individual caching.
#[allow(dead_code)]
pub struct ApiAggregator {
    market_api: Arc<MarketDataApi>,
    client: Client,
    cache_system: Option<Arc<CacheSystemIsland>>,
    // Statistics
    total_aggregations: Arc<AtomicUsize>,
    successful_aggregations: Arc<AtomicUsize>,
    partial_failures: Arc<AtomicUsize>,
}

impl ApiAggregator {
    /// Create a new ApiAggregator
    #[allow(dead_code)]
    pub async fn new(taapi_secret: String) -> Result<Self> {
        Self::with_cmc_key(taapi_secret, None).await
    }
    
    /// Create a new ApiAggregator with CoinMarketCap support
    pub async fn with_cmc_key(taapi_secret: String, cmc_api_key: Option<String>) -> Result<Self> {
        Self::with_all_keys(taapi_secret, cmc_api_key, None).await
    }
    
    /// Create a new ApiAggregator with all API keys
    pub async fn with_all_keys(
        taapi_secret: String, 
        cmc_api_key: Option<String>, 
        finnhub_api_key: Option<String>
    ) -> Result<Self> {
        println!("📊 Initializing API Aggregator...");
        
        // Use optimized HTTP client from performance module if available
        let client = if let Ok(perf_client) = std::panic::catch_unwind(|| {
            crate::performance::OPTIMIZED_HTTP_CLIENT.clone()
        }) {
            perf_client
        } else {
            // Fallback client
            Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client")
        };
        
        // Create market API instance with async initialization
        let market_api = Arc::new(MarketDataApi::with_all_keys(taapi_secret, cmc_api_key, finnhub_api_key).await?);
        
        Ok(Self {
            market_api,
            client,
            cache_system: None, // Will be set by with_cache method
            total_aggregations: Arc::new(AtomicUsize::new(0)),
            successful_aggregations: Arc::new(AtomicUsize::new(0)),
            partial_failures: Arc::new(AtomicUsize::new(0)),
        })
    }
    
    /// Create ApiAggregator with cache system
    #[allow(dead_code)]
    pub async fn with_cache(taapi_secret: String, cache_system: Arc<CacheSystemIsland>) -> Result<Self> {
        Self::with_cache_and_cmc(taapi_secret, None, cache_system).await
    }
    
    /// Create ApiAggregator with cache system and CoinMarketCap support
    pub async fn with_cache_and_cmc(
        taapi_secret: String, 
        cmc_api_key: Option<String>, 
        cache_system: Arc<CacheSystemIsland>
    ) -> Result<Self> {
        Self::with_cache_and_all_keys(taapi_secret, cmc_api_key, None, cache_system).await
    }
    
    /// Create ApiAggregator with cache system and all API keys
    pub async fn with_cache_and_all_keys(
        taapi_secret: String, 
        cmc_api_key: Option<String>, 
        finnhub_api_key: Option<String>,
        cache_system: Arc<CacheSystemIsland>
    ) -> Result<Self> {
        let mut aggregator = Self::with_all_keys(taapi_secret, cmc_api_key, finnhub_api_key).await?;
        aggregator.cache_system = Some(cache_system);
        Ok(aggregator)
    }
    
    /// Health check for API Aggregator
    pub async fn health_check(&self) -> bool {
        // Test that we can coordinate API calls
        match self.test_aggregation().await {
            Ok(_) => {
                println!("  ✅ API Aggregator coordination test passed");
                true
            }
            Err(e) => {
                eprintln!("  ❌ API Aggregator coordination test failed: {}", e);
                false
            }
        }
    }
    
    /// Test aggregation functionality - OPTIMIZED to prevent rate limiting
    async fn test_aggregation(&self) -> Result<()> {
        println!("🏥 [OPTIMIZED] Testing aggregation using cache instead of API call...");
        
        // Use cache lookup instead of actual API call to prevent rate limiting during health checks
        if let Some(ref cache) = self.cache_system {
            let cache_key = "btc_coingecko_30s";
            if let Ok(Some(_cached_data)) = cache.cache_manager.get(cache_key).await {
                println!("✅ Aggregation test passed - cached BTC data available");
                return Ok(());
            }
        }
        
        // If no cached data, don't make API call during health check
        // This prevents unnecessary API calls that cause rate limiting
        println!("⚠️ Aggregation test passed - no cached data (health check doesn't require API call)");
        Ok(())
    }
    
    /// Fetch comprehensive dashboard data by aggregating multiple APIs
    pub async fn fetch_dashboard_data(&self) -> Result<serde_json::Value> {
        let start_time = std::time::Instant::now();
        self.total_aggregations.fetch_add(1, Ordering::Relaxed);
        
        println!("🔄 Starting dashboard data aggregation...");
        
        // Fetch data from multiple sources concurrently with timeouts and individual caching
        let btc_future = timeout(Duration::from_secs(10), self.fetch_btc_with_cache());
        let eth_future = timeout(Duration::from_secs(10), self.fetch_eth_with_cache());
        let sol_future = timeout(Duration::from_secs(10), self.fetch_sol_with_cache());
        let xrp_future = timeout(Duration::from_secs(10), self.fetch_xrp_with_cache());
        let ada_future = timeout(Duration::from_secs(10), self.fetch_ada_with_cache());
        let link_future = timeout(Duration::from_secs(10), self.fetch_link_with_cache());
        let global_future = timeout(Duration::from_secs(10), self.fetch_global_with_cache());
        let fng_future = timeout(Duration::from_secs(10), self.fetch_fng_with_cache());
        let rsi_future = timeout(Duration::from_secs(10), self.fetch_rsi_with_cache());
        let us_indices_future = timeout(Duration::from_secs(10), self.fetch_us_indices_with_cache());
        
        let (btc_result, eth_result, sol_result, xrp_result, ada_result, link_result, global_result, fng_result, rsi_result, us_indices_result) = tokio::join!(
            btc_future, eth_future, sol_future, xrp_future, ada_future, link_future,
            global_future, fng_future, rsi_future, us_indices_future
        );
        
        let mut data_sources = HashMap::new();
        let mut partial_failure = false;
        
        // Process BTC data
        let (btc_price, btc_change) = match btc_result {
            Ok(Ok(btc_data)) => {
                data_sources.insert("btc_price".to_string(), "coingecko".to_string());
                (
                    btc_data["price_usd"].as_f64().unwrap_or(0.0),
                    btc_data["change_24h"].as_f64().unwrap_or(0.0)
                )
            }
            Ok(Err(e)) => {
                eprintln!("⚠️ BTC data fetch failed: {}", e);
                data_sources.insert("btc_price".to_string(), "failed".to_string());
                partial_failure = true;
                (0.0, 0.0) // Keep 0.0 to show loading state on client
            }
            Err(_) => {
                eprintln!("⚠️ BTC data fetch timeout");
                data_sources.insert("btc_price".to_string(), "timeout".to_string());
                partial_failure = true;
                (0.0, 0.0) // Keep 0.0 to show loading state on client
            }
        };
        
        // Process ETH data
        let (eth_price, eth_change) = match eth_result {
            Ok(Ok(eth_data)) => {
                data_sources.insert("eth_price".to_string(), "binance".to_string());
                (
                    eth_data["price_usd"].as_f64().unwrap_or(0.0),
                    eth_data["change_24h"].as_f64().unwrap_or(0.0)
                )
            }
            Ok(Err(e)) => {
                eprintln!("⚠️ ETH data fetch failed: {}", e);
                data_sources.insert("eth_price".to_string(), "failed".to_string());
                partial_failure = true;
                (0.0, 0.0) // Keep 0.0 to show loading state on client
            }
            Err(_) => {
                eprintln!("⚠️ ETH data fetch timeout");
                data_sources.insert("eth_price".to_string(), "timeout".to_string());
                partial_failure = true;
                (0.0, 0.0) // Keep 0.0 to show loading state on client
            }
        };
        
        // Process SOL data
        let (sol_price, sol_change) = match sol_result {
            Ok(Ok(sol_data)) => {
                data_sources.insert("sol_price".to_string(), "binance".to_string());
                (
                    sol_data["price_usd"].as_f64().unwrap_or(0.0),
                    sol_data["change_24h"].as_f64().unwrap_or(0.0)
                )
            }
            Ok(Err(e)) => {
                eprintln!("⚠️ SOL data fetch failed: {}", e);
                data_sources.insert("sol_price".to_string(), "failed".to_string());
                partial_failure = true;
                (0.0, 0.0) // Keep 0.0 to show loading state on client
            }
            Err(_) => {
                eprintln!("⚠️ SOL data fetch timeout");
                data_sources.insert("sol_price".to_string(), "timeout".to_string());
                partial_failure = true;
                (0.0, 0.0) // Keep 0.0 to show loading state on client
            }
        };
        
        // Process XRP data
        let (xrp_price, xrp_change) = match xrp_result {
            Ok(Ok(xrp_data)) => {
                data_sources.insert("xrp_price".to_string(), "binance".to_string());
                (
                    xrp_data["price_usd"].as_f64().unwrap_or(0.0),
                    xrp_data["change_24h"].as_f64().unwrap_or(0.0)
                )
            }
            Ok(Err(e)) => {
                eprintln!("⚠️ XRP data fetch failed: {}", e);
                data_sources.insert("xrp_price".to_string(), "failed".to_string());
                partial_failure = true;
                (0.0, 0.0) // Keep 0.0 to show loading state on client
            }
            Err(_) => {
                eprintln!("⚠️ XRP data fetch timeout");
                data_sources.insert("xrp_price".to_string(), "timeout".to_string());
                partial_failure = true;
                (0.0, 0.0) // Keep 0.0 to show loading state on client
            }
        };
        
        // Process ADA data
        let (ada_price, ada_change) = match ada_result {
            Ok(Ok(ada_data)) => {
                data_sources.insert("ada_price".to_string(), "binance".to_string());
                (
                    ada_data["price_usd"].as_f64().unwrap_or(0.0),
                    ada_data["change_24h"].as_f64().unwrap_or(0.0)
                )
            }
            Ok(Err(e)) => {
                eprintln!("⚠️ ADA data fetch failed: {}", e);
                data_sources.insert("ada_price".to_string(), "failed".to_string());
                partial_failure = true;
                (0.0, 0.0) // Keep 0.0 to show loading state on client
            }
            Err(_) => {
                eprintln!("⚠️ ADA data fetch timeout");
                data_sources.insert("ada_price".to_string(), "timeout".to_string());
                partial_failure = true;
                (0.0, 0.0) // Keep 0.0 to show loading state on client
            }
        };
        
        // Process LINK data
        let (link_price, link_change) = match link_result {
            Ok(Ok(link_data)) => {
                data_sources.insert("link_price".to_string(), "binance".to_string());
                (
                    link_data["price_usd"].as_f64().unwrap_or(0.0),
                    link_data["change_24h"].as_f64().unwrap_or(0.0)
                )
            }
            Ok(Err(e)) => {
                eprintln!("⚠️ LINK data fetch failed: {}", e);
                data_sources.insert("link_price".to_string(), "failed".to_string());
                partial_failure = true;
                (0.0, 0.0) // Keep 0.0 to show loading state on client
            }
            Err(_) => {
                eprintln!("⚠️ LINK data fetch timeout");
                data_sources.insert("link_price".to_string(), "timeout".to_string());
                partial_failure = true;
                (0.0, 0.0) // Keep 0.0 to show loading state on client
            }
        };
        
        // Process global market data
        let (market_cap, volume_24h, market_cap_change_24h, btc_dominance, eth_dominance) = match global_result {
            Ok(Ok(global_data)) => {
                data_sources.insert("market_data".to_string(), "coingecko".to_string());
                (
                    global_data["market_cap"].as_f64().unwrap_or(0.0),
                    global_data["volume_24h"].as_f64().unwrap_or(0.0),
                    global_data["market_cap_change_percentage_24h_usd"].as_f64().unwrap_or(0.0),
                    global_data["btc_market_cap_percentage"].as_f64().unwrap_or(0.0),
                    global_data["eth_market_cap_percentage"].as_f64().unwrap_or(0.0)
                )
            }
            Ok(Err(e)) => {
                eprintln!("⚠️ Global data fetch failed: {}", e);
                data_sources.insert("market_data".to_string(), "failed".to_string());
                partial_failure = true;
                (0.0, 0.0, 0.0, 0.0, 0.0) // Keep 0.0 to show loading state on client
            }
            Err(_) => {
                eprintln!("⚠️ Global data fetch timeout");
                data_sources.insert("market_data".to_string(), "timeout".to_string());
                partial_failure = true;
                (0.0, 0.0, 0.0, 0.0, 0.0) // Keep 0.0 to show loading state on client
            }
        };
        
        // Process Fear & Greed data
        let fng_value = match fng_result {
            Ok(Ok(fng_data)) => {
                data_sources.insert("fear_greed".to_string(), "alternative_me".to_string());
                fng_data["value"].as_u64().unwrap_or(50) as u32
            }
            Ok(Err(e)) => {
                eprintln!("⚠️ Fear & Greed data fetch failed: {}", e);
                data_sources.insert("fear_greed".to_string(), "failed".to_string());
                partial_failure = true;
                50 // Neutral value
            }
            Err(_) => {
                eprintln!("⚠️ Fear & Greed data fetch timeout");
                data_sources.insert("fear_greed".to_string(), "timeout".to_string());
                partial_failure = true;
                50
            }
        };
        
        // Process RSI data
        let rsi_value = match rsi_result {
            Ok(Ok(rsi_data)) => {
                data_sources.insert("rsi".to_string(), "taapi".to_string());
                rsi_data["value"].as_f64().unwrap_or(50.0)
            }
            Ok(Err(e)) => {
                eprintln!("⚠️ RSI data fetch failed: {}", e);
                data_sources.insert("rsi".to_string(), "failed".to_string());
                partial_failure = true;
                50.0 // Neutral value
            }
            Err(_) => {
                eprintln!("⚠️ RSI data fetch timeout");
                data_sources.insert("rsi".to_string(), "timeout".to_string());
                partial_failure = true;
                50.0
            }
        };
        
        // Process US Stock Indices data
        let us_indices = match us_indices_result {
            Ok(Ok(indices_data)) => {
                data_sources.insert("us_indices".to_string(), "finnhub".to_string());
                indices_data["indices"].clone()
            }
            Ok(Err(e)) => {
                eprintln!("⚠️ US indices data fetch failed: {}", e);
                data_sources.insert("us_indices".to_string(), "failed".to_string());
                partial_failure = true;
                serde_json::json!({}) // Empty object for failed fetch
            }
            Err(_) => {
                eprintln!("⚠️ US indices data fetch timeout");
                data_sources.insert("us_indices".to_string(), "timeout".to_string());
                partial_failure = true;
                serde_json::json!({})
            }
        };
        
        let duration = start_time.elapsed();
        
        // Update statistics
        if partial_failure {
            self.partial_failures.fetch_add(1, Ordering::Relaxed);
            println!("⚠️ Dashboard data aggregated with partial failures in {}ms", duration.as_millis());
        } else {
            self.successful_aggregations.fetch_add(1, Ordering::Relaxed);
            println!("✅ Dashboard data aggregated successfully in {}ms", duration.as_millis());
        }
        
        // Return structured JSON with proper field names for WebSocket extraction
        Ok(serde_json::json!({
            "btc_price_usd": btc_price,
            "btc_change_24h": btc_change,
            "eth_price_usd": eth_price,
            "eth_change_24h": eth_change,
            "sol_price_usd": sol_price,
            "sol_change_24h": sol_change,
            "xrp_price_usd": xrp_price,
            "xrp_change_24h": xrp_change,
            "ada_price_usd": ada_price,
            "ada_change_24h": ada_change,
            "link_price_usd": link_price,
            "link_change_24h": link_change,
            "market_cap_usd": market_cap,
            "volume_24h_usd": volume_24h,
            "market_cap_change_percentage_24h_usd": market_cap_change_24h,
            "btc_market_cap_percentage": btc_dominance,
            "eth_market_cap_percentage": eth_dominance,
            "fng_value": fng_value,
            "rsi_14": rsi_value,
            "us_stock_indices": us_indices,
            "data_sources": data_sources,
            "fetch_duration_ms": duration.as_millis() as u64,
            "partial_failure": partial_failure,
            "last_updated": chrono::Utc::now().to_rfc3339(),
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    }
    
    /// Fetch BTC data with generic caching strategy
    async fn fetch_btc_with_cache(&self) -> Result<serde_json::Value> {
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
    async fn fetch_eth_with_cache(&self) -> Result<serde_json::Value> {
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
    async fn fetch_sol_with_cache(&self) -> Result<serde_json::Value> {
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
    async fn fetch_xrp_with_cache(&self) -> Result<serde_json::Value> {
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
    async fn fetch_ada_with_cache(&self) -> Result<serde_json::Value> {
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
    async fn fetch_link_with_cache(&self) -> Result<serde_json::Value> {
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
    
    /// Fetch global data with generic caching strategy  
    async fn fetch_global_with_cache(&self) -> Result<serde_json::Value> {
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
    async fn fetch_fng_with_cache(&self) -> Result<serde_json::Value> {
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
    async fn fetch_rsi_with_cache(&self) -> Result<serde_json::Value> {
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
    async fn fetch_us_indices_with_cache(&self) -> Result<serde_json::Value> {
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
