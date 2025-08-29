//! L2 Cache - Redis Cache
//! 
//! Redis-based distributed cache for warm data storage with persistence.

use std::sync::Arc;
use std::time::Duration;
use anyhow::Result;
use redis::{Client, AsyncCommands};
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
    
    /// Set value with custom TTL
    pub async fn set_with_ttl(&self, key: &str, value: serde_json::Value, ttl: Duration) -> Result<()> {
        let json_str = serde_json::to_string(&value)?;
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        
        let _: () = conn.set_ex(key, json_str, ttl.as_secs()).await?;
        self.sets.fetch_add(1, Ordering::Relaxed);
        println!("💾 [L2] Cached '{}' with TTL {:?}", key, ttl);
        Ok(())
    }
    
    /// Remove value from cache
    pub async fn remove(&self, key: &str) -> Result<()> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let _: () = conn.del(key).await?;
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
}
