#![allow(dead_code)]
// src/data_service.rs - Service để fetch dữ liệu từ external APIs

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use anyhow::{Result, Context};
use std::sync::Arc;
use crate::cache::{CacheManager, CacheKeys};

// API URLs
const BASE_GLOBAL_URL: &str = "https://api.coingecko.com/api/v3/global";
const BASE_BTC_PRICE_URL: &str = "https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=usd&include_24hr_change=true";
const BASE_FNG_URL: &str = "https://api.alternative.me/fng/?limit=1";
const BASE_RSI_URL_TEMPLATE: &str = "https://api.taapi.io/rsi?secret={secret}&exchange=binance&symbol=BTC/USDT&interval=1d";

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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MarketData {
    pub symbol: String,
    pub price: f64,
    pub volume_24h: f64,
    pub change_24h: f64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TechnicalIndicator {
    pub symbol: String,
    pub indicator: String,
    pub period: String,
    pub value: f64,
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

#[derive(Clone)]
pub struct DataService {
    client: Client,
    taapi_secret: String,
    // Unified cache manager for all caching operations
    cache_manager: Option<Arc<CacheManager>>,
}

impl DataService {
    pub fn new(taapi_secret: String) -> Self {
        // Sử dụng optimized HTTP client từ performance module
        let client = crate::performance::OPTIMIZED_HTTP_CLIENT.clone();
        
        Self {
            client,
            taapi_secret,
            cache_manager: None,
        }
    }

    pub fn with_cache_manager(taapi_secret: String, cache_manager: Arc<CacheManager>) -> Self {
        // Sử dụng optimized HTTP client từ performance module
        let client = crate::performance::OPTIMIZED_HTTP_CLIENT.clone();
        
        Self {
            client,
            taapi_secret,
            cache_manager: Some(cache_manager),
        }
    }

    /// Main dashboard summary method - automatically uses cache if available
    pub async fn fetch_dashboard_summary(&self) -> Result<DashboardSummary> {
        // If cache manager is available, use cache-or-compute pattern
        if let Some(cache_manager) = &self.cache_manager {
            return cache_manager.cache_dashboard_data(|| {
                self.fetch_dashboard_summary_direct()
            }).await;
        }

        // Fallback to direct fetch if no cache
        self.fetch_dashboard_summary_direct().await
    }

    /// Direct fetch without caching (for internal use)
    /// Direct fetch without caching (for internal use)
    async fn fetch_dashboard_summary_direct(&self) -> Result<DashboardSummary> {
        println!("🔄 Fetching dashboard summary from external APIs...");

        // Fetch all data concurrently với better error handling
        let (global_result, btc_result, fng_result, rsi_result) = tokio::join!(
            self.fetch_global_data_with_retry(),
            self.fetch_btc_price_with_retry(),
            self.fetch_fear_greed_with_retry(),
            self.fetch_rsi_with_retry()
        );

        // Handle partial failures gracefully
        let (market_cap, volume_24h) = match global_result {
            Ok(data) => data,
            Err(e) => {
                eprintln!("⚠️ Failed to fetch global data: {}", e);
                (0.0, 0.0) // Default values
            }
        };

        let (btc_price_usd, btc_change_24h) = match btc_result {
            Ok(data) => data,
            Err(e) => {
                eprintln!("⚠️ Failed to fetch BTC price: {}", e);
                (0.0, 0.0) // Default values
            }
        };

        let fng_value = match fng_result {
            Ok(value) => value,
            Err(e) => {
                eprintln!("⚠️ Failed to fetch Fear & Greed: {}", e);
                50 // Neutral default
            }
        };

        let rsi_14 = match rsi_result {
            Ok(value) => value,
            Err(e) => {
                eprintln!("⚠️ Failed to fetch RSI: {}", e);
                50.0 // Neutral default
            }
        };

        let summary = DashboardSummary {
            market_cap,
            volume_24h,
            btc_price_usd,
            btc_change_24h,
            fng_value,
            rsi_14,
            last_updated: chrono::Utc::now(),
        };

        println!("✅ Dashboard summary fetched successfully");
        Ok(summary)
    }

    /// Fetch market data with intelligent caching
    pub async fn fetch_market_data(&self, symbol: &str) -> Result<MarketData> {
        if let Some(cache_manager) = &self.cache_manager {
            return cache_manager.cache_market_data(symbol, || {
                self.fetch_market_data_direct(symbol)
            }).await;
        }

        self.fetch_market_data_direct(symbol).await
    }

    async fn fetch_market_data_direct(&self, symbol: &str) -> Result<MarketData> {
        // Implementation for fetching specific market data
        println!("🔄 Fetching market data for {}", symbol);
        
        // Example: Fetch price, volume, change data for a specific symbol
        // This would call the appropriate API endpoints
        
        // Placeholder implementation
        Ok(MarketData {
            symbol: symbol.to_string(),
            price: 0.0,
            volume_24h: 0.0,
            change_24h: 0.0,
            last_updated: chrono::Utc::now(),
        })
    }

    /// Fetch technical indicator with caching
    pub async fn fetch_technical_indicator(&self, symbol: &str, indicator: &str, period: &str) -> Result<TechnicalIndicator> {
        if let Some(cache_manager) = &self.cache_manager {
            let key = CacheKeys::technical_indicator(symbol, indicator, period);
            return cache_manager.cache_or_compute(&key, 300, || { // 5 minute TTL for technical indicators
                self.fetch_technical_indicator_direct(symbol, indicator, period)
            }).await;
        }

        self.fetch_technical_indicator_direct(symbol, indicator, period).await
    }

    async fn fetch_technical_indicator_direct(&self, symbol: &str, indicator: &str, period: &str) -> Result<TechnicalIndicator> {
        println!("🔄 Fetching {} indicator for {} (period: {})", indicator, symbol, period);
        
        // Placeholder implementation - would call TAAPI or similar service
        Ok(TechnicalIndicator {
            symbol: symbol.to_string(),
            indicator: indicator.to_string(),
            period: period.to_string(),
            value: 0.0,
            last_updated: chrono::Utc::now(),
        })
    }

    /// Cache management methods
    pub async fn invalidate_cache(&self, pattern: &str) -> Result<u32> {
        if let Some(cache_manager) = &self.cache_manager {
            return cache_manager.clear_pattern(pattern).await;
        }
        Ok(0)
    }

    pub async fn get_cache_stats(&self) -> Option<crate::cache::CacheStats> {
        self.cache_manager.as_ref().map(|_cm| {
            // Note: This would need to be made async in a real implementation
            // For now, return a placeholder
            crate::cache::CacheStats {
                l1_entry_count: 0,
                l1_hit_count: 0,
                l1_miss_count: 0,
                l1_hit_rate: 0.0,
            }
        })
    }

    // Retry wrapper methods with exponential backoff
    async fn fetch_global_data_with_retry(&self) -> Result<(f64, f64)> {
        self.retry_with_backoff(|| self.fetch_global_data(), 3).await
    }

    async fn fetch_btc_price_with_retry(&self) -> Result<(f64, f64)> {
        self.retry_with_backoff(|| self.fetch_btc_price(), 3).await
    }

    async fn fetch_fear_greed_with_retry(&self) -> Result<u32> {
        self.retry_with_backoff(|| self.fetch_fear_greed(), 3).await
    }

    async fn fetch_rsi_with_retry(&self) -> Result<f64> {
        self.retry_with_backoff(|| self.fetch_rsi(), 3).await
    }

    // Generic retry logic với exponential backoff và special handling cho rate limits
    async fn retry_with_backoff<T, F, Fut>(&self, mut operation: F, max_retries: u32) -> Result<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let mut retries = 0;
        loop {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(err) => {
                    retries += 1;
                    if retries >= max_retries {
                        return Err(err);
                    }
                    
                    // Special handling cho 429 Too Many Requests
                    let delay = if err.to_string().contains("429") || err.to_string().contains("Too Many Requests") {
                        // Longer delay for rate limit errors - 2 minutes base
                        Duration::from_secs(120 * 2u64.pow(retries - 1)) // 2m, 4m, 8m
                    } else {
                        // Normal exponential backoff: 10s, 20s, 40s
                        Duration::from_secs(10 * 2u64.pow(retries - 1))
                    };
                    
                    println!("⏳ Retry {}/{} after {}s for error: {}", 
                        retries, max_retries, delay.as_secs(), err);
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }

    async fn fetch_global_data(&self) -> Result<(f64, f64)> {
        let response = self.client
            .get(BASE_GLOBAL_URL)
            .header("Accept", "application/json")
            .header("User-Agent", "Mozilla/5.0 (compatible; RustWebServer/1.0)")
            .send()
            .await
            .context("Failed to fetch global data - SSL/network error")?;

        // Check response status
        if !response.status().is_success() {
            anyhow::bail!("Global data API returned error: {}", response.status());
        }

        let global_data: CoinGeckoGlobal = response
            .json()
            .await
            .context("Failed to parse global data JSON")?;

        let market_cap = global_data.data.total_market_cap
            .get("usd")
            .copied()
            .unwrap_or(0.0);

        let volume_24h = global_data.data.total_volume
            .get("usd")
            .copied()
            .unwrap_or(0.0);

        println!("📊 Market Cap: ${:.2}, Volume 24h: ${:.2}", market_cap, volume_24h);
        Ok((market_cap, volume_24h))
    }

    async fn fetch_btc_price(&self) -> Result<(f64, f64)> {
        let response = self.client
            .get(BASE_BTC_PRICE_URL)
            .header("Accept", "application/json")
            .header("User-Agent", "Mozilla/5.0 (compatible; RustWebServer/1.0)")
            .send()
            .await
            .context("Failed to fetch BTC price - SSL/network error")?;

        // Check response status
        if !response.status().is_success() {
            anyhow::bail!("BTC price API returned error: {}", response.status());
        }

        let btc_data: CoinGeckoBtcPrice = response
            .json()
            .await
            .context("Failed to parse BTC price JSON")?;

        let price = btc_data.bitcoin.usd;
        let change_24h = btc_data.bitcoin.usd_24h_change;

        println!("₿ BTC Price: ${:.2}, Change 24h: {:.2}%", price, change_24h);
        Ok((price, change_24h))
    }

    async fn fetch_fear_greed(&self) -> Result<u32> {
        let response = self.client
            .get(BASE_FNG_URL)
            .header("Accept", "application/json")
            .header("User-Agent", "Mozilla/5.0 (compatible; RustWebServer/1.0)")
            .send()
            .await
            .context("Failed to fetch Fear & Greed index - SSL/network error")?;

        // Check response status
        if !response.status().is_success() {
            anyhow::bail!("Fear & Greed API returned error: {}", response.status());
        }

        let fng_data: FearGreedResponse = response
            .json()
            .await
            .context("Failed to parse Fear & Greed JSON")?;

        let value = fng_data.data
            .first()
            .context("No Fear & Greed data found")?
            .value
            .parse::<u32>()
            .context("Failed to parse Fear & Greed value")?;

        println!("😨 Fear & Greed Index: {}", value);
        Ok(value)
    }

    async fn fetch_rsi(&self) -> Result<f64> {
        let url = BASE_RSI_URL_TEMPLATE.replace("{secret}", &self.taapi_secret);
        
        let response = self.client
            .get(&url)
            .header("Accept", "application/json")
            .header("User-Agent", "Mozilla/5.0 (compatible; RustWebServer/1.0)")
            .send()
            .await
            .context("Failed to fetch RSI data - SSL/network error")?;

        // Check response status
        if !response.status().is_success() {
            anyhow::bail!("RSI API returned error: {}", response.status());
        }

        let rsi_data: TaapiRsiResponse = response
            .json()
            .await
            .context("Failed to parse RSI JSON")?;

        println!("📈 RSI 14: {:.2}", rsi_data.value);
        Ok(rsi_data.value)
    }
}
