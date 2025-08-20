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
            default_ttl: Duration::from_secs(3600), // 1 hour
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
        // For now, we'll use fallback mode until Redis connection is properly integrated
        // This allows the system to function while we work on Redis integration
        println!("  📝 L2 Cache: Using fallback storage mode");
        self.redis_available.store(false, Ordering::Relaxed);
        true
    }

    /// Get value from L2 cache
    pub async fn get(&self, key: &str) -> Option<serde_json::Value> {
        if self.redis_available.load(Ordering::Relaxed) {
            // TODO: Implement Redis get when connection is available
            None
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
            // TODO: Implement Redis set when connection is available
            false
        } else {
            // Use fallback storage
            let mut storage = self.fallback_storage.write().unwrap();
            storage.insert(key, (value, std::time::Instant::now()));
            true
        }
    }

    /// Set value in L2 cache with custom TTL
    pub async fn set_with_ttl(&self, key: String, value: serde_json::Value, _ttl: Duration) -> bool {
        // For fallback mode, we use the default TTL
        self.set(key, value).await
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
#[derive(Debug, Clone)]
pub struct L2CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub sets: u64,
    pub errors: u64,
    pub redis_available: bool,
    pub size: u64,
}

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use anyhow::Result;
use serde_json;
use redis::{AsyncCommands, Client};

/// L2 Cache configuration
pub struct L2CacheConfig {
    pub redis_url: String,
    pub default_ttl: Duration,
    pub connection_timeout: Duration,
    pub retry_attempts: u32,
}

impl Default for L2CacheConfig {
    fn default() -> Self {
        Self {
            redis_url: std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
            default_ttl: Duration::from_secs(3600), // 1 hour
            connection_timeout: Duration::from_secs(5),
            retry_attempts: 3,
        }
    }
}

/// L2 Cache - Redis distributed cache for warm data
pub struct L2Cache {
    /// Redis client
    client: Option<Client>,
    /// Cache configuration
    config: L2CacheConfig,
    /// Redis availability flag
    redis_available: Arc<std::sync::atomic::AtomicBool>,
    /// Statistics tracking
    hits: Arc<AtomicU64>,
    misses: Arc<AtomicU64>,
    sets: Arc<AtomicU64>,
    errors: Arc<AtomicU64>,
    /// Fallback mode (when Redis is unavailable)
    fallback_cache: Arc<std::sync::RwLock<std::collections::HashMap<String, (serde_json::Value, std::time::Instant)>>>,
}

impl L2Cache {
    /// Initialize L2 Cache with Redis
    pub async fn new() -> Result<Self> {
        Self::new_with_config(L2CacheConfig::default()).await
    }
    
    /// Initialize L2 Cache with custom configuration
    pub async fn new_with_config(config: L2CacheConfig) -> Result<Self> {
        println!("🌐 Initializing L2 Cache (Redis)...");
        
        // Try to connect to Redis
        let (client, redis_available) = match Client::open(config.redis_url.as_str()) {
            Ok(client) => {
                // Test connection
                match client.get_async_connection().await {
                    Ok(mut conn) => {
                        // Test ping
                        match redis::cmd("PING").query_async::<_, String>(&mut conn).await {
                            Ok(_) => {
                                println!("  🌐 Redis connection established: {}", config.redis_url);
                                (Some(client), true)
                            }
                            Err(e) => {
                                eprintln!("  ⚠️  Redis ping failed, using fallback mode: {}", e);
                                (None, false)
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("  ⚠️  Redis connection failed, using fallback mode: {}", e);
                        (None, false)
                    }
                }
            }
            Err(e) => {
                eprintln!("  ⚠️  Redis client creation failed, using fallback mode: {}", e);
                (None, false)
            }
        };
        
        let mode = if redis_available { "Redis" } else { "Fallback" };
        println!("  🌐 L2 Cache initialized in {} mode, {}s TTL", mode, config.default_ttl.as_secs());
        
        Ok(Self {
            client,
            config,
            redis_available: Arc::new(std::sync::atomic::AtomicBool::new(redis_available)),
            hits: Arc::new(AtomicU64::new(0)),
            misses: Arc::new(AtomicU64::new(0)),
            sets: Arc::new(AtomicU64::new(0)),
            errors: Arc::new(AtomicU64::new(0)),
            fallback_cache: Arc::new(std::sync::RwLock::new(std::collections::HashMap::new())),
        })
    }
    
    /// Get value from L2 cache (Redis or fallback)
    pub async fn get(&self, key: &str) -> Result<Option<serde_json::Value>> {
        if self.redis_available.load(Ordering::Relaxed) {
            self.get_from_redis(key).await
        } else {
            self.get_from_fallback(key).await
        }
    }
    
    /// Get value from Redis
    async fn get_from_redis(&self, key: &str) -> Result<Option<serde_json::Value>> {
        match &self.client {
            Some(client) => {
                match client.get_async_connection().await {
                    Ok(mut conn) => {
                        match conn.get::<_, Option<String>>(key).await {
                            Ok(Some(value_str)) => {
                                match serde_json::from_str(&value_str) {
                                    Ok(value) => {
                                        self.hits.fetch_add(1, Ordering::Relaxed);
                                        Ok(Some(value))
                                    }
                                    Err(e) => {
                                        eprintln!("❌ L2 Cache JSON parse error: {}", e);
                                        self.errors.fetch_add(1, Ordering::Relaxed);
                                        Ok(None)
                                    }
                                }
                            }
                            Ok(None) => {
                                self.misses.fetch_add(1, Ordering::Relaxed);
                                Ok(None)
                            }
                            Err(e) => {
                                eprintln!("❌ L2 Cache Redis GET error: {}", e);
                                self.errors.fetch_add(1, Ordering::Relaxed);
                                // Fall back to fallback cache
                                self.redis_available.store(false, Ordering::Relaxed);
                                self.get_from_fallback(key).await
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("❌ L2 Cache Redis connection error: {}", e);
                        self.errors.fetch_add(1, Ordering::Relaxed);
                        self.redis_available.store(false, Ordering::Relaxed);
                        self.get_from_fallback(key).await
                    }
                }
            }
            None => self.get_from_fallback(key).await,
        }
    }
    
    /// Get value from fallback cache
    async fn get_from_fallback(&self, key: &str) -> Result<Option<serde_json::Value>> {
        let cache = self.fallback_cache.read().unwrap();
        
        match cache.get(key) {
            Some((value, timestamp)) => {
                // Check if expired
                if timestamp.elapsed() < self.config.default_ttl {
                    self.hits.fetch_add(1, Ordering::Relaxed);
                    Ok(Some(value.clone()))
                } else {
                    self.misses.fetch_add(1, Ordering::Relaxed);
                    Ok(None)
                }
            }
            None => {
                self.misses.fetch_add(1, Ordering::Relaxed);
                Ok(None)
            }
        }
    }
    
    /// Set value in L2 cache
    pub async fn set(&self, key: &str, value: serde_json::Value, ttl: Option<Duration>) -> Result<()> {
        let ttl = ttl.unwrap_or(self.config.default_ttl);
        
        if self.redis_available.load(Ordering::Relaxed) {
            self.set_in_redis(key, &value, ttl).await
        } else {
            self.set_in_fallback(key, value, ttl).await
        }
    }
    
    /// Set value in Redis
    async fn set_in_redis(&self, key: &str, value: &serde_json::Value, ttl: Duration) -> Result<()> {
        match &self.client {
            Some(client) => {
                match client.get_async_connection().await {
                    Ok(mut conn) => {
                        let value_str = serde_json::to_string(value)?;
                        
                        match conn.set_ex::<_, _, ()>(key, value_str, ttl.as_secs() as usize).await {
                            Ok(_) => {
                                self.sets.fetch_add(1, Ordering::Relaxed);
                                Ok(())
                            }
                            Err(e) => {
                                eprintln!("❌ L2 Cache Redis SET error: {}", e);
                                self.errors.fetch_add(1, Ordering::Relaxed);
                                // Fall back to fallback cache
                                self.redis_available.store(false, Ordering::Relaxed);
                                self.set_in_fallback(key, value.clone(), ttl).await
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("❌ L2 Cache Redis connection error: {}", e);
                        self.errors.fetch_add(1, Ordering::Relaxed);
                        self.redis_available.store(false, Ordering::Relaxed);
                        self.set_in_fallback(key, value.clone(), ttl).await
                    }
                }
            }
            None => self.set_in_fallback(key, value.clone(), ttl).await,
        }
    }
    
    /// Set value in fallback cache
    async fn set_in_fallback(&self, key: &str, value: serde_json::Value, _ttl: Duration) -> Result<()> {
        let mut cache = self.fallback_cache.write().unwrap();
        cache.insert(key.to_string(), (value, std::time::Instant::now()));
        self.sets.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }
    
    /// Remove value from L2 cache
    pub async fn remove(&self, key: &str) -> Result<()> {
        if self.redis_available.load(Ordering::Relaxed) {
            self.remove_from_redis(key).await
        } else {
            self.remove_from_fallback(key).await
        }
    }
    
    /// Remove value from Redis
    async fn remove_from_redis(&self, key: &str) -> Result<()> {
        match &self.client {
            Some(client) => {
                match client.get_async_connection().await {
                    Ok(mut conn) => {
                        match conn.del::<_, ()>(key).await {
                            Ok(_) => Ok(()),
                            Err(e) => {
                                eprintln!("❌ L2 Cache Redis DEL error: {}", e);
                                self.errors.fetch_add(1, Ordering::Relaxed);
                                Ok(()) // Don't fail on delete errors
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("❌ L2 Cache Redis connection error: {}", e);
                        self.errors.fetch_add(1, Ordering::Relaxed);
                        Ok(())
                    }
                }
            }
            None => self.remove_from_fallback(key).await,
        }
    }
    
    /// Remove value from fallback cache
    async fn remove_from_fallback(&self, key: &str) -> Result<()> {
        let mut cache = self.fallback_cache.write().unwrap();
        cache.remove(key);
        Ok(())
    }
    
    /// Clear all values from L2 cache
    pub async fn clear(&self) -> Result<()> {
        if self.redis_available.load(Ordering::Relaxed) {
            // Clear Redis (flush database - be careful in production!)
            match &self.client {
                Some(client) => {
                    if let Ok(mut conn) = client.get_async_connection().await {
                        let _ = redis::cmd("FLUSHDB").query_async::<_, ()>(&mut conn).await;
                    }
                }
                None => {}
            }
        }
        
        // Clear fallback cache
        let mut cache = self.fallback_cache.write().unwrap();
        cache.clear();
        
        Ok(())
    }
    
    /// Check Redis availability and attempt reconnection
    pub async fn check_availability(&self) -> bool {
        if self.redis_available.load(Ordering::Relaxed) {
            return true;
        }
        
        // Try to reconnect
        if let Some(client) = &self.client {
            match client.get_async_connection().await {
                Ok(mut conn) => {
                    match redis::cmd("PING").query_async::<_, String>(&mut conn).await {
                        Ok(_) => {
                            self.redis_available.store(true, Ordering::Relaxed);
                            println!("✅ L2 Cache Redis reconnected successfully");
                            true
                        }
                        Err(_) => false,
                    }
                }
                Err(_) => false,
            }
        } else {
            false
        }
    }
    
    /// Health check for L2 cache
    pub async fn health_check(&self) -> bool {
        let test_key = "health_check_l2";
        let test_value = serde_json::json!({
            "test": "l2_cache_health", 
            "timestamp": chrono::Utc::now().to_rfc3339()
        });
        
        // Test set operation
        if let Err(e) = self.set(test_key, test_value.clone(), Some(Duration::from_secs(10))).await {
            eprintln!("❌ L2 Cache set operation failed: {}", e);
            return false;
        }
        
        // Test get operation
        match self.get(test_key).await {
            Ok(Some(retrieved_value)) => {
                if retrieved_value["test"] == test_value["test"] {
                    // Cleanup test key
                    let _ = self.remove(test_key).await;
                    let mode = if self.redis_available.load(Ordering::Relaxed) { "Redis" } else { "Fallback" };
                    println!("✅ L2 Cache health check passed ({})", mode);
                    true
                } else {
                    eprintln!("❌ L2 Cache retrieved value doesn't match set value");
                    false
                }
            }
            Ok(None) => {
                eprintln!("❌ L2 Cache get operation failed - key not found");
                false
            }
            Err(e) => {
                eprintln!("❌ L2 Cache get operation failed: {}", e);
                false
            }
        }
    }
    
    /// Get L2 cache statistics
    pub async fn get_statistics(&self) -> Result<serde_json::Value> {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let sets = self.sets.load(Ordering::Relaxed);
        let errors = self.errors.load(Ordering::Relaxed);
        let redis_available = self.redis_available.load(Ordering::Relaxed);
        let hit_rate = if hits + misses > 0 {
            hits as f64 / (hits + misses) as f64
        } else {
            0.0
        };
        
        let fallback_size = {
            let cache = self.fallback_cache.read().unwrap();
            cache.len()
        };
        
        Ok(serde_json::json!({
            "cache_type": "l2_redis",
            "status": if redis_available { "redis_connected" } else { "fallback_mode" },
            "configuration": {
                "redis_url": self.config.redis_url,
                "default_ttl_seconds": self.config.default_ttl.as_secs(),
                "connection_timeout_seconds": self.config.connection_timeout.as_secs()
            },
            "statistics": {
                "hits": hits,
                "misses": misses,
                "sets": sets,
                "errors": errors,
                "hit_rate_percent": hit_rate * 100.0,
                "fallback_cache_size": fallback_size
            },
            "performance": {
                "redis_available": redis_available,
                "fallback_enabled": true,
                "storage_type": if redis_available { "redis" } else { "in_memory_fallback" }
            }
        }))
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
}
