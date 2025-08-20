// src/features/external_apis/market_data_provider.rs
//
// Main market data provider with intelligent caching and rate limiting

use anyhow::{Context, Result};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};

use super::api_client::ApiClient;
use super::models::*;
use crate::features::cache_system::CacheManager;

/// Market data provider with intelligent caching and retry logic
#[derive(Debug, Clone)]
pub struct MarketDataProvider {
    api_client: ApiClient,
    taapi_secret: String,
    cache_manager: Option<Arc<CacheManager>>,
    // Rate limiting protection for BTC API
    last_btc_fetch: Arc<AtomicU64>,
    btc_api_circuit_breaker: Arc<AtomicBool>,
}

impl MarketDataProvider {
    pub async fn new(taapi_secret: String, cache_manager: Option<Arc<CacheManager>>) -> Result<Self> {
        let api_client = ApiClient::new()?;
        
        // Configure rate limiting intervals
        api_client.rate_limiter().set_interval("coingecko_global", 1).await;
        api_client.rate_limiter().set_interval("coingecko_btc_price", 3).await; // 3 second minimum
        api_client.rate_limiter().set_interval("fear_greed_index", 60).await;
        api_client.rate_limiter().set_interval("taapi_rsi", 60).await;

        Ok(Self {
            api_client,
            taapi_secret,
            cache_manager,
            last_btc_fetch: Arc::new(AtomicU64::new(0)),
            btc_api_circuit_breaker: Arc::new(AtomicBool::new(false)),
        })
    }

    /// Main dashboard summary method with optimized caching strategy
    pub async fn fetch_dashboard_summary(&self) -> Result<DashboardSummary> {
        // First check for rapid cache to prevent client spam
        if let Some(cache_manager) = &self.cache_manager {
            let rapid_cache_key = "dashboard:summary:rapid";
            if let Ok(Some(recent_summary)) = cache_manager.get::<DashboardSummary>(rapid_cache_key).await {
                println!("🎯 Using rapid cache (30s protection against client spam)");
                return Ok(recent_summary);
            }
        }

        // Use optimized real-time BTC method
        let summary = self.fetch_dashboard_summary_with_realtime_btc().await?;

        // Cache complete summary for 30 seconds to protect against client spam
        if let Some(cache_manager) = &self.cache_manager {
            let rapid_cache_key = "dashboard:summary:rapid";
            let _ = cache_manager.set_with_ttl(rapid_cache_key, &summary, 30).await;
        }

        Ok(summary)
    }

    /// Fetch dashboard with real-time BTC price and cached other data
    async fn fetch_dashboard_summary_with_realtime_btc(&self) -> Result<DashboardSummary> {
        println!("🔄 Fetching dashboard summary with optimized BTC price...");

        // Try to get recently cached BTC price (3-second cache)
        let (btc_price_usd, btc_change_24h) = if let Some(cache_manager) = &self.cache_manager {
            let btc_cache_key = "price:btc:realtime";
            
            if let Ok(Some(cached_btc)) = cache_manager.get::<(f64, f64)>(btc_cache_key).await {
                println!("🎯 Using cached BTC price (within 3s window)");
                cached_btc
            } else {
                // Fetch fresh BTC price and cache it
                match self.fetch_btc_price_with_rate_limit().await {
                    Ok(btc_data) => {
                        let _ = cache_manager.set_with_ttl(btc_cache_key, &btc_data, 3).await;
                        btc_data
                    }
                    Err(e) => {
                        eprintln!("⚠️ Failed to fetch BTC price: {}", e);
                        // Try fallback cache
                        let fallback_key = "price:btc:fallback";
                        if let Ok(Some(fallback_btc)) = cache_manager.get::<(f64, f64)>(fallback_key).await {
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
            self.fetch_btc_price_with_rate_limit().await.unwrap_or((0.0, 0.0))
        };

        // Store successful BTC fetch as fallback
        if btc_price_usd > 0.0 {
            if let Some(cache_manager) = &self.cache_manager {
                let fallback_key = "price:btc:fallback";
                let _ = cache_manager.set_with_ttl(fallback_key, &(btc_price_usd, btc_change_24h), 60).await;
            }
        }

        // Try to get cached non-BTC data
        if let Some(cache_manager) = &self.cache_manager {
            let non_btc_key = "dashboard:summary:non_btc";
            if let Ok(Some(mut cached_summary)) = cache_manager.get::<DashboardSummary>(non_btc_key).await {
                // Update with fresh BTC data
                cached_summary.btc_price_usd = btc_price_usd;
                cached_summary.btc_change_24h = btc_change_24h;
                cached_summary.last_updated = chrono::Utc::now();
                
                println!("✅ Dashboard summary with cached data + optimized BTC");
                return Ok(cached_summary);
            }
        }

        // No cached data, fetch everything fresh
        self.fetch_dashboard_summary_direct_non_btc(btc_price_usd, btc_change_24h).await
    }

    /// Fetch non-BTC data and cache it separately
    async fn fetch_dashboard_summary_direct_non_btc(&self, btc_price_usd: f64, btc_change_24h: f64) -> Result<DashboardSummary> {
        println!("🔄 Fetching non-BTC dashboard data from external APIs...");

        // Fetch non-BTC data concurrently
        let (global_result, fng_result, rsi_result) = tokio::join!(
            self.fetch_global_data(),
            self.fetch_fear_greed_index(),
            self.fetch_rsi()
        );

        // Handle partial failures gracefully
        let (market_cap, volume_24h) = global_result.unwrap_or_else(|e| {
            eprintln!("⚠️ Failed to fetch global data: {}", e);
            (0.0, 0.0)
        });

        let fng_value = fng_result.unwrap_or_else(|e| {
            eprintln!("⚠️ Failed to fetch Fear & Greed: {}", e);
            50 // Neutral default
        });

        let rsi_14 = rsi_result.unwrap_or_else(|e| {
            eprintln!("⚠️ Failed to fetch RSI: {}", e);
            50.0 // Neutral default
        });

        let summary = DashboardSummary {
            market_cap,
            volume_24h,
            btc_price_usd,
            btc_change_24h,
            fng_value,
            rsi_14,
            last_updated: chrono::Utc::now(),
        };

        // Cache non-BTC data with longer TTL
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
            
            let non_btc_key = "dashboard:summary:non_btc";
            let _ = cache_manager.set_with_ttl(non_btc_key, &non_btc_summary, 600).await; // 10 minutes
        }

        println!("✅ Dashboard summary with fresh non-BTC data + real-time BTC");
        Ok(summary)
    }

    /// Fetch BTC price with intelligent rate limiting and circuit breaker
    async fn fetch_btc_price_with_rate_limit(&self) -> Result<(f64, f64)> {
        // Check circuit breaker
        if self.btc_api_circuit_breaker.load(Ordering::Relaxed) {
            anyhow::bail!("BTC API circuit breaker is active");
        }

        // Rate limiting enforcement
        let now = chrono::Utc::now().timestamp() as u64;
        let last_fetch = self.last_btc_fetch.load(Ordering::Relaxed);
        
        if last_fetch > 0 && (now - last_fetch) < 3 {
            let wait_time = 3 - (now - last_fetch);
            println!("⏳ Rate limiting: waiting {}s before BTC API call", wait_time);
            tokio::time::sleep(std::time::Duration::from_secs(wait_time)).await;
        }

        self.last_btc_fetch.store(now, Ordering::Relaxed);

        // Perform the request
        match self.fetch_btc_price_direct().await {
            Ok(result) => {
                self.btc_api_circuit_breaker.store(false, Ordering::Relaxed);
                Ok(result)
            }
            Err(err) => {
                // Handle rate limiting
                if err.to_string().contains("429") || err.to_string().contains("Too Many Requests") {
                    println!("🚨 BTC API rate limited - opening circuit breaker for 5 minutes");
                    self.btc_api_circuit_breaker.store(true, Ordering::Relaxed);
                    
                    // Schedule reset
                    let circuit_breaker = self.btc_api_circuit_breaker.clone();
                    tokio::spawn(async move {
                        tokio::time::sleep(std::time::Duration::from_secs(300)).await;
                        circuit_breaker.store(false, Ordering::Relaxed);
                        println!("🔄 BTC API circuit breaker reset");
                    });
                }
                Err(err)
            }
        }
    }

    /// Direct API calls
    async fn fetch_global_data(&self) -> Result<(f64, f64)> {
        let endpoint = ApiEndpoint::coingecko_global();
        let response = self.api_client.get_with_retry::<CoinGeckoGlobal>(
            &endpoint.url,
            &endpoint.name,
            3,
        ).await?;

        if let Some(global_data) = response.data {
            let market_cap = global_data.data.total_market_cap.get("usd").copied().unwrap_or(0.0);
            let volume_24h = global_data.data.total_volume.get("usd").copied().unwrap_or(0.0);
            println!("📊 Market Cap: ${:.2}, Volume 24h: ${:.2}", market_cap, volume_24h);
            Ok((market_cap, volume_24h))
        } else {
            anyhow::bail!("No data received from global API");
        }
    }

    async fn fetch_btc_price_direct(&self) -> Result<(f64, f64)> {
        let endpoint = ApiEndpoint::coingecko_btc_price();
        let response = self.api_client.get_with_retry::<CoinGeckoBtcPrice>(
            &endpoint.url,
            &endpoint.name,
            3,
        ).await?;

        if let Some(btc_data) = response.data {
            let price = btc_data.bitcoin.usd;
            let change_24h = btc_data.bitcoin.usd_24h_change;
            println!("₿ BTC Price: ${:.2}, Change 24h: {:.2}%", price, change_24h);
            Ok((price, change_24h))
        } else {
            anyhow::bail!("No data received from BTC price API");
        }
    }

    async fn fetch_fear_greed_index(&self) -> Result<u32> {
        let endpoint = ApiEndpoint::fear_greed_index();
        let response = self.api_client.get_with_retry::<FearGreedResponse>(
            &endpoint.url,
            &endpoint.name,
            3,
        ).await?;

        if let Some(fng_data) = response.data {
            if let Some(fng_item) = fng_data.data.first() {
                let fng_value = fng_item.value.parse::<u32>()
                    .context("Failed to parse Fear & Greed value")?;
                println!("😱 Fear & Greed Index: {}", fng_value);
                Ok(fng_value)
            } else {
                anyhow::bail!("No Fear & Greed data available");
            }
        } else {
            anyhow::bail!("No data received from Fear & Greed API");
        }
    }

    async fn fetch_rsi(&self) -> Result<f64> {
        let endpoint = ApiEndpoint::taapi_rsi(&self.taapi_secret);
        let response = self.api_client.get_with_retry::<TaapiRsiResponse>(
            &endpoint.url,
            &endpoint.name,
            3,
        ).await?;

        if let Some(rsi_data) = response.data {
            println!("📈 RSI 14: {:.2}", rsi_data.value);
            Ok(rsi_data.value)
        } else {
            anyhow::bail!("No data received from RSI API");
        }
    }

    /// Get rate limiting status for monitoring
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

    /// Market data for specific symbols
    pub async fn fetch_market_data(&self, symbol: &str) -> Result<MarketData> {
        if let Some(cache_manager) = &self.cache_manager {
            let key = format!("market:{}", symbol.to_lowercase());
            return cache_manager.cache_or_compute(&key, 300, || { // 5 minute TTL
                self.fetch_market_data_direct(symbol)
            }).await;
        }

        self.fetch_market_data_direct(symbol).await
    }

    async fn fetch_market_data_direct(&self, symbol: &str) -> Result<MarketData> {
        println!("🔄 Fetching market data for {}", symbol);
        
        // Placeholder implementation - would call appropriate API
        Ok(MarketData {
            symbol: symbol.to_string(),
            price: 0.0,
            volume_24h: 0.0,
            change_24h: 0.0,
            last_updated: chrono::Utc::now(),
        })
    }

    /// Technical indicators
    pub async fn fetch_technical_indicator(&self, symbol: &str, indicator: &str, period: &str) -> Result<TechnicalIndicator> {
        if let Some(cache_manager) = &self.cache_manager {
            let key = format!("tech:{}:{}:{}", symbol.to_lowercase(), indicator, period);
            return cache_manager.cache_or_compute(&key, 300, || {
                self.fetch_technical_indicator_direct(symbol, indicator, period)
            }).await;
        }

        self.fetch_technical_indicator_direct(symbol, indicator, period).await
    }

    async fn fetch_technical_indicator_direct(&self, symbol: &str, indicator: &str, period: &str) -> Result<TechnicalIndicator> {
        println!("🔄 Fetching {} indicator for {} (period: {})", indicator, symbol, period);
        
        // Placeholder - would use TAAPI or similar
        Ok(TechnicalIndicator {
            symbol: symbol.to_string(),
            indicator: indicator.to_string(),
            period: period.to_string(),
            value: 0.0,
            last_updated: chrono::Utc::now(),
        })
    }
}
