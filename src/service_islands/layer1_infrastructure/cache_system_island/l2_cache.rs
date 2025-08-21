//! L2 Cache - Redis Distributed Cache
//! 
//! Distributed cache using Redis for warm data with fallback capabilities.
//! Provides persistence and sharing across multiple application instances.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use std::collections::HashMap;
use std::sync::RwLock;
use anyhow::Result;
use serde_json;

/// L2 Cache configuration
pub struct L2CacheConfig {
    pub redis_url: String,
    pub max_connections: u32,
    pub default_ttl: Duration,
}

impl Default for L2CacheConfig {
    fn default() -> Self {
        Self {
            redis_url: "redis://127.0.0.1:6379".to_string(),
            max_connections: 10,
            default_ttl: Duration::from_secs(10800), // Increased from 3600s (1hr) to 10800s (3hrs)
        }
    }
}

/// L2 Cache implementation with Redis backend and fallback storage
pub struct L2Cache {
    config: L2CacheConfig,
    // Fallback storage when Redis is unavailable
    fallback_storage: Arc<RwLock<HashMap<String, (serde_json::Value, std::time::Instant)>>>,
    // Statistics
    hits: AtomicU64,
    misses: AtomicU64,
    sets: AtomicU64,
    errors: AtomicU64,
    redis_available: std::sync::atomic::AtomicBool,
}

impl L2Cache {
    /// Initialize L2 Cache with default configuration
    pub async fn new() -> Result<Self> {
        Self::new_with_config(L2CacheConfig::default()).await
    }

    /// Initialize L2 Cache with custom configuration
    pub async fn new_with_config(config: L2CacheConfig) -> Result<Self> {
        println!("🌐 Initializing L2 Cache (Redis + Fallback)...");
        
        let cache = L2Cache {
            config,
            fallback_storage: Arc::new(RwLock::new(HashMap::new())),
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
            sets: AtomicU64::new(0),
            errors: AtomicU64::new(0),
            redis_available: std::sync::atomic::AtomicBool::new(false),
        };

        // Test Redis connection
        if cache.test_connection().await {
            println!("  ✅ L2 Cache initialized with Redis backend");
        } else {
            println!("  ⚠️ L2 Cache initialized with fallback storage only");
        }

        Ok(cache)
    }

    /// Test Redis connection
    async fn test_connection(&self) -> bool {
        let client = match redis::Client::open(self.config.redis_url.as_str()) {
            Ok(client) => client,
            Err(e) => {
                println!("  ❌ Failed to create Redis client for L2 cache: {}", e);
                self.redis_available.store(false, Ordering::Relaxed);
                return false;
            }
        };

        match client.get_multiplexed_async_connection().await {
            Ok(mut conn) => {
                // Test with a simple PING command
                match redis::cmd("PING").query_async::<String>(&mut conn).await {
                    Ok(_) => {
                        println!("  ✅ L2 Cache Redis connection successful ({})", self.config.redis_url);
                        self.redis_available.store(true, Ordering::Relaxed);
                        true
                    }
                    Err(e) => {
                        println!("  ❌ L2 Cache Redis PING failed: {}", e);
                        self.redis_available.store(false, Ordering::Relaxed);
                        false
                    }
                }
            }
            Err(e) => {
                println!("  ❌ L2 Cache failed to connect to Redis: {}", e);
                self.redis_available.store(false, Ordering::Relaxed);
                false
            }
        }
    }

    /// Get value from L2 cache
    pub async fn get(&self, key: &str) -> Option<serde_json::Value> {
        if self.redis_available.load(Ordering::Relaxed) {
            match self.get_from_redis(key).await {
                Ok(value) => {
                    if value.is_some() {
                        self.hits.fetch_add(1, Ordering::Relaxed);
                    } else {
                        self.misses.fetch_add(1, Ordering::Relaxed);
                    }
                    value
                }
                Err(_) => {
                    self.errors.fetch_add(1, Ordering::Relaxed);
                    self.misses.fetch_add(1, Ordering::Relaxed);
                    None
                }
            }
        } else {
            // Use fallback storage
            let storage = self.fallback_storage.read().unwrap();
            if let Some((value, timestamp)) = storage.get(key) {
                // Check if value hasn't expired
                if timestamp.elapsed() < self.config.default_ttl {
                    self.hits.fetch_add(1, Ordering::Relaxed);
                    Some(value.clone())
                } else {
                    self.misses.fetch_add(1, Ordering::Relaxed);
                    None
                }
            } else {
                self.misses.fetch_add(1, Ordering::Relaxed);
                None
            }
        }
    }

    /// Set value in L2 cache
    pub async fn set(&self, key: String, value: serde_json::Value) -> bool {
        self.sets.fetch_add(1, Ordering::Relaxed);
        
        if self.redis_available.load(Ordering::Relaxed) {
            match self.set_in_redis(&key, &value, None).await {
                Ok(_) => true,
                Err(_) => {
                    self.errors.fetch_add(1, Ordering::Relaxed);
                    false
                }
            }
        } else {
            // Use fallback storage
            let mut storage = self.fallback_storage.write().unwrap();
            storage.insert(key, (value, std::time::Instant::now()));
            true
        }
    }

    /// Set value in L2 cache with custom TTL
    pub async fn set_with_ttl(&self, key: String, value: serde_json::Value, ttl: Duration) -> bool {
        self.sets.fetch_add(1, Ordering::Relaxed);
        
        if self.redis_available.load(Ordering::Relaxed) {
            match self.set_in_redis(&key, &value, Some(ttl)).await {
                Ok(_) => true,
                Err(_) => {
                    self.errors.fetch_add(1, Ordering::Relaxed);
                    false
                }
            }
        } else {
            // Use fallback storage
            let mut storage = self.fallback_storage.write().unwrap();
            storage.insert(key, (value, std::time::Instant::now()));
            true
        }
    }

    /// Remove value from L2 cache
    pub async fn remove(&self, key: &str) -> bool {
        if self.redis_available.load(Ordering::Relaxed) {
            // TODO: Implement Redis delete when connection is available
            false
        } else {
            // Use fallback storage
            let mut storage = self.fallback_storage.write().unwrap();
            storage.remove(key).is_some()
        }
    }

    /// Get value from Redis
    async fn get_from_redis(&self, key: &str) -> Result<Option<serde_json::Value>> {
        let client = redis::Client::open(self.config.redis_url.as_str())?;
        let mut conn = client.get_multiplexed_async_connection().await?;
        
        let result: Option<String> = redis::cmd("GET")
            .arg(key)
            .query_async(&mut conn)
            .await?;
        
        match result {
            Some(json_str) => {
                match serde_json::from_str(&json_str) {
                    Ok(value) => Ok(Some(value)),
                    Err(_) => Ok(Some(serde_json::Value::String(json_str))),
                }
            }
            None => Ok(None),
        }
    }

    /// Set value in Redis
    async fn set_in_redis(&self, key: &str, value: &serde_json::Value, ttl: Option<Duration>) -> Result<()> {
        let client = redis::Client::open(self.config.redis_url.as_str())?;
        let mut conn = client.get_multiplexed_async_connection().await?;
        
        let json_str = value.to_string();
        let ttl_seconds = ttl.unwrap_or(self.config.default_ttl).as_secs();
        
        redis::cmd("SETEX")
            .arg(key)
            .arg(ttl_seconds)
            .arg(json_str)
            .query_async::<()>(&mut conn)
            .await?;
        
        Ok(())
    }

    /// Check if key exists in L2 cache
    pub async fn exists(&self, key: &str) -> bool {
        if self.redis_available.load(Ordering::Relaxed) {
            // TODO: Implement Redis exists when connection is available
            false
        } else {
            // Use fallback storage
            let storage = self.fallback_storage.read().unwrap();
            if let Some((_, timestamp)) = storage.get(key) {
                timestamp.elapsed() < self.config.default_ttl
            } else {
                false
            }
        }
    }

    /// Clear all entries from L2 cache
    pub async fn clear(&self) -> bool {
        if self.redis_available.load(Ordering::Relaxed) {
            // TODO: Implement Redis flush when connection is available
            false
        } else {
            // Use fallback storage
            let mut storage = self.fallback_storage.write().unwrap();
            storage.clear();
            true
        }
    }

    /// Get cache size
    pub async fn len(&self) -> usize {
        if self.redis_available.load(Ordering::Relaxed) {
            // TODO: Implement Redis size when connection is available
            0
        } else {
            // Use fallback storage
            let storage = self.fallback_storage.read().unwrap();
            // Count only non-expired entries
            storage.iter()
                .filter(|(_, (_, timestamp))| timestamp.elapsed() < self.config.default_ttl)
                .count()
        }
    }

    /// Get cache statistics
    pub fn get_stats(&self) -> L2CacheStats {
        L2CacheStats {
            hits: self.hits.load(Ordering::Relaxed),
            misses: self.misses.load(Ordering::Relaxed),
            sets: self.sets.load(Ordering::Relaxed),
            errors: self.errors.load(Ordering::Relaxed),
            redis_available: self.redis_available.load(Ordering::Relaxed),
            size: {
                if self.redis_available.load(Ordering::Relaxed) {
                    0 // TODO: Get Redis size
                } else {
                    let storage = self.fallback_storage.read().unwrap();
                    storage.len() as u64
                }
            },
        }
    }

    /// Cleanup expired entries from fallback storage
    pub async fn cleanup_expired(&self) {
        if !self.redis_available.load(Ordering::Relaxed) {
            let mut storage = self.fallback_storage.write().unwrap();
            let now = std::time::Instant::now();
            storage.retain(|_, (_, timestamp)| now.duration_since(*timestamp) < self.config.default_ttl);
        }
    }

    /// Health check method
    pub async fn health_check(&self) -> bool {
        if self.redis_available.load(Ordering::Relaxed) {
            // TODO: Implement Redis health check
            false
        } else {
            // Test fallback storage
            let test_key = "_health_check_test";
            let test_value = serde_json::json!({"test": true, "timestamp": std::time::SystemTime::now()});
            
            // Test set
            if self.set(test_key.to_string(), test_value.clone()).await {
                // Test get
                if let Some(retrieved) = self.get(test_key).await {
                    if retrieved == test_value {
                        // Test remove
                        self.remove(test_key).await;
                        println!("✅ L2 Cache health check passed (fallback mode)");
                        return true;
                    }
                }
            }
            
            eprintln!("❌ L2 Cache health check failed");
            false
        }
    }
}

/// L2 Cache statistics
#[derive(Debug, Clone, serde::Serialize)]
pub struct L2CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub sets: u64,
    pub errors: u64,
    pub redis_available: bool,
    pub size: u64,
}
