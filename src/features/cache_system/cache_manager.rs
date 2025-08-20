// src/features/cache_system/cache_manager.rs
//
// Unified Cache Manager - Single interface for all caching operations

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::future::Future;

use super::multi_tier_cache::MultiTierCache;
use super::cache_keys::CacheKeys;
use super::cache_stats::{CacheStats, CacheHealthCheck};

// Cache TTL configurations for different data types
const DASHBOARD_TTL_SECONDS: u64 = 300;     // 5 minutes for dashboard data
const MARKET_DATA_TTL_SECONDS: u64 = 60;    // 1 minute for market data
const REPORT_TTL_SECONDS: u64 = 1800;       // 30 minutes for reports
const USER_DATA_TTL_SECONDS: u64 = 900;     // 15 minutes for user data

/// Unified Cache Manager - Single interface for all caching operations
#[derive(Debug, Clone)]
pub struct CacheManager {
    cache: Arc<MultiTierCache>,
}

impl CacheManager {
    pub async fn new(redis_url: &str) -> Result<Self> {
        let cache = Arc::new(MultiTierCache::new(redis_url).await?);
        
        println!("🚀 Unified Cache Manager initialized");
        
        Ok(Self { cache })
    }

    /// Generic cache-or-compute pattern
    pub async fn cache_or_compute<T, F, Fut>(&self, key: &str, ttl_seconds: u64, compute_fn: F) -> Result<T>
    where
        T: Serialize + for<'de> Deserialize<'de> + Clone,
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T>>,
    {
        // Try to get from cache first
        if let Ok(Some(cached_data)) = self.cache.get::<T>(key).await {
            return Ok(cached_data);
        }

        // Cache miss - compute fresh data
        println!("💻 Computing fresh data for key: {}", key);
        let fresh_data = compute_fn().await?;

        // Cache the result with custom TTL
        if let Err(e) = self.cache.set_with_ttl(key, &fresh_data, ttl_seconds).await {
            eprintln!("⚠️ Failed to cache computed data: {}", e);
        }

        Ok(fresh_data)
    }

    /// Dashboard-specific caching with intelligent refresh
    pub async fn cache_dashboard_data<T, F, Fut>(&self, compute_fn: F) -> Result<T>
    where
        T: Serialize + for<'de> Deserialize<'de> + Clone + Send + 'static,
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T>>,
    {
        let key = CacheKeys::dashboard_summary();
        self.cache_or_compute(&key, DASHBOARD_TTL_SECONDS, compute_fn).await
    }

    /// Market data caching with short TTL
    pub async fn cache_market_data<T, F, Fut>(&self, symbol: &str, compute_fn: F) -> Result<T>
    where
        T: Serialize + for<'de> Deserialize<'de> + Clone + Send + 'static,
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T>>,
    {
        let key = CacheKeys::market_data(symbol);
        self.cache_or_compute(&key, MARKET_DATA_TTL_SECONDS, compute_fn).await
    }

    /// Report caching with medium TTL
    pub async fn cache_report_data<T, F, Fut>(&self, symbol: &str, timeframe: &str, compute_fn: F) -> Result<T>
    where
        T: Serialize + for<'de> Deserialize<'de> + Clone + Send + 'static,
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T>>,
    {
        let key = CacheKeys::crypto_report(symbol, timeframe);
        self.cache_or_compute(&key, REPORT_TTL_SECONDS, compute_fn).await
    }

    /// User-specific data caching
    pub async fn cache_user_data<T, F, Fut>(&self, user_id: u32, data_type: &str, compute_fn: F) -> Result<T>
    where
        T: Serialize + for<'de> Deserialize<'de> + Clone + Send + 'static,
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T>>,
    {
        let key = format!("user:{}:{}", user_id, data_type);
        self.cache_or_compute(&key, USER_DATA_TTL_SECONDS, compute_fn).await
    }

    // Direct access to underlying cache operations
    pub async fn get<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        self.cache.get(key).await
    }

    pub async fn set<T>(&self, key: &str, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.cache.set(key, value).await
    }

    pub async fn set_with_ttl<T>(&self, key: &str, value: &T, ttl_seconds: u64) -> Result<()>
    where
        T: Serialize,
    {
        self.cache.set_with_ttl(key, value, ttl_seconds).await
    }

    pub async fn invalidate(&self, key: &str) -> Result<()> {
        self.cache.invalidate(key).await
    }

    pub async fn clear_pattern(&self, pattern: &str) -> Result<u32> {
        self.cache.clear_pattern(pattern).await
    }

    pub async fn stats(&self) -> CacheStats {
        self.cache.stats().await
    }

    pub async fn health_check(&self) -> CacheHealthCheck {
        self.cache.health_check().await
    }
}
