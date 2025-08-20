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

/// Aggregated dashboard data
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AggregatedDashboardData {
    pub market_cap: f64,
    pub volume_24h: f64,
    pub btc_price_usd: f64,
    pub btc_change_24h: f64,
    pub fng_value: u32,
    pub rsi_14: f64,
    pub data_sources: HashMap<String, String>,
    pub fetch_duration_ms: u64,
    pub partial_failure: bool,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// API Aggregator
/// 
/// Coordinates data fetching from multiple APIs and provides unified dashboard data.
pub struct ApiAggregator {
    market_api: Arc<MarketDataApi>,
    client: Client,
    // Statistics
    total_aggregations: Arc<AtomicUsize>,
    successful_aggregations: Arc<AtomicUsize>,
    partial_failures: Arc<AtomicUsize>,
}

impl ApiAggregator {
    /// Create a new ApiAggregator
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
            total_aggregations: Arc::new(AtomicUsize::new(0)),
            successful_aggregations: Arc::new(AtomicUsize::new(0)),
            partial_failures: Arc::new(AtomicUsize::new(0)),
        })
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
        // Simple test to verify API coordination is working
        timeout(Duration::from_secs(5), async {
            self.market_api.fetch_btc_price().await
        }).await
        .map_err(|_| anyhow::anyhow!("Aggregation test timeout"))?
        .map(|_| ())
    }
    
    /// Fetch comprehensive dashboard data by aggregating multiple APIs
    pub async fn fetch_dashboard_data(&self) -> Result<serde_json::Value> {
        let start_time = std::time::Instant::now();
        self.total_aggregations.fetch_add(1, Ordering::Relaxed);
        
        println!("🔄 Starting dashboard data aggregation...");
        
        // Fetch data from multiple sources concurrently with timeouts
        let btc_future = timeout(Duration::from_secs(10), self.market_api.fetch_btc_price());
        let global_future = timeout(Duration::from_secs(10), self.market_api.fetch_global_data());
        let fng_future = timeout(Duration::from_secs(10), self.market_api.fetch_fear_greed_index());
        let rsi_future = timeout(Duration::from_secs(10), self.market_api.fetch_rsi());
        
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
                (0.0, 0.0)
            }
            Err(_) => {
                eprintln!("⚠️ BTC data fetch timeout");
                data_sources.insert("btc_price".to_string(), "timeout".to_string());
                partial_failure = true;
                (0.0, 0.0)
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
                (0.0, 0.0)
            }
            Err(_) => {
                eprintln!("⚠️ Global data fetch timeout");
                data_sources.insert("market_data".to_string(), "timeout".to_string());
                partial_failure = true;
                (0.0, 0.0)
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
        
        let fetch_duration = start_time.elapsed().as_millis() as u64;
        
        // Create aggregated response
        let aggregated_data = AggregatedDashboardData {
            market_cap,
            volume_24h,
            btc_price_usd: btc_price,
            btc_change_24h: btc_change,
            fng_value,
            rsi_14: rsi_value,
            data_sources,
            fetch_duration_ms: fetch_duration,
            partial_failure,
            last_updated: chrono::Utc::now(),
        };
        
        // Update statistics
        if partial_failure {
            self.partial_failures.fetch_add(1, Ordering::Relaxed);
            println!("⚠️ Dashboard data aggregated with partial failures in {}ms", fetch_duration);
        } else {
            self.successful_aggregations.fetch_add(1, Ordering::Relaxed);
            println!("✅ Dashboard data aggregated successfully in {}ms", fetch_duration);
        }
        
        Ok(serde_json::to_value(aggregated_data)?)
    }
    
    /// Fetch optimized BTC price (real-time)
    pub async fn fetch_btc_price_optimized(&self) -> Result<serde_json::Value> {
        self.total_aggregations.fetch_add(1, Ordering::Relaxed);
        
        // Single API call for BTC price with short timeout
        match timeout(Duration::from_secs(5), self.market_api.fetch_btc_price()).await {
            Ok(Ok(btc_data)) => {
                self.successful_aggregations.fetch_add(1, Ordering::Relaxed);
                Ok(btc_data)
            }
            Ok(Err(e)) => {
                self.partial_failures.fetch_add(1, Ordering::Relaxed);
                Err(e)
            }
            Err(_) => {
                self.partial_failures.fetch_add(1, Ordering::Relaxed);
                Err(anyhow::anyhow!("BTC price fetch timeout"))
            }
        }
    }
    
    /// Fetch non-critical data (Fear & Greed, RSI) with longer timeouts
    pub async fn fetch_supplementary_data(&self) -> Result<serde_json::Value> {
        self.total_aggregations.fetch_add(1, Ordering::Relaxed);
        
        let fng_future = timeout(Duration::from_secs(15), self.market_api.fetch_fear_greed_index());
        let rsi_future = timeout(Duration::from_secs(15), self.market_api.fetch_rsi());
        
        let (fng_result, rsi_result) = tokio::join!(
            fng_future,
            rsi_future
        );
        
        let mut partial_failure = false;
        
        let fng_value = match fng_result {
            Ok(Ok(data)) => data["value"].as_u64().unwrap_or(50) as u32,
            _ => {
                partial_failure = true;
                50
            }
        };
        
        let rsi_value = match rsi_result {
            Ok(Ok(data)) => data["value"].as_f64().unwrap_or(50.0),
            _ => {
                partial_failure = true;
                50.0
            }
        };
        
        if partial_failure {
            self.partial_failures.fetch_add(1, Ordering::Relaxed);
        } else {
            self.successful_aggregations.fetch_add(1, Ordering::Relaxed);
        }
        
        Ok(serde_json::json!({
            "fear_greed_value": fng_value,
            "rsi_14": rsi_value,
            "partial_failure": partial_failure,
            "last_updated": chrono::Utc::now().to_rfc3339()
        }))
    }
    
    /// Get aggregator statistics
    pub async fn get_statistics(&self) -> serde_json::Value {
        let total = self.total_aggregations.load(Ordering::Relaxed);
        let successful = self.successful_aggregations.load(Ordering::Relaxed);
        let partial = self.partial_failures.load(Ordering::Relaxed);
        
        let success_rate = if total > 0 {
            (successful as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        
        let partial_failure_rate = if total > 0 {
            (partial as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        
        serde_json::json!({
            "total_aggregations": total,
            "successful_aggregations": successful,
            "partial_failures": partial,
            "success_rate_percent": success_rate,
            "partial_failure_rate_percent": partial_failure_rate,
            "component_status": "active"
        })
    }
}
