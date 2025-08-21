//! Cache Manager - Unified Cache Operations
//! 
//! Manages operations across L1 (Moka) and L2 (Redis) caches with intelligent fallback.

use std::sync::Arc;
use std::time::Duration;
use anyhow::Result;
use serde_json;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

use super::l1_cache::L1Cache;
use super::l2_cache::L2Cache;

/// Cache strategies for different data types
#[derive(Debug, Clone)]
pub enum CacheStrategy {
    /// Real-time data - 30 seconds TTL
    RealTime,
    /// Short-term data - 5 minutes TTL  
    ShortTerm,
    /// Medium-term data - 1 hour TTL
    MediumTerm,
    /// Long-term data - 3 hours TTL
    LongTerm,
    /// Custom TTL
    Custom(Duration),
    /// Default strategy (5 minutes)
    Default,
}

impl CacheStrategy {
    /// Convert strategy to duration
    pub fn to_duration(&self) -> Duration {
        match self {
            Self::RealTime => Duration::from_secs(30),
            Self::ShortTerm => Duration::from_secs(300), // 5 minutes
            Self::MediumTerm => Duration::from_secs(3600), // 1 hour
            Self::LongTerm => Duration::from_secs(10800), // 3 hours
            Self::Custom(duration) => *duration,
            Self::Default => Duration::from_secs(300),
        }
    }
}

/// Cache Manager - Unified operations across L1 and L2
pub struct CacheManager {
    /// L1 Cache (Moka)
    l1_cache: Arc<L1Cache>,
    /// L2 Cache (Redis)
    l2_cache: Arc<L2Cache>,
    /// Statistics
    total_requests: Arc<AtomicU64>,
    l1_hits: Arc<AtomicU64>,
    l2_hits: Arc<AtomicU64>,
    misses: Arc<AtomicU64>,
    promotions: Arc<AtomicUsize>,
}

impl CacheManager {
    /// Create new cache manager
    pub async fn new(l1_cache: Arc<L1Cache>, l2_cache: Arc<L2Cache>) -> Result<Self> {
        println!("  🎯 Initializing Cache Manager...");
        
        Ok(Self {
            l1_cache,
            l2_cache,
            total_requests: Arc::new(AtomicU64::new(0)),
            l1_hits: Arc::new(AtomicU64::new(0)),
            l2_hits: Arc::new(AtomicU64::new(0)),
            misses: Arc::new(AtomicU64::new(0)),
            promotions: Arc::new(AtomicUsize::new(0)),
        })
    }
    
    /// Get value from cache (L1 -> L2 -> miss)
    pub async fn get(&self, key: &str) -> Result<Option<serde_json::Value>> {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        
        // Try L1 first
        if let Some(value) = self.l1_cache.get(key).await {
            self.l1_hits.fetch_add(1, Ordering::Relaxed);
            return Ok(Some(value));
        }
        
        // Try L2
        if let Some(value) = self.l2_cache.get(key).await {
            self.l2_hits.fetch_add(1, Ordering::Relaxed);
            
            // Promote to L1 for faster access next time
            if let Err(_) = self.l1_cache.set(key, value.clone()).await {
                // L1 promotion failed, but we still have the data
                eprintln!("⚠️ Failed to promote key '{}' to L1 cache", key);
            } else {
                self.promotions.fetch_add(1, Ordering::Relaxed);
            }
            
            return Ok(Some(value));
        }
        
        // Cache miss
        self.misses.fetch_add(1, Ordering::Relaxed);
        Ok(None)
    }
    
    /// Set value with default strategy (both L1 and L2)
    pub async fn set(&self, key: &str, value: serde_json::Value) -> Result<()> {
        self.set_with_strategy(key, value, CacheStrategy::Default).await
    }
    
    /// Set value with specific cache strategy
    pub async fn set_with_strategy(&self, key: &str, value: serde_json::Value, strategy: CacheStrategy) -> Result<()> {
        let ttl = strategy.to_duration();
        
        // Store in both L1 and L2
        let l1_result = self.l1_cache.set_with_ttl(key, value.clone(), ttl).await;
        let l2_result = self.l2_cache.set_with_ttl(key, value, ttl).await;
        
        // Return success if at least one cache succeeded
        match (l1_result, l2_result) {
            (Ok(_), Ok(_)) => {
                // Both succeeded
                Ok(())
            }
            (Ok(_), Err(_)) => {
                // L1 succeeded, L2 failed
                eprintln!("⚠️ L2 cache set failed for key '{}', continuing with L1", key);
                Ok(())
            }
            (Err(_), Ok(_)) => {
                // L1 failed, L2 succeeded
                eprintln!("⚠️ L1 cache set failed for key '{}', continuing with L2", key);
                Ok(())
            }
            (Err(e1), Err(_e2)) => {
                // Both failed
                Err(anyhow::anyhow!("Both L1 and L2 cache set failed for key '{}': {}", key, e1))
            }
        }
    }
    
    /// Remove value from both caches
    pub async fn remove(&self, key: &str) -> Result<()> {
        let l1_result = self.l1_cache.remove(key).await;
        let l2_result = self.l2_cache.remove(key).await;
        
        // Log errors but don't fail if one cache removal fails
        if let Err(e) = l1_result {
            eprintln!("⚠️ L1 cache remove failed for key '{}': {}", key, e);
        }
        if let Err(e) = l2_result {
            eprintln!("⚠️ L2 cache remove failed for key '{}': {}", key, e);
        }
        
        Ok(())
    }
    
    /// Clear both caches
    pub async fn clear(&self) -> Result<()> {
        let l1_result = self.l1_cache.clear().await;
        let l2_result = self.l2_cache.clear().await;
        
        match (l1_result, l2_result) {
            (Ok(_), Ok(_)) => Ok(()),
            (Ok(_), Err(e)) => {
                eprintln!("⚠️ L2 cache clear failed: {}", e);
                Ok(()) // L1 cleared successfully
            }
            (Err(e), Ok(_)) => {
                eprintln!("⚠️ L1 cache clear failed: {}", e);
                Ok(()) // L2 cleared successfully
            }
            (Err(e1), Err(e2)) => {
                Err(anyhow::anyhow!("Both cache clear failed - L1: {}, L2: {}", e1, e2))
            }
        }
    }
    
    /// Health check for cache manager
    pub async fn health_check(&self) -> bool {
        let l1_ok = self.l1_cache.health_check().await;
        let l2_ok = self.l2_cache.health_check().await;
        
        // Cache manager is healthy if at least L1 is working
        if l1_ok {
            if !l2_ok {
                println!("  ⚠️ Cache Manager: L1 OK, L2 degraded");
            } else {
                println!("  ✅ Cache Manager: Both L1 and L2 OK");
            }
            true
        } else {
            println!("  ❌ Cache Manager: L1 failed, system degraded");
            false
        }
    }
    
    /// Get cache statistics
    pub async fn get_statistics(&self) -> serde_json::Value {
        let total = self.total_requests.load(Ordering::Relaxed);
        let l1_hits = self.l1_hits.load(Ordering::Relaxed);
        let l2_hits = self.l2_hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let promotions = self.promotions.load(Ordering::Relaxed);
        
        let hit_rate = if total > 0 {
            ((l1_hits + l2_hits) as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        
        let l1_hit_rate = if total > 0 {
            (l1_hits as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        
        serde_json::json!({
            "total_requests": total,
            "l1_hits": l1_hits,
            "l2_hits": l2_hits,
            "misses": misses,
            "overall_hit_rate_percent": hit_rate,
            "l1_hit_rate_percent": l1_hit_rate,
            "promotions": promotions,
            "last_updated": chrono::Utc::now().to_rfc3339()
        })
    }
}
