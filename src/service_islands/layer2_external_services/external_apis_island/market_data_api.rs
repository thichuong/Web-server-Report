//! Market Data API Component
//! 
//! This component handles direct interactions with cryptocurrency APIs.

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::Result;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

// API URLs - extracted from existing data_service.rs with cache-friendly grouping
const BASE_GLOBAL_URL: &str = "https://api.coingecko.com/api/v3/global"; // 30 sec cache
const BASE_BTC_PRICE_URL: &str = "https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=usd&include_24hr_change=true"; // 30 sec cache
const BASE_FNG_URL: &str = "https://api.alternative.me/fng/?limit=1"; // 5 min cache
const BASE_RSI_URL_TEMPLATE: &str = "https://api.taapi.io/rsi?secret={secret}&exchange=binance&symbol=BTC/USDT&interval=1d"; // 5 min cache

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DashboardSummary {
    pub market_cap: f64,
    pub volume_24h: f64,
    pub btc_price_usd: f64,
    pub btc_change_24h: f64,
    pub fng_value: u32,
    pub rsi_14: f64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
struct CoinGeckoGlobal {
    data: CoinGeckoGlobalData,
}

#[derive(Debug, Deserialize)]
struct CoinGeckoGlobalData {
    total_market_cap: HashMap<String, f64>,
    total_volume: HashMap<String, f64>,
}

#[derive(Debug, Deserialize)]
struct CoinGeckoBtcPrice {
    bitcoin: BtcPriceData,
}

#[derive(Debug, Deserialize)]
struct BtcPriceData {
    usd: f64,
    usd_24h_change: f64,
}

#[derive(Debug, Deserialize)]
struct FearGreedResponse {
    data: Vec<FearGreedData>,
}

#[derive(Debug, Deserialize)]
struct FearGreedData {
    value: String,
}

#[derive(Debug, Deserialize)]
struct TaapiRsiResponse {
    value: f64,
}

/// Market Data API
/// 
/// Handles direct API calls to cryptocurrency data sources.
pub struct MarketDataApi {
    client: Client,
    taapi_secret: String,
    // Statistics tracking
    api_calls_count: Arc<AtomicUsize>,
    successful_calls: Arc<AtomicUsize>,
    failed_calls: Arc<AtomicUsize>,
    last_call_timestamp: Arc<AtomicU64>,
}

impl MarketDataApi {
    /// Create a new MarketDataApi
    pub async fn new(taapi_secret: String) -> Result<Self> {
        println!("🌐 Initializing Market Data API...");
        
        // Use optimized HTTP client from performance module
        let client = if let Ok(perf_client) = std::panic::catch_unwind(|| {
            crate::performance::OPTIMIZED_HTTP_CLIENT.clone()
        }) {
            perf_client
        } else {
            // Fallback client if performance module not available
            Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()?
        };
        
        Ok(Self {
            client,
            taapi_secret,
            api_calls_count: Arc::new(AtomicUsize::new(0)),
            successful_calls: Arc::new(AtomicUsize::new(0)),
            failed_calls: Arc::new(AtomicUsize::new(0)),
            last_call_timestamp: Arc::new(AtomicU64::new(0)),
        })
    }
    
    /// Health check for Market Data API
    pub async fn health_check(&self) -> bool {
        match self.test_api_connectivity().await {
            Ok(_) => {
                println!("  ✅ Market Data API connectivity test passed");
                true
            }
            Err(e) => {
                let error_str = e.to_string();
                if error_str.contains("429") || error_str.contains("Too Many Requests") {
                    println!("  ⚠️ Market Data API health check: Rate limited, but service is available");
                    true // Rate limiting means API is working, just busy
                } else {
                    eprintln!("  ❌ Market Data API connectivity test failed: {}", e);
                    false
                }
            }
        }
    }
    
    /// Test API connectivity
    async fn test_api_connectivity(&self) -> Result<()> {
        // Simple test call to CoinGecko global endpoint
        let response = self.client
            .get("https://api.coingecko.com/api/v3/ping")
            .send()
            .await?;
        
        if response.status().is_success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("API connectivity test failed with status: {}", response.status()))
        }
    }
    
    /// Fetch Bitcoin price data
    pub async fn fetch_btc_price(&self) -> Result<serde_json::Value> {
        self.record_api_call();
        
        match self.fetch_btc_price_internal().await {
            Ok(data) => {
                self.record_success();
                Ok(data)
            }
            Err(e) => {
                self.record_failure();
                Err(e)
            }
        }
    }
    
    /// Internal Bitcoin price fetching with rate limiting
    async fn fetch_btc_price_internal(&self) -> Result<serde_json::Value> {
        self.fetch_with_retry(BASE_BTC_PRICE_URL, |response_data: CoinGeckoBtcPrice| {
            serde_json::json!({
                "price_usd": response_data.bitcoin.usd,
                "change_24h": response_data.bitcoin.usd_24h_change,
                "last_updated": chrono::Utc::now().to_rfc3339()
            })
        }).await
    }
    
    /// Generic fetch with retry logic for rate limiting
    async fn fetch_with_retry<T, F>(&self, url: &str, transformer: F) -> Result<serde_json::Value> 
    where
        T: for<'de> serde::Deserialize<'de>,
        F: Fn(T) -> serde_json::Value,
    {
        let mut attempts = 0;
        let max_attempts = 3;
        
        while attempts < max_attempts {
            let response = self.client
                .get(url)
                .send()
                .await?;
            
            match response.status() {
                status if status.is_success() => {
                    let data: T = response.json().await?;
                    return Ok(transformer(data));
                }
                status if status == 429 => {
                    // Rate limiting - implement exponential backoff
                    attempts += 1;
                    if attempts >= max_attempts {
                        return Err(anyhow::anyhow!("Rate limit exceeded after {} attempts for URL: {}", max_attempts, url));
                    }
                    
                    let delay = std::time::Duration::from_millis(1000 * (2_u64.pow(attempts)));
                    println!("⚠️ Rate limit (429) hit for {}, retrying in {:?} (attempt {}/{})", url, delay, attempts, max_attempts);
                    tokio::time::sleep(delay).await;
                    continue;
                }
                status => {
                    return Err(anyhow::anyhow!("API returned status: {} for URL: {}", status, url));
                }
            }
        }
        
        Err(anyhow::anyhow!("Max retry attempts reached for URL: {}", url))
    }
    
    /// Fetch global market data
    pub async fn fetch_global_data(&self) -> Result<serde_json::Value> {
        self.record_api_call();
        
        match self.fetch_global_data_internal().await {
            Ok(data) => {
                self.record_success();
                Ok(data)
            }
            Err(e) => {
                self.record_failure();
                Err(e)
            }
        }
    }
    
    /// Internal global data fetching
    async fn fetch_global_data_internal(&self) -> Result<serde_json::Value> {
        self.fetch_with_retry(BASE_GLOBAL_URL, |global_data: CoinGeckoGlobal| {
            let market_cap = global_data.data.total_market_cap.get("usd").copied().unwrap_or(0.0);
            let volume_24h = global_data.data.total_volume.get("usd").copied().unwrap_or(0.0);
            
            serde_json::json!({
                "market_cap": market_cap,
                "volume_24h": volume_24h,
                "last_updated": chrono::Utc::now().to_rfc3339()
            })
        }).await
    }
    
    /// Fetch Fear & Greed Index
    pub async fn fetch_fear_greed_index(&self) -> Result<serde_json::Value> {
        self.record_api_call();
        
        match self.fetch_fear_greed_internal().await {
            Ok(data) => {
                self.record_success();
                Ok(data)
            }
            Err(e) => {
                self.record_failure();
                Err(e)
            }
        }
    }
    
    /// Internal Fear & Greed fetching
    async fn fetch_fear_greed_internal(&self) -> Result<serde_json::Value> {
        self.fetch_with_retry(BASE_FNG_URL, |fng_data: FearGreedResponse| {
            let fng_value: u32 = fng_data
                .data
                .first()
                .and_then(|d| d.value.parse().ok())
                .unwrap_or(50); // Default neutral value
            
            serde_json::json!({
                "value": fng_value,
                "last_updated": chrono::Utc::now().to_rfc3339()
            })
        }).await
    }
    
    /// Fetch RSI data
    pub async fn fetch_rsi(&self) -> Result<serde_json::Value> {
        self.record_api_call();
        
        match self.fetch_rsi_internal().await {
            Ok(data) => {
                self.record_success();
                Ok(data)
            }
            Err(e) => {
                self.record_failure();
                Err(e)
            }
        }
    }
    
    /// Internal RSI fetching
    async fn fetch_rsi_internal(&self) -> Result<serde_json::Value> {
        let url = BASE_RSI_URL_TEMPLATE.replace("{secret}", &self.taapi_secret);
        
        // RSI uses a different approach because URL is dynamic
        let mut attempts = 0;
        let max_attempts = 3;
        
        while attempts < max_attempts {
            let response = self.client
                .get(&url)
                .send()
                .await?;
            
            match response.status() {
                status if status.is_success() => {
                    let rsi_data: TaapiRsiResponse = response.json().await?;
                    return Ok(serde_json::json!({
                        "value": rsi_data.value,
                        "period": "14",
                        "last_updated": chrono::Utc::now().to_rfc3339()
                    }));
                }
                status if status == 429 => {
                    attempts += 1;
                    if attempts >= max_attempts {
                        return Err(anyhow::anyhow!("RSI API rate limit exceeded after {} attempts", max_attempts));
                    }
                    
                    let delay = std::time::Duration::from_millis(1000 * (2_u64.pow(attempts)));
                    println!("⚠️ RSI API rate limit (429), retrying in {:?} (attempt {}/{})", delay, attempts, max_attempts);
                    tokio::time::sleep(delay).await;
                    continue;
                }
                status => {
                    return Err(anyhow::anyhow!("RSI API returned status: {}", status));
                }
            }
        }
        
        Err(anyhow::anyhow!("RSI API max retry attempts reached"))
    }
    
    /// Record API call
    fn record_api_call(&self) {
        self.api_calls_count.fetch_add(1, Ordering::Relaxed);
        self.last_call_timestamp.store(
            chrono::Utc::now().timestamp() as u64,
            Ordering::Relaxed
        );
    }
    
    /// Record successful API call
    fn record_success(&self) {
        self.successful_calls.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Record failed API call
    fn record_failure(&self) {
        self.failed_calls.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Get API statistics
    pub async fn get_statistics(&self) -> serde_json::Value {
        let total_calls = self.api_calls_count.load(Ordering::Relaxed);
        let successful = self.successful_calls.load(Ordering::Relaxed);
        let failed = self.failed_calls.load(Ordering::Relaxed);
        let last_call = self.last_call_timestamp.load(Ordering::Relaxed);
        
        let success_rate = if total_calls > 0 {
            (successful as f64 / total_calls as f64) * 100.0
        } else {
            0.0
        };
        
        serde_json::json!({
            "total_calls": total_calls,
            "successful_calls": successful,
            "failed_calls": failed,
            "success_rate_percent": success_rate,
            "last_call_timestamp": last_call,
            "uptime": "active"
        })
    }
}
