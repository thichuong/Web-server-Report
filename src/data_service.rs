#![allow(dead_code)]
// src/data_service.rs - Service để fetch dữ liệu từ external APIs

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use anyhow::{Result, Context};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RateLimitStatus {
    pub btc_api_circuit_breaker_open: bool,
    pub seconds_since_last_btc_fetch: u64,
    pub can_fetch_btc_now: bool,
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
    // Rate limiting protection
    last_btc_fetch: Arc<AtomicU64>,
    btc_api_circuit_breaker: Arc<AtomicBool>,
}

impl DataService {
    pub fn new(taapi_secret: String) -> Self {
        // Sử dụng optimized HTTP client từ performance module
        let client = crate::performance::OPTIMIZED_HTTP_CLIENT.clone();
        
        Self {
            client,
            taapi_secret,
            cache_manager: None,
            last_btc_fetch: Arc::new(AtomicU64::new(0)),
            btc_api_circuit_breaker: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn with_cache_manager(taapi_secret: String, cache_manager: Arc<CacheManager>) -> Self {
        // Sử dụng optimized HTTP client từ performance module
        let client = crate::performance::OPTIMIZED_HTTP_CLIENT.clone();
        
        Self {
            client,
            taapi_secret,
            cache_manager: Some(cache_manager),
            last_btc_fetch: Arc::new(AtomicU64::new(0)),
            btc_api_circuit_breaker: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Main dashboard summary method - BTC price real-time, other data cached
    /// 
    /// This method implements an optimized caching strategy with rate limiting:
    /// - BTC price: Cached for 3 seconds minimum to prevent API spam
    /// - Other data (market cap, volume, Fear & Greed, RSI): Cached for 10 minutes
    /// - Circuit breaker protection for rate-limited APIs
    /// 
    /// This provides near real-time BTC price updates while staying within rate limits
    /// and protecting against client spam.
    pub async fn fetch_dashboard_summary(&self) -> Result<DashboardSummary> {
        // First check if we have a very recent complete dashboard cache (30 seconds)
        // This protects against rapid client requests
        if let Some(cache_manager) = &self.cache_manager {
            let rapid_cache_key = CacheKeys::dashboard_summary();
            if let Ok(Some(recent_summary)) = cache_manager.get::<DashboardSummary>(&rapid_cache_key).await {
                println!("🎯 Using rapid cache (30s protection against client spam)");
                return Ok(recent_summary);
            }
        }

        // If no rapid cache, use the optimized real-time method
        let summary = self.fetch_dashboard_summary_with_realtime_btc().await?;

        // Cache complete summary for 30 seconds to protect against client spam
        if let Some(cache_manager) = &self.cache_manager {
            let rapid_cache_key = CacheKeys::dashboard_summary();
            let _ = cache_manager.set_with_ttl(&rapid_cache_key, &summary, 30).await; // 30 seconds anti-spam cache
        }

        Ok(summary)
    }

    /// Fetch dashboard summary with real-time BTC price and cached other data
    /// Uses intelligent BTC caching: 3-second minimum interval between API calls
    async fn fetch_dashboard_summary_with_realtime_btc(&self) -> Result<DashboardSummary> {
        println!("🔄 Fetching dashboard summary with optimized BTC price...");

        // Try to get recently cached BTC price first (3-second cache)
        let btc_cache_key = CacheKeys::price_data("btc", "realtime");
        let (btc_price_usd, btc_change_24h) = if let Some(cache_manager) = &self.cache_manager {
            // Try to get BTC price from short-term cache (3 seconds)
            if let Ok(Some(cached_btc)) = cache_manager.get::<(f64, f64)>(&btc_cache_key).await {
                println!("🎯 Using cached BTC price (within 3s window)");
                cached_btc
            } else {
                // Fetch fresh BTC price and cache it for 3 seconds
                match self.fetch_btc_price_with_retry().await {
                    Ok(btc_data) => {
                        // Cache BTC price for 3 seconds to prevent API spam
                        let _ = cache_manager.set_with_ttl(&btc_cache_key, &btc_data, 3).await;
                        btc_data
                    }
                    Err(e) => {
                        eprintln!("⚠️ Failed to fetch BTC price: {}", e);
                        // Try to get older cached BTC data as fallback (up to 1 minute)
                        let fallback_key = CacheKeys::price_data("btc", "fallback");
                        if let Ok(Some(fallback_btc)) = cache_manager.get::<(f64, f64)>(&fallback_key).await {
                            println!("🔄 Using fallback cached BTC price");
                            fallback_btc
                        } else {
                            (0.0, 0.0)
                        }
                    }
                }
            }
        } else {
            // No cache manager, fetch directly with rate limiting
            match self.fetch_btc_price_with_retry().await {
                Ok(data) => data,
                Err(e) => {
                    eprintln!("⚠️ Failed to fetch BTC price: {}", e);
                    (0.0, 0.0)
                }
            }
        };

        // Store successful BTC fetch as fallback for future errors
        if btc_price_usd > 0.0 {
            if let Some(cache_manager) = &self.cache_manager {
                let fallback_key = CacheKeys::price_data("btc", "fallback");
                let _ = cache_manager.set_with_ttl(&fallback_key, &(btc_price_usd, btc_change_24h), 60).await; // 1 minute fallback
            }
        }

        // For other data, try to use cached values first
        if let Some(cache_manager) = &self.cache_manager {
            // Try to get cached non-BTC data
            if let Ok(Some(mut cached_summary)) = cache_manager.get::<DashboardSummary>(&CacheKeys::dashboard_summary_non_btc()).await {
                // Update with fresh/cached BTC data
                cached_summary.btc_price_usd = btc_price_usd;
                cached_summary.btc_change_24h = btc_change_24h;
                cached_summary.last_updated = chrono::Utc::now();
                
                println!("✅ Dashboard summary with cached data + optimized BTC");
                return Ok(cached_summary);
            }
        }

        // If no cached data available, fetch everything fresh
        self.fetch_dashboard_summary_direct_non_btc(btc_price_usd, btc_change_24h).await
    }

    /// Fetch non-BTC dashboard data and cache it separately
    async fn fetch_dashboard_summary_direct_non_btc(&self, btc_price_usd: f64, btc_change_24h: f64) -> Result<DashboardSummary> {
        println!("🔄 Fetching non-BTC dashboard data from external APIs...");

        // Fetch non-BTC data concurrently
        let (global_result, fng_result, rsi_result) = tokio::join!(
            self.fetch_global_data_with_retry(),
            self.fetch_fear_greed_with_retry(),
            self.fetch_rsi_with_retry()
        );

        // Handle partial failures gracefully
        let (market_cap, volume_24h) = match global_result {
            Ok(data) => data,
            Err(e) => {
                eprintln!("⚠️ Failed to fetch global data: {}", e);
                (0.0, 0.0)
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

        // Cache non-BTC data for future use
        if let Some(cache_manager) = &self.cache_manager {
            let non_btc_summary = DashboardSummary {
                market_cap,
                volume_24h,
                btc_price_usd: 0.0, // Will be replaced with real-time data
                btc_change_24h: 0.0, // Will be replaced with real-time data
                fng_value,
                rsi_14,
                last_updated: chrono::Utc::now(),
            };
            
            // Cache non-BTC data with longer TTL since it changes less frequently
            let _ = cache_manager.set_with_ttl(&CacheKeys::dashboard_summary_non_btc(), &non_btc_summary, 600).await; // 10 minutes TTL
        }

        println!("✅ Dashboard summary with fresh non-BTC data + real-time BTC");
        Ok(summary)
    }

    /// Legacy method - fetches all data fresh (for backward compatibility)
    pub async fn fetch_dashboard_summary_all_fresh(&self) -> Result<DashboardSummary> {
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

    /// Get current rate limiting status for monitoring
    pub fn get_rate_limit_status(&self) -> RateLimitStatus {
        let now = chrono::Utc::now().timestamp() as u64;
        let last_fetch = self.last_btc_fetch.load(Ordering::Relaxed);
        let circuit_breaker_open = self.btc_api_circuit_breaker.load(Ordering::Relaxed);
        
        RateLimitStatus {
            btc_api_circuit_breaker_open: circuit_breaker_open,
            seconds_since_last_btc_fetch: if last_fetch > 0 { now - last_fetch } else { 0 },
            can_fetch_btc_now: !circuit_breaker_open && (last_fetch == 0 || (now - last_fetch) >= 3),
        }
    }

    // Retry wrapper methods with exponential backoff
    async fn fetch_global_data_with_retry(&self) -> Result<(f64, f64)> {
        self.retry_with_backoff(|| self.fetch_global_data(), 3).await
    }

    async fn fetch_btc_price_with_retry(&self) -> Result<(f64, f64)> {
        self.fetch_btc_price_with_rate_limit().await
    }

    /// BTC price fetch with intelligent rate limiting and circuit breaker
    async fn fetch_btc_price_with_rate_limit(&self) -> Result<(f64, f64)> {
        // Check if circuit breaker is open (API temporarily blocked due to rate limits)
        if self.btc_api_circuit_breaker.load(Ordering::Relaxed) {
            println!("⚠️ BTC API circuit breaker is open, skipping API call");
            anyhow::bail!("BTC API circuit breaker is active");
        }

        // Get current timestamp in seconds
        let now = chrono::Utc::now().timestamp() as u64;
        let last_fetch = self.last_btc_fetch.load(Ordering::Relaxed);
        
        // Enforce minimum 3-second interval between API calls
        if last_fetch > 0 && (now - last_fetch) < 3 {
            let wait_time = 3 - (now - last_fetch);
            println!("⏳ Rate limiting: waiting {}s before BTC API call", wait_time);
            tokio::time::sleep(Duration::from_secs(wait_time)).await;
        }

        // Update last fetch timestamp
        self.last_btc_fetch.store(now, Ordering::Relaxed);

        // Try to fetch with circuit breaker protection
        match self.retry_with_backoff(|| self.fetch_btc_price(), 3).await {
            Ok(result) => {
                // Successful fetch - reset circuit breaker if it was open
                self.btc_api_circuit_breaker.store(false, Ordering::Relaxed);
                Ok(result)
            }
            Err(err) => {
                // Check if it's a rate limit error
                if err.to_string().contains("429") || err.to_string().contains("Too Many Requests") {
                    println!("🚨 BTC API rate limited - opening circuit breaker for 5 minutes");
                    self.btc_api_circuit_breaker.store(true, Ordering::Relaxed);
                    
                    // Schedule circuit breaker reset after 5 minutes
                    let circuit_breaker = self.btc_api_circuit_breaker.clone();
                    tokio::spawn(async move {
                        tokio::time::sleep(Duration::from_secs(300)).await; // 5 minutes
                        circuit_breaker.store(false, Ordering::Relaxed);
                        println!("🔄 BTC API circuit breaker reset");
                    });
                }
                Err(err)
            }
        }
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
                        // For rate limit errors - progressive delays: 30s, 60s, 120s
                        let base_delay = 30 * (1 << (retries - 1)); // 30s, 60s, 120s
                        Duration::from_secs(base_delay.min(300)) // Cap at 5 minutes max
                    } else {
                        // Normal exponential backoff: 5s, 10s, 20s
                        Duration::from_secs(5 * 2u64.pow(retries - 1))
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
