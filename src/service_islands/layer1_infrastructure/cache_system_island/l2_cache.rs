//! L2 Cache - Redis Cache
//! 
//! Redis-based distributed cache for warm data storage with persistence.

use std::sync::Arc;
use std::time::Duration;
use anyhow::Result;
use redis::{Client, AsyncCommands, Connection};
use serde_json;
use std::sync::atomic::{AtomicU64, Ordering};

/// L2 Cache using Redis
pub struct L2Cache {
    /// Redis client
    client: Client,
    /// Connection pool (using a simple approach)
    /// Hit counter
    hits: Arc<AtomicU64>,
    /// Miss counter
    misses: Arc<AtomicU64>,
    /// Set counter
    sets: Arc<AtomicU64>,
}

impl L2Cache {
    /// Create new L2 cache
    pub async fn new() -> Result<Self> {
        println!("  🔴 Initializing L2 Cache (Redis)...");
        
        // Try to connect to Redis
        let redis_url = std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
            
        let client = Client::open(redis_url.as_str())?;
        
        // Test connection
        let mut conn = client.get_multiplexed_async_connection().await?;
        let _: String = redis::cmd("PING").query_async(&mut conn).await?;
        
        println!("  ✅ L2 Cache connected to Redis at {}", redis_url);
        
        Ok(Self {
            client,
            hits: Arc::new(AtomicU64::new(0)),
            misses: Arc::new(AtomicU64::new(0)),
            sets: Arc::new(AtomicU64::new(0)),
        })
    }
    
    /// Get value from L2 cache
    pub async fn get(&self, key: &str) -> Option<serde_json::Value> {
        match self.client.get_multiplexed_async_connection().await {
            Ok(mut conn) => {
                match conn.get::<_, String>(key).await {
                    Ok(json_str) => {
                        match serde_json::from_str(&json_str) {
                            Ok(value) => {
                                self.hits.fetch_add(1, Ordering::Relaxed);
                                Some(value)
                            }
                            Err(_) => {
                                self.misses.fetch_add(1, Ordering::Relaxed);
                                None
                            }
                        }
                    }
                    Err(_) => {
                        self.misses.fetch_add(1, Ordering::Relaxed);
                        None
                    }
                }
            }
            Err(_) => {
                self.misses.fetch_add(1, Ordering::Relaxed);
                None
            }
        }
    }
    
    /// Set value in L2 cache with default TTL (1 hour)
    pub async fn set(&self, key: &str, value: serde_json::Value) -> Result<()> {
        self.set_with_ttl(key, value, Duration::from_secs(3600)).await
    }
    
    /// Set value with custom TTL
    pub async fn set_with_ttl(&self, key: &str, value: serde_json::Value, ttl: Duration) -> Result<()> {
        let json_str = serde_json::to_string(&value)?;
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        
        let _: () = conn.set_ex(key, json_str, ttl.as_secs()).await?;
        self.sets.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }
    
    /// Remove value from cache
    pub async fn remove(&self, key: &str) -> Result<()> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let _: () = conn.del(key).await?;
        Ok(())
    }
    
    /// Clear entire cache (use with caution)
    pub async fn clear(&self) -> Result<()> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let _: () = redis::cmd("FLUSHDB").query_async(&mut conn).await?;
        println!("🧹 L2 Cache (Redis) cleared");
        Ok(())
    }
    
    /// Health check
    pub async fn health_check(&self) -> bool {
        let test_key = "health_check_l2";
        let test_value = serde_json::json!({"test": true, "timestamp": chrono::Utc::now().to_rfc3339()});
        
        match self.set_with_ttl(test_key, test_value.clone(), Duration::from_secs(10)).await {
            Ok(_) => {
                match self.get(test_key).await {
                    Some(retrieved) => {
                        let _ = self.remove(test_key).await;
                        retrieved["test"].as_bool().unwrap_or(false)
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
        
        // Try to get Redis info
        let mut redis_info = serde_json::json!({});
        if let Ok(mut conn) = self.client.get_multiplexed_async_connection().await {
            if let Ok(info_str) = redis::cmd("INFO").arg("memory").query_async::<String>(&mut conn).await {
                // Parse basic memory info from Redis INFO response
                if let Some(used_memory_line) = info_str.lines().find(|line| line.starts_with("used_memory:")) {
                    if let Some(memory_bytes) = used_memory_line.split(':').nth(1) {
                        redis_info["used_memory_bytes"] = serde_json::Value::String(memory_bytes.to_string());
                    }
                }
            }
        }
        
        serde_json::json!({
            "type": "L2_Redis",
            "hits": hits,
            "misses": misses,
            "sets": sets,
            "total_requests": total_requests,
            "hit_rate_percent": hit_rate,
            "redis_info": redis_info
        })
    }
}
