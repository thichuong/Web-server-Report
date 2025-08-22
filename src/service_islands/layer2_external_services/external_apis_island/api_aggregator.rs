//! API Aggregator Component
//! 
//! This component aggregates data from multiple APIs and handles coordination between different data sources.

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::Result;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::time::{timeout, Duration};
use futures::FutureExt;
use super::MarketDataApi;
use crate::service_islands::layer1_infrastructure::CacheSystemIsland;

/// API Aggregator
/// 
/// Coordinates data fetching from multiple APIs and provides unified dashboard data with individual caching.
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
    /// Create a new ApiAggregator with cache integration
    pub async fn new(taapi_secret: String) -> Result<Self> {
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
        let market_api = Arc::new(MarketDataApi::new(taapi_secret).await?);
        
        Ok(Self {
            market_api,
            client,
            cache_system: None, // Will be set by with_cache method
            total_aggregations: Arc::new(AtomicUsize::new(0)),
            successful_aggregations: Arc::new(AtomicUsize::new(0)),
            partial_failures: Arc::new(AtomicUsize::new(0)),
        })
    }
    
    /// Create a new ApiAggregator in cache-free mode (NEW - Phase 2)
    pub async fn new_cache_free(taapi_secret: String) -> Result<Self> {
        println!("📊 [V2] Initializing API Aggregator (cache-free mode)...");
        Self::new(taapi_secret).await
    }
    
    /// Create ApiAggregator with cache system
    pub async fn with_cache(taapi_secret: String, cache_system: Arc<CacheSystemIsland>) -> Result<Self> {
        let mut aggregator = Self::new(taapi_secret).await?;
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
    
    /// Test aggregation functionality
    async fn test_aggregation(&self) -> Result<()> {
        // Simple test to verify API coordination is working with rate limit handling
        timeout(Duration::from_secs(10), async {
            // Try with exponential backoff for rate limiting
            let mut attempts = 0;
            let max_attempts = 3;
            
            while attempts < max_attempts {
                match self.market_api.fetch_btc_price().await {
                    Ok(_) => return Ok(()),
                    Err(e) => {
                        let error_str = e.to_string();
                        if error_str.contains("429") || error_str.contains("Too Many Requests") {
                            attempts += 1;
                            let delay = Duration::from_millis(1000 * (2_u64.pow(attempts)));
                            println!("⚠️ Rate limit hit, retrying in {:?} (attempt {}/{})", delay, attempts, max_attempts);
                            tokio::time::sleep(delay).await;
                            continue;
                        } else {
                            return Err(e);
                        }
                    }
                }
            }
            
            Err(anyhow::anyhow!("Max retry attempts reached due to rate limiting"))
        }).await
        .map_err(|_| anyhow::anyhow!("Aggregation test timeout"))?
    }
    
    /// Fetch comprehensive dashboard data by aggregating multiple APIs
    pub async fn fetch_dashboard_data(&self) -> Result<serde_json::Value> {
        let start_time = std::time::Instant::now();
        self.total_aggregations.fetch_add(1, Ordering::Relaxed);
        
        println!("🔄 Starting dashboard data aggregation...");
        
        // Fetch data from multiple sources concurrently with timeouts and individual caching
        let btc_future = timeout(Duration::from_secs(10), self.fetch_btc_with_cache());
        let global_future = timeout(Duration::from_secs(10), self.fetch_global_with_cache());
        let fng_future = timeout(Duration::from_secs(10), self.fetch_fng_with_cache());
        let rsi_future = timeout(Duration::from_secs(10), self.fetch_rsi_with_cache());
        
        let (btc_result, global_result, fng_result, rsi_result) = tokio::join!(
            btc_future,
            global_future,
            fng_future,
            rsi_future
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
        
        // Process global market data
        let (market_cap, volume_24h) = match global_result {
            Ok(Ok(global_data)) => {
                data_sources.insert("market_data".to_string(), "coingecko".to_string());
                (
                    global_data["market_cap"].as_f64().unwrap_or(0.0),
                    global_data["volume_24h"].as_f64().unwrap_or(0.0)
                )
            }
            Ok(Err(e)) => {
                eprintln!("⚠️ Global data fetch failed: {}", e);
                data_sources.insert("market_data".to_string(), "failed".to_string());
                partial_failure = true;
                (0.0, 0.0) // Keep 0.0 to show loading state on client
            }
            Err(_) => {
                eprintln!("⚠️ Global data fetch timeout");
                data_sources.insert("market_data".to_string(), "timeout".to_string());
                partial_failure = true;
                (0.0, 0.0) // Keep 0.0 to show loading state on client
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
            "market_cap_usd": market_cap,
            "volume_24h_usd": volume_24h,
            "fng_value": fng_value,
            "rsi_14": rsi_value,
            "data_sources": data_sources,
            "fetch_duration_ms": duration.as_millis() as u64,
            "partial_failure": partial_failure,
            "last_updated": chrono::Utc::now().to_rfc3339(),
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    }
    
    /// Fetch BTC data with generic caching strategy
    async fn fetch_btc_with_cache(&self) -> Result<serde_json::Value> {
        let cache_key = "btc_coingecko_30s";
        
        // Try cache first
        if let Some(ref cache) = self.cache_system {
            if let Ok(Some(cached_data)) = cache.cache_manager.get(cache_key).await {
                return Ok(cached_data);
            }
        }
        
        // Fetch from API
        match self.market_api.fetch_btc_price().await {
            Ok(data) => {
                // Cache using generic ShortTerm strategy (5 minutes)
                if let Some(ref cache) = self.cache_system {
                    let _ = cache.cache_manager.set_with_strategy(cache_key, data.clone(), 
                        crate::service_islands::layer1_infrastructure::cache_system_island::cache_manager::CacheStrategy::ShortTerm).await;
                    println!("💾 BTC price cached for 5 minutes (short-term strategy)");
                }
                Ok(data)
            }
            Err(e) => Err(e)
        }
    }
    
    /// Fetch global data with generic caching strategy  
    async fn fetch_global_with_cache(&self) -> Result<serde_json::Value> {
        let cache_key = "global_coingecko_30s";
        
        // Try cache first
        if let Some(ref cache) = self.cache_system {
            if let Ok(Some(cached_data)) = cache.cache_manager.get(cache_key).await {
                return Ok(cached_data);
            }
        }
        
        // Fetch from API
        match self.market_api.fetch_global_data().await {
            Ok(data) => {
                // Cache using generic RealTime strategy (30 seconds)
                if let Some(ref cache) = self.cache_system {
                    let _ = cache.cache_manager.set_with_strategy(cache_key, data.clone(),
                        crate::service_islands::layer1_infrastructure::cache_system_island::cache_manager::CacheStrategy::RealTime).await;
                    println!("💾 Global data cached for 30 seconds (real-time strategy)");
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
    
    /// Get API statistics
    pub async fn get_statistics(&self) -> serde_json::Value {
        let total = self.total_aggregations.load(Ordering::Relaxed);
        let successful = self.successful_aggregations.load(Ordering::Relaxed);
        let partial = self.partial_failures.load(Ordering::Relaxed);
        
        let success_rate = if total > 0 {
            (successful as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        
        serde_json::json!({
            "total_aggregations": total,
            "successful_aggregations": successful,
            "partial_failures": partial,
            "success_rate_percent": success_rate,
            "last_updated": chrono::Utc::now().to_rfc3339()
        })
    }
    
    // ===== GENERIC CACHE HELPER METHODS =====
    
    /// Generic cache helper for any API data with custom key and strategy
    async fn cache_api_data<F, T>(&self, 
        cache_key: &str,
        strategy: crate::service_islands::layer1_infrastructure::cache_system_island::cache_manager::CacheStrategy,
        fetch_fn: F) -> Result<serde_json::Value>
    where
        F: std::future::Future<Output = Result<T>> + Send,
        T: serde::Serialize,
    {
        // Try cache first
        if let Some(ref cache) = self.cache_system {
            if let Ok(Some(cached_data)) = cache.cache_manager.get(cache_key).await {
                return Ok(cached_data);
            }
        }
        
        // Fetch from API
        match fetch_fn.await {
            Ok(data) => {
                let json_data = serde_json::to_value(data)?;
                // Cache using provided strategy
                if let Some(ref cache) = self.cache_system {
                    let _ = cache.cache_manager.set_with_strategy(cache_key, json_data.clone(), strategy.clone()).await;
                    println!("💾 Cached '{}' with strategy: {:?}", cache_key, strategy);
                }
                Ok(json_data)
            }
            Err(e) => Err(e)
        }
    }
    
    /// Wrapper for BTC data using generic cache helper
    async fn fetch_btc_generic(&self) -> Result<serde_json::Value> {
        self.cache_api_data(
            "btc_coingecko_30s",
            crate::service_islands::layer1_infrastructure::cache_system_island::cache_manager::CacheStrategy::ShortTerm,
            self.market_api.fetch_btc_price()
        ).await
    }
    
    /// Wrapper for RSI data using generic cache helper
    async fn fetch_rsi_generic(&self) -> Result<serde_json::Value> {
        self.cache_api_data(
            "rsi_taapi_3h", 
            crate::service_islands::layer1_infrastructure::cache_system_island::cache_manager::CacheStrategy::LongTerm,
            self.market_api.fetch_rsi()
        ).await
    }

    // ===== NEW CACHE-FREE METHODS (Phase 2 Refactoring) =====
    
    /// Fetch dashboard data without cache logic (NEW - Cache-free)
    /// 
    /// Pure API aggregation without cache management.
    /// Layer 1 handles all caching after this method returns.
    pub async fn fetch_dashboard_data_v2(&self) -> Result<serde_json::Value> {
        let start_time = std::time::Instant::now();
        self.total_aggregations.fetch_add(1, Ordering::Relaxed);
        
        println!("🔄 [V2] Starting dashboard data aggregation (cache-free)...");
        
        // Fetch data from multiple sources concurrently WITHOUT caching
        let btc_future = timeout(Duration::from_secs(10), self.fetch_btc_direct());
        let global_future = timeout(Duration::from_secs(10), self.fetch_global_direct());
        let fng_future = timeout(Duration::from_secs(10), self.fetch_fng_direct());
        let rsi_future = timeout(Duration::from_secs(10), self.fetch_rsi_direct());
        
        let (btc_result, global_result, fng_result, rsi_result) = tokio::join!(
            btc_future,
            global_future,
            fng_future,
            rsi_future
        );
        
        let mut data_sources = HashMap::new();
        let mut partial_failure = false;
        
        // Process results without cache logic
        let (btc_price, btc_change) = match btc_result {
            Ok(Ok(btc_data)) => {
                data_sources.insert("btc_price".to_string(), "coingecko_direct".to_string());
                (
                    btc_data["price_usd"].as_f64().unwrap_or(0.0),
                    btc_data["price_change_24h"].as_f64().unwrap_or(0.0)
                )
            }
            _ => {
                partial_failure = true;
                (0.0, 0.0)
            }
        };
        
        // Build aggregated response
        let (market_cap, volume_24h) = match global_result {
            Ok(Ok(ref data)) => (
                data["total_market_cap"]["usd"].as_f64().unwrap_or(0.0),
                data["total_volume"]["usd"].as_f64().unwrap_or(0.0)
            ),
            _ => {
                partial_failure = true;
                (0.0, 0.0)
            }
        };
        
        let aggregated_data = serde_json::json!({
            "btc_price_usd": btc_price,
            "btc_change_24h": btc_change,
            "market_cap_usd": market_cap,
            "volume_24h_usd": volume_24h,
            "fng_value": match fng_result {
                Ok(Ok(data)) => data["data"][0]["value"].as_str().unwrap_or("50").parse::<i32>().unwrap_or(50),
                _ => { partial_failure = true; 50 }
            },
            "rsi_14": match rsi_result {
                Ok(Ok(data)) => data["value"].as_f64().unwrap_or(50.0),
                _ => { partial_failure = true; 50.0 }
            },
            "data_sources": data_sources,
            "aggregation_time_ms": start_time.elapsed().as_millis(),
            "partial_failure": partial_failure,
            "mode": "cache_free_v2",
            "timestamp": chrono::Utc::now().to_rfc3339()
        });
        
        if partial_failure {
            self.partial_failures.fetch_add(1, Ordering::Relaxed);
            println!("⚠️ [V2] Dashboard aggregation completed with partial failures");
        } else {
            self.successful_aggregations.fetch_add(1, Ordering::Relaxed);
            println!("✅ [V2] Dashboard aggregation completed successfully - ready for Layer 1");
        }
        
        Ok(aggregated_data)
    }
    
    /// Direct BTC fetch without cache
    async fn fetch_btc_direct(&self) -> Result<serde_json::Value> {
        println!("  📈 [V2] Fetching BTC data directly...");
        self.market_api.fetch_btc_price().await
    }
    
    /// Direct global market fetch without cache  
    async fn fetch_global_direct(&self) -> Result<serde_json::Value> {
        println!("  🌍 [V2] Fetching global market data directly...");
        self.market_api.fetch_global_data().await
    }
    
    /// Direct fear & greed fetch without cache
    async fn fetch_fng_direct(&self) -> Result<serde_json::Value> {
        println!("  😨😤 [V2] Fetching Fear & Greed Index directly...");
        self.market_api.fetch_fear_greed_index().await
    }
    
    /// Direct RSI fetch without cache
    async fn fetch_rsi_direct(&self) -> Result<serde_json::Value> {
        println!("  📊 [V2] Fetching RSI data directly...");
        self.market_api.fetch_rsi().await
    }
    
    /// Check if running in cache-free mode
    pub fn is_cache_free(&self) -> bool {
        self.cache_system.is_none()
    }
}
