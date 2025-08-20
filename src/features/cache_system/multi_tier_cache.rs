// src/features/cache_system/multi_tier_cache.rs
//
// Multi-tier cache implementation with L1 (in-memory) and L2 (Redis) layers

use anyhow::{Context, Result};
use moka::future::Cache as MokaCache;
use bb8_redis::{bb8::Pool, RedisConnectionManager, redis::AsyncCommands};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use std::sync::Arc;

use super::cache_stats::{CacheStats, CacheHealthCheck};

// Cache configuration constants
const L1_MAX_CAPACITY: u64 = 2000; // Increased capacity for more data
const L1_TTL_SECONDS: u64 = 300;   // 5 minutes TTL for L1 cache
const L2_TTL_SECONDS: u64 = 3600;  // 1 hour TTL for L2 cache

pub type RedisPool = Pool<RedisConnectionManager>;

#[derive(Debug, Clone)]
pub struct MultiTierCache {
    l1_cache: Arc<MokaCache<String, String>>, // In-memory cache (L1)
    redis_pool: Arc<RedisPool>,               // Redis connection pool (L2)
}

impl MultiTierCache {
    pub async fn new(redis_url: &str) -> Result<Self> {
        // Initialize L1 cache (In-Memory with moka)
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

    /// Health check for both cache tiers
    pub async fn health_check(&self) -> CacheHealthCheck {
        let l1_healthy = true; // L1 cache is always healthy if initialized
        let l2_healthy = self.check_redis_health().await;
        let overall_healthy = l1_healthy; // System can work with just L1

        CacheHealthCheck {
            l1_healthy,
            l2_healthy,
            overall_healthy,
        }
    }

    // Private Redis operations
    async fn get_from_redis(&self, key: &str) -> Result<Option<String>> {
        let mut conn = self.redis_pool.get().await
            .context("Failed to get Redis connection from pool")?;
            
        let result: Option<String> = conn.get(key).await
            .context("Failed to get value from Redis")?;
            
        Ok(result)
    }

    async fn set_in_redis(&self, key: &str, value: &str) -> Result<()> {
        let mut conn = self.redis_pool.get().await
            .context("Failed to get Redis connection from pool")?;
            
        conn.set_ex(key, value, L2_TTL_SECONDS).await
            .context("Failed to set value in Redis")?;
            
        Ok(())
    }

    async fn set_in_redis_with_ttl(&self, key: &str, value: &str, ttl_seconds: u64) -> Result<()> {
        let mut conn = self.redis_pool.get().await
            .context("Failed to get Redis connection from pool")?;
            
        conn.set_ex(key, value, ttl_seconds).await
            .context("Failed to set value in Redis with TTL")?;
            
        Ok(())
    }

    async fn delete_from_redis(&self, key: &str) -> Result<()> {
        let mut conn = self.redis_pool.get().await
            .context("Failed to get Redis connection from pool")?;
            
        conn.del(key).await
            .context("Failed to delete value from Redis")?;
            
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
