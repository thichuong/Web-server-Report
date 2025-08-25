//! L1 Cache - Moka In-Memory Cache
//! 
//! High-performance in-memory cache using Moka for hot data storage.

use std::sync::Arc;
use std::time::Duration;
use std::future::Future;
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
    /// Coalesced requests counter (requests that waited for an ongoing computation)
    coalesced_requests: Arc<AtomicU64>,
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
            coalesced_requests: Arc::new(AtomicU64::new(0)),
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
    pub async fn set(&self, key: &str, value: serde_json::Value, _ttl: Duration) -> Result<()> {
        self.cache.insert(key.to_string(), value).await;
        self.sets.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }
    
    /// Set value with custom TTL
    pub async fn set_with_ttl(&self, key: &str, value: serde_json::Value, _ttl: Duration) -> Result<()> {
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
    
    /// Health check
    pub async fn health_check(&self) -> bool {
        // Test basic functionality
        let test_key = "health_check_l1";
        let test_value = serde_json::json!({"test": true});
        
        match self.set(test_key, test_value.clone(), Duration::from_secs(60)).await {
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

    /// Get value from cache or compute if not exists (Cache Stampede Protection)
    /// 
    /// This method prevents Cache Stampede by using Moka's built-in request coalescing.
    /// When multiple concurrent requests ask for the same key that's not in cache,
    /// only the first request will execute the compute function, while others wait
    /// for the result.
    /// 
    /// # Arguments
    /// * `key` - Cache key
    /// * `compute_fn` - Async function to compute the value if not in cache
    /// 
    /// # Example
    /// ```rust
    /// let value = cache.get_or_compute_with("expensive_data", || async {
    ///     // This expensive computation will only run once even with concurrent requests
    ///     fetch_data_from_database().await
    /// }).await?;
    /// ```
    pub async fn get_or_compute_with<F, Fut>(
        &self,
        key: &str,
        compute_fn: F,
    ) -> Result<serde_json::Value>
    where
        F: FnOnce() -> Fut + Send,
        Fut: Future<Output = Result<serde_json::Value>> + Send,
    {
        // Check if already in cache first (fast path)
        if let Some(cached_value) = self.get(key).await {
            return Ok(cached_value);
        }

        // Use a simpler approach with get_with that returns the computed value directly
        let result = self.cache.get_with(key.to_string(), async {
            println!("🔄 Computing fresh data for key: '{}' (Cache Stampede protected)", key);
            compute_fn().await.unwrap_or_else(|e| {
                eprintln!("❌ Computation failed for key '{}': {}", key, e);
                // Return a default error value that can be detected
                serde_json::json!({
                    "error": true,
                    "message": format!("Computation failed: {}", e)
                })
            })
        }).await;
        
        // Check if the result is an error
        if result.get("error").and_then(|v| v.as_bool()).unwrap_or(false) {
            let error_msg = result.get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown computation error");
            return Err(anyhow::anyhow!("{}", error_msg));
        }
        
        self.hits.fetch_add(1, Ordering::Relaxed);
        Ok(result)
    }

    /// Get statistics about cache performance and coalescing
    pub fn get_stats(&self) -> CacheStats {
        CacheStats {
            hits: self.hits.load(Ordering::Relaxed),
            misses: self.misses.load(Ordering::Relaxed),
            sets: self.sets.load(Ordering::Relaxed),
            coalesced_requests: self.coalesced_requests.load(Ordering::Relaxed),
            size: self.cache.entry_count(),
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub sets: u64,
    pub coalesced_requests: u64,
    pub size: u64,
}
