//! L1 Cache - Moka In-Memory Cache
//! 
//! High-performance in-memory cache using Moka for hot data.
//! Optimized for sub-millisecond access times with intelligent eviction.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, AtomicU64, Ordering};
use std::time::Duration;
use anyhow::Result;
use serde_json;
use moka::future::Cache;

/// L1 Cache configuration
pub struct L1CacheConfig {
    pub max_capacity: u64,
    pub time_to_live: Duration,
    pub time_to_idle: Duration,
}

impl Default for L1CacheConfig {
    fn default() -> Self {
        Self {
            max_capacity: 2000,
            time_to_live: Duration::from_secs(900), // Increased from 300s (5min) to 900s (15min)
            time_to_idle: Duration::from_secs(300), // Increased from 120s (2min) to 300s (5min)
        }
    }
}

/// L1 Cache - Moka in-memory cache for hot data
pub struct L1Cache {
    /// Moka cache instance
    cache: Cache<String, serde_json::Value>,
    /// Cache configuration
    config: L1CacheConfig,
    /// Statistics tracking
    hits: Arc<AtomicU64>,
    misses: Arc<AtomicU64>,
    sets: Arc<AtomicU64>,
    evictions: Arc<AtomicUsize>,
}

impl L1Cache {
    /// Initialize L1 Cache with Moka
    pub async fn new() -> Result<Self> {
        Self::new_with_config(L1CacheConfig::default()).await
    }
    
    /// Initialize L1 Cache with custom configuration
    pub async fn new_with_config(config: L1CacheConfig) -> Result<Self> {
        println!("🔥 Initializing L1 Cache (Moka)...");
        
        // Build Moka cache with configuration
        let cache = Cache::builder()
            .max_capacity(config.max_capacity)
            .time_to_live(config.time_to_live)
            .time_to_idle(config.time_to_idle)
            .eviction_listener(|key, _value, _cause| {
                println!("🗑️  L1 Cache evicted key: {}", key);
            })
            .build();
        
        println!("  🔥 L1 Cache configured: {} entries, {}s TTL", 
                 config.max_capacity, config.time_to_live.as_secs());
        
        Ok(Self {
            cache,
            config,
            hits: Arc::new(AtomicU64::new(0)),
            misses: Arc::new(AtomicU64::new(0)),
            sets: Arc::new(AtomicU64::new(0)),
            evictions: Arc::new(AtomicUsize::new(0)),
        })
    }
    
    /// Get value from L1 cache
    pub async fn get(&self, key: &str) -> Option<serde_json::Value> {
        match self.cache.get(key).await {
            Some(value) => {
                self.hits.fetch_add(1, Ordering::Relaxed);
                Some(value)
            }
            None => {
                self.misses.fetch_add(1, Ordering::Relaxed);
                None
            }
        }
    }
    
    /// Set value in L1 cache
    pub async fn set(&self, key: &str, value: serde_json::Value) -> Result<()> {
        self.cache.insert(key.to_string(), value).await;
        self.sets.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }
    
    /// Set value in L1 cache with custom TTL
    pub async fn set_with_ttl(&self, key: &str, value: serde_json::Value, ttl: Duration) -> Result<()> {
        // Note: Moka doesn't support per-key TTL, but we can work around this
        // In a real implementation, you might use a different approach
        self.cache.insert(key.to_string(), value).await;
        self.sets.fetch_add(1, Ordering::Relaxed);
        
        // Schedule expiration (simplified approach)
        if ttl < self.config.time_to_live {
            let cache_clone = self.cache.clone();
            let key_clone = key.to_string();
            tokio::spawn(async move {
                tokio::time::sleep(ttl).await;
                cache_clone.invalidate(&key_clone).await;
            });
        }
        
        Ok(())
    }
    
    /// Get all keys in L1 cache
    pub async fn get_all_keys(&self) -> Vec<String> {
        // Moka doesn't have a direct way to get all keys
        // For now, we'll return an empty vector since this is mainly for sync operations
        // In production, you might want to maintain a separate set of keys or use a different approach
        Vec::new()
    }

    /// Check if key exists in L1 cache
    pub async fn contains_key(&self, key: &str) -> bool {
        self.cache.get(key).await.is_some()
    }

    /// Remove value from L1 cache
    pub async fn remove(&self, key: &str) -> Result<()> {
        self.cache.invalidate(key).await;
        Ok(())
    }
    
    /// Clear all values from L1 cache
    pub async fn clear(&self) -> Result<()> {
        self.cache.invalidate_all();
        Ok(())
    }
    
    /// Get current cache size
    pub async fn size(&self) -> u64 {
        self.cache.entry_count()
    }
    
    /// Get cache capacity
    pub fn capacity(&self) -> u64 {
        self.config.max_capacity
    }
    
    /// Get cache hit rate
    pub async fn hit_rate(&self) -> f64 {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let total = hits + misses;
        
        if total > 0 {
            hits as f64 / total as f64
        } else {
            0.0
        }
    }
    
    /// Health check for L1 cache
    pub async fn health_check(&self) -> bool {
        // Test basic cache operations
        let test_key = "health_check_l1";
        let test_value = serde_json::json!({"test": "l1_cache_health", "timestamp": chrono::Utc::now().to_rfc3339()});
        
        // Test set operation
        if let Err(e) = self.set(test_key, test_value.clone()).await {
            eprintln!("❌ L1 Cache set operation failed: {}", e);
            return false;
        }
        
        // Test get operation
        match self.get(test_key).await {
            Some(retrieved_value) => {
                if retrieved_value == test_value {
                    // Cleanup test key
                    let _ = self.remove(test_key).await;
                    println!("✅ L1 Cache health check passed");
                    true
                } else {
                    eprintln!("❌ L1 Cache retrieved value doesn't match set value");
                    false
                }
            }
            None => {
                eprintln!("❌ L1 Cache get operation failed - key not found");
                false
            }
        }
    }
    
    /// Get L1 cache statistics
    pub async fn get_statistics(&self) -> Result<serde_json::Value> {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let sets = self.sets.load(Ordering::Relaxed);
        let evictions = self.evictions.load(Ordering::Relaxed);
        let current_size = self.size().await;
        let hit_rate = self.hit_rate().await;
        
        Ok(serde_json::json!({
            "cache_type": "l1_moka",
            "status": "operational",
            "configuration": {
                "max_capacity": self.config.max_capacity,
                "ttl_seconds": self.config.time_to_live.as_secs(),
                "tti_seconds": self.config.time_to_idle.as_secs()
            },
            "statistics": {
                "current_size": current_size,
                "capacity_utilization_percent": (current_size as f64 / self.config.max_capacity as f64) * 100.0,
                "hits": hits,
                "misses": misses,
                "sets": sets,
                "evictions": evictions,
                "hit_rate_percent": hit_rate * 100.0
            },
            "performance": {
                "access_time": "sub_millisecond",
                "storage_type": "in_memory",
                "thread_safety": "yes"
            }
        }))
    }
    
    /// Get cache keys (for debugging - expensive operation)
    pub async fn get_keys(&self) -> Vec<String> {
        // Note: Moka doesn't provide a direct way to list all keys
        // This is a simplified implementation
        vec![] // In real implementation, you'd need a different approach
    }
    
    /// Force cache maintenance (cleanup expired entries)
    pub async fn maintain(&self) -> Result<()> {
        // Moka handles this automatically, but we can trigger maintenance
        self.cache.run_pending_tasks().await;
        Ok(())
    }
}
