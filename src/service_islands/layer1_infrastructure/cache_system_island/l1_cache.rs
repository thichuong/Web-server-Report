//! L1 Cache - Moka In-Memory Cache
//! 
//! High-performance in-memory cache using Moka for hot data storage.

use std::sync::Arc;
use std::time::Duration;
use anyhow::Result;
use serde_json;
use moka::future::Cache;
use std::sync::atomic::{AtomicU64, Ordering};

/// L1 Cache using Moka
pub struct L1Cache {
    /// Moka cache instance
    cache: Cache<String, serde_json::Value>,
    /// Hit counter
    hits: Arc<AtomicU64>,
    /// Miss counter  
    misses: Arc<AtomicU64>,
    /// Set counter
    sets: Arc<AtomicU64>,
}

impl L1Cache {
    /// Create new L1 cache
    pub async fn new() -> Result<Self> {
        println!("  🚀 Initializing L1 Cache (Moka)...");
        
        let cache = Cache::builder()
            .max_capacity(2000) // 2000 entries max
            .time_to_live(Duration::from_secs(300)) // 5 minutes default TTL
            .time_to_idle(Duration::from_secs(120)) // 2 minutes idle time
            .build();
            
        println!("  ✅ L1 Cache initialized with 2000 capacity, 5min TTL");
        
        Ok(Self {
            cache,
            hits: Arc::new(AtomicU64::new(0)),
            misses: Arc::new(AtomicU64::new(0)),
            sets: Arc::new(AtomicU64::new(0)),
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
    
    /// Set value in L1 cache with default TTL
    pub async fn set(&self, key: &str, value: serde_json::Value) -> Result<()> {
        self.cache.insert(key.to_string(), value).await;
        self.sets.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }
    
    /// Set value with custom TTL
    pub async fn set_with_ttl(&self, key: &str, value: serde_json::Value, ttl: Duration) -> Result<()> {
        // Note: Moka doesn't support per-key TTL, so we use the global TTL
        // For different TTLs, we'd need separate cache instances
        self.cache.insert(key.to_string(), value).await;
        self.sets.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }
    
    /// Remove value from cache
    pub async fn remove(&self, key: &str) -> Result<()> {
        self.cache.remove(key).await;
        Ok(())
    }
    
    /// Clear entire cache
    pub async fn clear(&self) -> Result<()> {
        self.cache.invalidate_all();
        println!("🧹 L1 Cache cleared");
        Ok(())
    }
    
    /// Health check
    pub async fn health_check(&self) -> bool {
        // Test basic functionality
        let test_key = "health_check_l1";
        let test_value = serde_json::json!({"test": true});
        
        match self.set(test_key, test_value.clone()).await {
            Ok(_) => {
                match self.get(test_key).await {
                    Some(retrieved) => {
                        let _ = self.remove(test_key).await;
                        retrieved == test_value
                    }
                    None => false
                }
            }
            Err(_) => false
        }
    }
    
    /// Get cache statistics
    pub async fn get_statistics(&self) -> serde_json::Value {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let sets = self.sets.load(Ordering::Relaxed);
        let total_requests = hits + misses;
        let hit_rate = if total_requests > 0 {
            (hits as f64 / total_requests as f64) * 100.0
        } else {
            0.0
        };
        
        serde_json::json!({
            "type": "L1_Moka",
            "entry_count": self.cache.entry_count(),
            "hits": hits,
            "misses": misses,
            "sets": sets,
            "total_requests": total_requests,
            "hit_rate_percent": hit_rate,
            "weighted_size": self.cache.weighted_size()
        })
    }
}
