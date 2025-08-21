//! Cache System Island (Rebuilt)
//! 
//! Simple two-tier cache system:
//! - L1 Cache: Moka in-memory cache for hot data
//! - L2 Cache: Redis for persistent cache storage
//! 
//! Maintains compatibility with existing API aggregator interface.

use std::sync::Arc;
use std::time::Duration;
use anyhow::Result;

pub mod l1_cache;
pub mod l2_cache;
pub mod cache_manager;

use l1_cache::L1Cache;
use l2_cache::L2Cache;
use cache_manager::{CacheManager, CacheStrategy};

/// Cache System Island - Two-tier caching system
pub struct CacheSystemIsland {
    /// Cache Manager - Unified cache operations
    pub cache_manager: Arc<CacheManager>,
    /// L1 Cache - Moka in-memory cache
    pub l1_cache: Arc<L1Cache>,
    /// L2 Cache - Redis cache
    pub l2_cache: Arc<L2Cache>,
}

impl CacheSystemIsland {
    /// Initialize the Cache System Island
    pub async fn new() -> Result<Self> {
        println!("🏗️ Initializing Cache System Island (Rebuilt)...");
        
        // Initialize L1 cache (Moka)
        let l1_cache = Arc::new(L1Cache::new().await?);
        
        // Initialize L2 cache (Redis)
        let l2_cache = Arc::new(L2Cache::new().await?);
        
        // Initialize cache manager
        let cache_manager = Arc::new(CacheManager::new(l1_cache.clone(), l2_cache.clone()).await?);
        
        println!("✅ Cache System Island initialized successfully");
        
        Ok(Self {
            cache_manager,
            l1_cache,
            l2_cache,
        })
    }
    
    /// Health check for cache system
    pub async fn health_check(&self) -> bool {
        let l1_ok = self.l1_cache.health_check().await;
        let l2_ok = self.l2_cache.health_check().await;
        
        if l1_ok && l2_ok {
            println!("  ✅ Cache System Island health check passed");
            true
        } else {
            println!("  ⚠️ Cache System Island health check failed - L1: {}, L2: {}", l1_ok, l2_ok);
            l1_ok // At minimum, L1 should work
        }
    }
    
    /// Get cache manager (for compatibility with existing code)
    pub fn get_cache_manager(&self) -> Arc<CacheManager> {
        self.cache_manager.clone()
    }
    
    /// Get statistics from cache system
    pub async fn get_statistics(&self) -> serde_json::Value {
        let l1_stats = self.l1_cache.get_statistics().await;
        let l2_stats = self.l2_cache.get_statistics().await;
        let manager_stats = self.cache_manager.get_statistics().await;
        
        serde_json::json!({
            "l1_cache": l1_stats,
            "l2_cache": l2_stats,
            "cache_manager": manager_stats,
            "system_status": "healthy"
        })
    }

    // ===== COMPATIBILITY METHODS FOR OLD API =====
    
    /// Get latest market data (compatibility method)
    pub async fn get_latest_market_data(&self) -> Result<Option<serde_json::Value>, anyhow::Error> {
        self.cache_manager.get("latest_market_data").await
    }
    
    /// Store market data (compatibility method)  
    pub async fn store_market_data(&self, data: serde_json::Value) -> Result<(), anyhow::Error> {
        self.cache_manager.set("latest_market_data", data).await
    }

    /// Generic get method (compatibility)
    pub async fn get(&self, key: &str) -> Result<Option<serde_json::Value>, anyhow::Error> {
        self.cache_manager.get(key).await
    }
    
    /// Generic set method with TTL (compatibility)
    pub async fn set(&self, key: &str, value: serde_json::Value, ttl: Option<Duration>) -> Result<(), anyhow::Error> {
        match ttl {
            Some(duration) => self.cache_manager.set_with_strategy(key, value, CacheStrategy::Custom(duration)).await,
            None => self.cache_manager.set(key, value).await,
        }
    }
}
