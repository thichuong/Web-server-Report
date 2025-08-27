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
// CoinGecko APIs (Primary)
const BASE_GLOBAL_URL: &str = "https://api.coingecko.com/api/v3/global"; // 30 sec cache
const BASE_BTC_PRICE_URL: &str = "https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=usd&include_24hr_change=true"; // 30 sec cache

// CoinMarketCap APIs (Fallback)
const CMC_GLOBAL_URL: &str = "https://pro-api.coinmarketcap.com/v1/global-metrics/quotes/latest"; // 30 sec cache
const CMC_BTC_PRICE_URL: &str = "https://pro-api.coinmarketcap.com/v1/cryptocurrency/quotes/latest?symbol=BTC"; // 30 sec cache

// Other APIs
const BASE_FNG_URL: &str = "https://api.alternative.me/fng/?limit=1"; // 5 min cache
const BASE_RSI_URL_TEMPLATE: &str = "https://api.taapi.io/rsi?secret={secret}&exchange=binance&symbol=BTC/USDT&interval=1d"; // 5 min cache

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DashboardSummary {
    pub market_cap: f64,
    pub volume_24h: f64,
    pub market_cap_change_percentage_24h_usd: f64,
    pub btc_price_usd: f64,
    pub btc_change_24h: f64,
    pub btc_market_cap_percentage: f64,
    pub eth_market_cap_percentage: f64,
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
    market_cap_change_percentage_24h_usd: f64,
    market_cap_percentage: HashMap<String, f64>,
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

// CoinMarketCap response structures
#[derive(Debug, Deserialize)]
struct CmcGlobalResponse {
    data: CmcGlobalData,
}

#[derive(Debug, Deserialize)]
struct CmcGlobalData {
    quote: HashMap<String, CmcGlobalQuote>,
}

#[derive(Debug, Deserialize)]
struct CmcGlobalQuote {
    total_market_cap: f64,
    total_volume_24h: f64,
    market_cap_change_percentage_24h: f64,
    btc_dominance: f64,
    eth_dominance: f64,
}

#[derive(Debug, Deserialize)]
struct CmcBtcResponse {
    data: HashMap<String, Vec<CmcBtcData>>,
}

#[derive(Debug, Deserialize)]
struct CmcBtcData {
    quote: HashMap<String, CmcBtcQuote>,
}

#[derive(Debug, Deserialize)]
struct CmcBtcQuote {
    price: f64,
    percent_change_24h: f64,
}

/// Market Data API
/// 
/// Handles direct API calls to cryptocurrency data sources.
pub struct MarketDataApi {
    client: Client,
    taapi_secret: String,
    cmc_api_key: Option<String>,
    // Statistics tracking
    api_calls_count: Arc<AtomicUsize>,
    successful_calls: Arc<AtomicUsize>,
    failed_calls: Arc<AtomicUsize>,
    last_call_timestamp: Arc<AtomicU64>,
}

impl MarketDataApi {
    /// Create a new MarketDataApi
    #[allow(dead_code)]
    pub async fn new(taapi_secret: String) -> Result<Self> {
        Self::with_cmc_key(taapi_secret, None).await
    }
    
    /// Create a new MarketDataApi with CoinMarketCap API key
    pub async fn with_cmc_key(taapi_secret: String, cmc_api_key: Option<String>) -> Result<Self> {
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
            cmc_api_key,
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
        
        // Try CoinGecko first
        match self.fetch_btc_price_coingecko().await {
            Ok(data) => {
                self.record_success();
                Ok(data)
            }
            Err(e) => {
                println!("⚠️ CoinGecko BTC price failed: {}, trying CoinMarketCap...", e);
                // Fallback to CoinMarketCap
                match self.fetch_btc_price_cmc().await {
                    Ok(data) => {
                        self.record_success();
                        Ok(data)
                    }
                    Err(cmc_err) => {
                        self.record_failure();
                        println!("❌ Both CoinGecko and CoinMarketCap failed for BTC price");
                        Err(anyhow::anyhow!("Primary error: {}. Fallback error: {}", e, cmc_err))
                    }
                }
            }
        }
    }
    
    /// Fetch Bitcoin price from CoinGecko
    async fn fetch_btc_price_coingecko(&self) -> Result<serde_json::Value> {
        let result = self.fetch_with_retry(BASE_BTC_PRICE_URL, |response_data: CoinGeckoBtcPrice| {
            serde_json::json!({
                "price_usd": response_data.bitcoin.usd,
                "change_24h": response_data.bitcoin.usd_24h_change,
                "source": "coingecko",
                "last_updated": chrono::Utc::now().to_rfc3339()
            })
        }).await?;
        
        // Post-processing validation: check if we got meaningful data
        let price_usd = result.get("price_usd").and_then(|v| v.as_f64()).unwrap_or(0.0);
        
        // Critical validation: Bitcoin price must be positive and reasonable
        if price_usd <= 0.0 || price_usd > 1_000_000.0 { // Basic sanity check
            return Err(anyhow::anyhow!(
                "CoinGecko Bitcoin price validation failed: price_usd={}", 
                price_usd
            ));
        }
        
        Ok(result)
    }
    
    /// Fetch Bitcoin price from CoinMarketCap
    async fn fetch_btc_price_cmc(&self) -> Result<serde_json::Value> {
        let cmc_key = self.cmc_api_key.as_ref()
            .ok_or_else(|| anyhow::anyhow!("CoinMarketCap API key not provided"))?;
        
        let mut attempts = 0;
        let max_attempts = 3;
        
        while attempts < max_attempts {
            let response = self.client
                .get(CMC_BTC_PRICE_URL)
                .header("X-CMC_PRO_API_KEY", cmc_key)
                .header("Accept", "application/json")
                .send()
                .await?;
            
            match response.status() {
                status if status.is_success() => {
                    let cmc_data: CmcBtcResponse = response.json().await?;
                    
                    if let Some(btc_data) = cmc_data.data.get("BTC").and_then(|v| v.first()) {
                        if let Some(usd_quote) = btc_data.quote.get("USD") {
                            return Ok(serde_json::json!({
                                "price_usd": usd_quote.price,
                                "change_24h": usd_quote.percent_change_24h,
                                "source": "coinmarketcap",
                                "last_updated": chrono::Utc::now().to_rfc3339()
                            }));
                        }
                    }
                    return Err(anyhow::anyhow!("Invalid CoinMarketCap BTC response structure"));
                }
                status if status == 429 => {
                    attempts += 1;
                    if attempts >= max_attempts {
                        return Err(anyhow::anyhow!("CoinMarketCap rate limit exceeded after {} attempts", max_attempts));
                    }
                    
                    let delay = std::time::Duration::from_millis(1000 * (2_u64.pow(attempts)));
                    println!("⚠️ CoinMarketCap rate limit (429), retrying in {:?} (attempt {}/{})", delay, attempts, max_attempts);
                    tokio::time::sleep(delay).await;
                    continue;
                }
                status => {
                    return Err(anyhow::anyhow!("CoinMarketCap BTC API returned status: {}", status));
                }
            }
        }
        
        Err(anyhow::anyhow!("CoinMarketCap BTC API max retry attempts reached"))
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
        
        // Try CoinGecko first
        match self.fetch_global_data_coingecko().await {
            Ok(data) => {
                self.record_success();
                Ok(data)
            }
            Err(e) => {
                println!("⚠️ CoinGecko global data failed: {}, trying CoinMarketCap...", e);
                // Fallback to CoinMarketCap
                match self.fetch_global_data_cmc().await {
                    Ok(data) => {
                        self.record_success();
                        Ok(data)
                    }
                    Err(cmc_err) => {
                        self.record_failure();
                        println!("❌ Both CoinGecko and CoinMarketCap failed for global data");
                        Err(anyhow::anyhow!("Primary error: {}. Fallback error: {}", e, cmc_err))
                    }
                }
            }
        }
    }
    
    /// Fetch global data from CoinGecko
    async fn fetch_global_data_coingecko(&self) -> Result<serde_json::Value> {
        let result = self.fetch_with_retry(BASE_GLOBAL_URL, |global_data: CoinGeckoGlobal| {
            let market_cap = global_data.data.total_market_cap.get("usd").copied().unwrap_or(0.0);
            let volume_24h = global_data.data.total_volume.get("usd").copied().unwrap_or(0.0);
            let market_cap_change_24h = global_data.data.market_cap_change_percentage_24h_usd;
            let btc_dominance = global_data.data.market_cap_percentage.get("btc").copied().unwrap_or(0.0);
            let eth_dominance = global_data.data.market_cap_percentage.get("eth").copied().unwrap_or(0.0);
            
            serde_json::json!({
                "market_cap": market_cap,
                "volume_24h": volume_24h,
                "market_cap_change_percentage_24h_usd": market_cap_change_24h,
                "btc_market_cap_percentage": btc_dominance,
                "eth_market_cap_percentage": eth_dominance,
                "source": "coingecko",
                "last_updated": chrono::Utc::now().to_rfc3339()
            })
        }).await?;
        
        // Post-processing validation: check if we got meaningful data
        let market_cap = result.get("market_cap").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let volume_24h = result.get("volume_24h").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let btc_dominance = result.get("btc_market_cap_percentage").and_then(|v| v.as_f64()).unwrap_or(0.0);
        
        // Critical validation: if any essential data is missing or invalid, return error
        if market_cap <= 0.0 || volume_24h <= 0.0 || btc_dominance <= 0.0 {
            return Err(anyhow::anyhow!(
                "CoinGecko data validation failed: market_cap={}, volume_24h={}, btc_dominance={}", 
                market_cap, volume_24h, btc_dominance
            ));
        }
        
        Ok(result)
    }
    
    /// Fetch global data from CoinMarketCap
    async fn fetch_global_data_cmc(&self) -> Result<serde_json::Value> {
        let cmc_key = self.cmc_api_key.as_ref()
            .ok_or_else(|| anyhow::anyhow!("CoinMarketCap API key not provided"))?;
        
        let mut attempts = 0;
        let max_attempts = 3;
        
        while attempts < max_attempts {
            let response = self.client
                .get(CMC_GLOBAL_URL)
                .header("X-CMC_PRO_API_KEY", cmc_key)
                .header("Accept", "application/json")
                .send()
                .await?;
            
            match response.status() {
                status if status.is_success() => {
                    let cmc_data: CmcGlobalResponse = response.json().await?;
                    
                    if let Some(usd_quote) = cmc_data.data.quote.get("USD") {
                        return Ok(serde_json::json!({
                            "market_cap": usd_quote.total_market_cap,
                            "volume_24h": usd_quote.total_volume_24h,
                            "market_cap_change_percentage_24h_usd": usd_quote.market_cap_change_percentage_24h,
                            "btc_market_cap_percentage": usd_quote.btc_dominance,
                            "eth_market_cap_percentage": usd_quote.eth_dominance,
                            "source": "coinmarketcap",
                            "last_updated": chrono::Utc::now().to_rfc3339()
                        }));
                    }
                    return Err(anyhow::anyhow!("Invalid CoinMarketCap global response structure"));
                }
                status if status == 429 => {
                    attempts += 1;
                    if attempts >= max_attempts {
                        return Err(anyhow::anyhow!("CoinMarketCap global API rate limit exceeded after {} attempts", max_attempts));
                    }
                    
                    let delay = std::time::Duration::from_millis(1000 * (2_u64.pow(attempts)));
                    println!("⚠️ CoinMarketCap global API rate limit (429), retrying in {:?} (attempt {}/{})", delay, attempts, max_attempts);
                    tokio::time::sleep(delay).await;
                    continue;
                }
                status => {
                    return Err(anyhow::anyhow!("CoinMarketCap global API returned status: {}", status));
                }
            }
        }
        
        Err(anyhow::anyhow!("CoinMarketCap global API max retry attempts reached"))
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
    #[allow(dead_code)]
    pub fn get_api_stats(&self) -> serde_json::Value {
        let total_calls = self.api_calls_count.load(Ordering::Relaxed);
        let successful_calls = self.successful_calls.load(Ordering::Relaxed);
        let failed_calls = self.failed_calls.load(Ordering::Relaxed);
        let last_call = self.last_call_timestamp.load(Ordering::Relaxed);
        
        serde_json::json!({
            "total_api_calls": total_calls,
            "successful_calls": successful_calls,
            "failed_calls": failed_calls,
            "success_rate": if total_calls > 0 { 
                (successful_calls as f64 / total_calls as f64 * 100.0).round() 
            } else { 
                0.0 
            },
            "last_call_timestamp": last_call,
            "has_coinmarketcap_key": self.cmc_api_key.is_some()
        })
    }
}
