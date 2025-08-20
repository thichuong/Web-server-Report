//! Cache System Island
//! 
//! This island provides a sophisticated multi-tier caching system:
//! - L1 Cache: Moka in-memory cache (2000 entries, 5min TTL) for hot data
//! - L2 Cache: Redis distributed cache (1hr TTL) for shared data
//! - Intelligent promotion: L2 → L1 automatic for frequently accessed data
//! - Fallback logic: Graceful degradation when cache unavailable

use std::sync::Arc;
use std::time::Duration;
use anyhow::Result;
use serde_json;

pub mod l1_cache;
pub mod l2_cache;
pub mod cache_manager;

use l1_cache::L1Cache;
use l2_cache::L2Cache;
use cache_manager::CacheManager;

/// Cache System Island
/// 
/// Provides a unified caching interface with multi-tier architecture:
/// L1 (Hot data) → L2 (Warm data) → Database/API (Cold data)
pub struct CacheSystemIsland {
    /// L1 Cache - Moka in-memory cache for hot data
    pub l1_cache: Arc<L1Cache>,
    /// L2 Cache - Redis distributed cache for warm data
    pub l2_cache: Arc<L2Cache>,
    /// Cache Manager - Unified cache operations
    pub cache_manager: Arc<CacheManager>,
}

impl CacheSystemIsland {
    /// Initialize the Cache System Island
    /// 
    /// Sets up the multi-tier caching system with intelligent promotion
    /// and fallback mechanisms for high availability.
    pub async fn new() -> Result<Self> {
        println!("💾 Initializing Cache System Island...");
        
        // Initialize L1 Cache (Moka - In-Memory)
        let l1_cache = Arc::new(L1Cache::new().await?);
        println!("  ✅ L1 Cache (Moka) initialized - 2000 entries, 5min TTL");
        
        // Initialize L2 Cache (Redis - Distributed)
        let l2_cache: Arc<L2Cache> = Arc::new(L2Cache::new().await?);
        println!("  ✅ L2 Cache (Redis) initialized - 1hr TTL, fallback ready");
        
        // Initialize Cache Manager
        let cache_manager = Arc::new(CacheManager::new(
            l1_cache.clone(),
            l2_cache.clone()
        ).await?);
        println!("  ✅ Cache Manager initialized - unified operations ready");
        
        println!("💾 Cache System Island initialization complete!");
        
        Ok(Self {
            l1_cache,
            l2_cache,
            cache_manager,
        })
    }
    
    /// Perform health check on the Cache System Island
    /// 
    /// Tests both L1 and L2 cache connectivity and basic operations
    pub async fn health_check(&self) -> bool {
        println!("🔍 Checking Cache System Island health...");
        
        let l1_healthy = self.l1_cache.health_check().await;
        let l2_healthy = self.l2_cache.health_check().await;
        let manager_healthy = self.cache_manager.health_check().await;
        
        let all_healthy = l1_healthy && l2_healthy && manager_healthy;
        
        if all_healthy {
            println!("✅ Cache System Island is healthy!");
        } else {
            println!("❌ Cache System Island health issues detected:");
            if !l1_healthy { println!("  ❌ L1 Cache (Moka) unhealthy"); }
            if !l2_healthy { println!("  ❌ L2 Cache (Redis) unhealthy"); }
            if !manager_healthy { println!("  ❌ Cache Manager unhealthy"); }
        }
        
        all_healthy
    }
    
    /// Get comprehensive cache statistics
    pub async fn get_statistics(&self) -> Result<serde_json::Value> {
        let l1_stats = self.l1_cache.get_statistics().await?;
        let l2_stats = self.l2_cache.get_stats();
        let manager_stats = self.cache_manager.get_statistics().await?;
        
        Ok(serde_json::json!({
            "island": "cache_system",
            "status": "operational",
            "architecture": "multi_tier",
            "tiers": {
                "l1_cache": l1_stats,
                "l2_cache": l2_stats,
                "manager": manager_stats
            },
            "performance": {
                "total_hit_rate": manager_stats.get("overall_hit_rate"),
                "promotion_rate": manager_stats.get("promotion_rate"),
                "fallback_usage": manager_stats.get("fallback_usage")
            }
        }))
    }
    
    /// Get cached value with intelligent promotion
    pub async fn get(&self, key: &str) -> Result<Option<serde_json::Value>> {
        self.cache_manager.get(key).await
    }
    
    /// Set cached value across appropriate tiers
    pub async fn set(&self, key: &str, value: serde_json::Value, ttl: Option<Duration>) -> Result<()> {
        self.cache_manager.set(key, value, ttl).await
    }
    
    /// Delete cached value from all tiers
    pub async fn delete(&self, key: &str) -> Result<()> {
        self.cache_manager.delete(key).await
    }
    
    /// Clear all caches (L1 + L2)
    pub async fn clear_all(&self) -> Result<()> {
        self.cache_manager.clear_all().await
    }
    
    /// Force promotion of key from L2 to L1
    pub async fn promote_to_l1(&self, key: &str) -> Result<bool> {
        self.cache_manager.promote_to_l1(key).await
    }
    
    /// Get cache hit rate statistics
    pub async fn get_hit_rates(&self) -> Result<serde_json::Value> {
        self.cache_manager.get_hit_rates().await
    }
    
    /// Warm up cache with frequently used data
    pub async fn warm_up(&self, keys: Vec<&str>) -> Result<()> {
        println!("🔥 Warming up cache with {} keys...", keys.len());
        for key in keys {
            // In a real implementation, this would load data from database
            let placeholder_data = serde_json::json!({
                "key": key,
                "warmed_up": true,
                "timestamp": chrono::Utc::now().to_rfc3339()
            });
            self.set(key, placeholder_data, Some(Duration::from_secs(300))).await?;
        }
        println!("✅ Cache warm-up complete!");
        Ok(())
    }
}
