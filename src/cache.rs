#![allow(dead_code)]
// src/cache.rs - Unified Multi-tier caching system: L1 (In-Memory) + L2 (Redis)
// Provides a single interface for all caching operations across the application

use anyhow::{Context, Result};
use moka::future::Cache as MokaCache;
use bb8_redis::{bb8::Pool, RedisConnectionManager, redis::AsyncCommands};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use std::sync::Arc;
use std::future::Future;

// Cache configuration constants
const L1_MAX_CAPACITY: u64 = 2000; // Increased capacity for more data
const L1_TTL_SECONDS: u64 = 300;   // 5 minutes TTL for L1 cache
const L2_TTL_SECONDS: u64 = 3600;  // 1 hour TTL for L2 cache

// Cache TTL configurations for different data types
const DASHBOARD_TTL_SECONDS: u64 = 300;     // 5 minutes for dashboard data
const MARKET_DATA_TTL_SECONDS: u64 = 60;    // 1 minute for market data
const REPORT_TTL_SECONDS: u64 = 1800;       // 30 minutes for reports
const USER_DATA_TTL_SECONDS: u64 = 900;     // 15 minutes for user data

pub type RedisPool = Pool<RedisConnectionManager>;

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

#[derive(Debug, Clone)]
pub struct MultiTierCache {
    l1_cache: Arc<MokaCache<String, String>>, // In-memory cache (L1)
    redis_pool: Arc<RedisPool>,               // Redis connection pool (L2)
}

impl MultiTierCache {
    pub async fn new(redis_url: &str) -> Result<Self> {
        // Initialize L1 cache (In-Memory với moka)
        let l1_cache = Arc::new(
            MokaCache::builder()
                .max_capacity(L1_MAX_CAPACITY)
                .time_to_live(Duration::from_secs(L1_TTL_SECONDS))
                .time_to_idle(Duration::from_secs(L1_TTL_SECONDS / 2))
                .build()
        );

        // Initialize L2 cache (Redis pool)
        let redis_manager = RedisConnectionManager::new(redis_url)
            .context("Failed to create Redis connection manager")?;
            
        let redis_pool = Arc::new(
            Pool::builder()
                .max_size(10) // Maximum 10 Redis connections
                .min_idle(Some(2)) // Keep at least 2 idle connections
                .build(redis_manager)
                .await
                .context("Failed to create Redis connection pool")?
        );

        println!("✅ Multi-tier cache system initialized");
        println!("   L1 (In-Memory): {} max entries, {}s TTL", L1_MAX_CAPACITY, L1_TTL_SECONDS);
        println!("   L2 (Redis): {}s TTL", L2_TTL_SECONDS);

        Ok(Self {
            l1_cache,
            redis_pool,
        })
    }

    /// Get data from cache with multi-tier fallback
    pub async fn get<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        // Step 1: Check L1 cache first (fastest)
        if let Some(cached_data) = self.l1_cache.get(key).await {
            println!("🎯 L1 Cache HIT for key: {}", key);
            let data: T = serde_json::from_str(&cached_data)
                .context("Failed to deserialize from L1 cache")?;
            return Ok(Some(data));
        }

        // Step 2: Check L2 cache (Redis) if L1 miss
        if let Ok(cached_data) = self.get_from_redis(key).await {
            if let Some(redis_data) = cached_data {
                println!("🔥 L2 Cache HIT for key: {}", key);
                
                // Promote to L1 cache for faster future access
                self.l1_cache.insert(key.to_string(), redis_data.clone()).await;
                
                let data: T = serde_json::from_str(&redis_data)
                    .context("Failed to deserialize from L2 cache")?;
                return Ok(Some(data));
            }
        }

        println!("❌ Cache MISS for key: {}", key);
        Ok(None)
    }

    /// Set data in both cache layers
    pub async fn set<T>(&self, key: &str, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        let serialized = serde_json::to_string(value)
            .context("Failed to serialize data for cache")?;

        // Set in L1 cache (immediate)
        self.l1_cache.insert(key.to_string(), serialized.clone()).await;

        // Set in L2 cache (Redis) with TTL
        if let Err(e) = self.set_in_redis(key, &serialized).await {
            eprintln!("⚠️ Failed to cache in Redis (L2): {}", e);
            // Don't fail the operation if Redis is unavailable
        }

        println!("💾 Cached data for key: {}", key);
        Ok(())
    }

    /// Set data with custom TTL
    pub async fn set_with_ttl<T>(&self, key: &str, value: &T, ttl_seconds: u64) -> Result<()>
    where
        T: Serialize,
    {
        let serialized = serde_json::to_string(value)
            .context("Failed to serialize data for cache")?;

        // Set in L1 cache (immediate)
        self.l1_cache.insert(key.to_string(), serialized.clone()).await;

        // Set in L2 cache (Redis) with custom TTL
        if let Err(e) = self.set_in_redis_with_ttl(key, &serialized, ttl_seconds).await {
            eprintln!("⚠️ Failed to cache in Redis (L2): {}", e);
            // Don't fail the operation if Redis is unavailable
        }

        println!("💾 Cached data for key: {} (TTL: {}s)", key, ttl_seconds);
        Ok(())
    }

    /// Clear keys matching a pattern (Redis only)
    pub async fn clear_pattern(&self, pattern: &str) -> Result<u32> {
        self.clear_redis_pattern(pattern).await
    }

    /// Invalidate data from both cache layers
    pub async fn invalidate(&self, key: &str) -> Result<()> {
        // Remove from L1 cache
        self.l1_cache.invalidate(key).await;

        // Remove from L2 cache (Redis)
        if let Err(e) = self.delete_from_redis(key).await {
            eprintln!("⚠️ Failed to invalidate Redis cache (L2): {}", e);
        }

        println!("🗑️ Invalidated cache for key: {}", key);
        Ok(())
    }

    /// Clear all cached data
    pub async fn clear_all(&self) -> Result<()> {
        // Clear L1 cache
        self.l1_cache.invalidate_all();
        
        // Note: We don't clear all Redis data as it might be shared with other services
        println!("🧹 Cleared L1 cache (in-memory)");
        Ok(())
    }

    /// Get cache statistics for monitoring
    pub async fn stats(&self) -> CacheStats {
        let l1_entry_count = self.l1_cache.entry_count();
        // Note: moka cache doesn't provide hit/miss counts directly
        // We'll implement our own counters or use approximations
        let l1_hit_count = 0; // Would need custom implementation
        let l1_miss_count = 0; // Would need custom implementation
        
        let l1_hit_rate = 0.0; // Calculate based on custom metrics

        CacheStats {
            l1_entry_count,
            l1_hit_count,
            l1_miss_count,
            l1_hit_rate,
        }
    }

    /// Health check for cache system
    pub async fn health_check(&self) -> CacheHealthCheck {
        let l1_healthy = true; // L1 is always available
        let l2_healthy = self.check_redis_health().await;

        CacheHealthCheck {
            l1_healthy,
            l2_healthy,
            overall_healthy: l1_healthy, // System works as long as L1 is available
        }
    }

    // Private helper methods for Redis operations
    async fn get_from_redis(&self, key: &str) -> Result<Option<String>> {
        let mut conn = self.redis_pool.get().await
            .context("Failed to get Redis connection from pool")?;
            
        let result: Option<String> = conn.get(key).await
            .context("Failed to get data from Redis")?;
            
        Ok(result)
    }

    async fn set_in_redis(&self, key: &str, value: &str) -> Result<()> {
        let mut conn = self.redis_pool.get().await
            .context("Failed to get Redis connection from pool")?;
            
    let _: () = conn.set_ex(key, value, L2_TTL_SECONDS as u64).await
            .context("Failed to set data in Redis")?;
            
        Ok(())
    }

    async fn delete_from_redis(&self, key: &str) -> Result<()> {
        let mut conn = self.redis_pool.get().await
            .context("Failed to get Redis connection from pool")?;
            
        let _: () = conn.del(key).await
            .context("Failed to delete data from Redis")?;
            
        Ok(())
    }

    async fn set_in_redis_with_ttl(&self, key: &str, value: &str, ttl_seconds: u64) -> Result<()> {
        let mut conn = self.redis_pool.get().await
            .context("Failed to get Redis connection from pool")?;
            
        let _: () = conn.set_ex(key, value, ttl_seconds).await
            .context("Failed to set data in Redis with custom TTL")?;
            
        Ok(())
    }

    async fn clear_redis_pattern(&self, pattern: &str) -> Result<u32> {
        let mut conn = self.redis_pool.get().await
            .context("Failed to get Redis connection from pool")?;
            
        // Get all keys matching the pattern
        let keys: Vec<String> = conn.keys(pattern).await
            .context("Failed to get keys matching pattern")?;
            
        if keys.is_empty() {
            return Ok(0);
        }

        // Delete all matching keys
        let deleted_count: u32 = conn.del(&keys).await
            .context("Failed to delete keys matching pattern")?;
            
        println!("🧹 Cleared {} keys matching pattern: {}", deleted_count, pattern);
        Ok(deleted_count)
    }

    async fn check_redis_health(&self) -> bool {
        match self.redis_pool.get().await {
            Ok(mut conn) => {
                match conn.get::<_, Option<String>>("__health_check__").await {
                    Ok(_) => true,
                    Err(_) => false,
                }
            }
            Err(_) => false,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CacheStats {
    pub l1_entry_count: u64,
    pub l1_hit_count: u64,
    pub l1_miss_count: u64,
    pub l1_hit_rate: f64, // Percentage
}

#[derive(Debug, Clone, Serialize)]
pub struct CacheHealthCheck {
    pub l1_healthy: bool,
    pub l2_healthy: bool,
    pub overall_healthy: bool,
}

// Cache key generators for consistent key naming
pub struct CacheKeys;

impl CacheKeys {
    pub fn dashboard_summary() -> String {
        "dashboard:summary".to_string()
    }
    
    pub fn crypto_report(symbol: &str, timeframe: &str) -> String {
        format!("crypto:report:{}:{}", symbol.to_lowercase(), timeframe)
    }
    
    pub fn market_data(symbol: &str) -> String {
        format!("market:{}", symbol.to_lowercase())
    }
    
    pub fn user_report(user_id: u32, report_id: u32) -> String {
        format!("user:{}:report:{}", user_id, report_id)
    }

    // New specialized cache keys
    pub fn api_data(provider: &str, endpoint: &str) -> String {
        format!("api:{}:{}", provider.to_lowercase(), endpoint)
    }

    pub fn technical_indicator(symbol: &str, indicator: &str, period: &str) -> String {
        format!("tech:{}:{}:{}", symbol.to_lowercase(), indicator, period)
    }

    pub fn price_data(symbol: &str, interval: &str) -> String {
        format!("price:{}:{}", symbol.to_lowercase(), interval)
    }
}

#[derive(Debug)]
pub enum CacheError {
    SerializationError(String),
    ConnectionError(String),
    TimeoutError,
}

impl std::fmt::Display for CacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CacheError::SerializationError(msg) => write!(f, "Cache serialization error: {}", msg),
            CacheError::ConnectionError(msg) => write!(f, "Cache connection error: {}", msg),
            CacheError::TimeoutError => write!(f, "Cache operation timeout"),
        }
    }
}

impl std::error::Error for CacheError {}
