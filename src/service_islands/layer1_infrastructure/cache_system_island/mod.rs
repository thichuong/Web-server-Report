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
use cache_manager::CacheManager;
pub use cache_manager::CacheStrategy;

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
    
    /// Direct access to cache manager
    pub fn cache_manager(&self) -> &Arc<CacheManager> {
        &self.cache_manager
    }

    // ===== COMPATIBILITY METHODS FOR OLD API (DEPRECATED) =====
    
    /// Get latest market data (compatibility method - DEPRECATED)
    /// 
    /// **DEPRECATED**: Use `cache_manager().get("latest_market_data")` instead.
    /// 
    /// Example:
    /// ```
    /// // Old way (deprecated):
    /// let data = cache_system.get_latest_market_data().await?;
    /// 
    /// // New way (recommended):
    /// let data = cache_system.cache_manager().get("latest_market_data").await?;
    /// ```
    #[deprecated(note = "Use cache_manager().get(\"latest_market_data\") instead")]
    #[allow(dead_code)]
    pub async fn get_latest_market_data(&self) -> Result<Option<serde_json::Value>, anyhow::Error> {
        self.cache_manager.get("latest_market_data").await
    }
    
    /// Store market data (compatibility method - DEPRECATED)
    /// 
    /// **DEPRECATED**: Use `cache_manager().set_with_strategy("latest_market_data", data, CacheStrategy::RealTime)` instead.
    /// 
    /// Example:
    /// ```
    /// // Old way (deprecated):
    /// cache_system.store_market_data(data).await?;
    /// 
    /// // New way (recommended):
    /// cache_system.cache_manager().set_with_strategy("latest_market_data", data, CacheStrategy::RealTime).await?;
    /// ```
    #[deprecated(note = "Use cache_manager().set_with_strategy() with appropriate CacheStrategy instead")]
    #[allow(dead_code)]
    pub async fn store_market_data(&self, data: serde_json::Value) -> Result<(), anyhow::Error> {
        self.cache_manager.set_with_strategy("latest_market_data", data, CacheStrategy::Default).await
    }

    /// **DEPRECATED**: Use `cache_manager().get(key)` instead.
    #[deprecated(note = "Use cache_manager().get(key) instead")]
    #[allow(dead_code)]
    pub async fn get(&self, key: &str) -> Result<Option<serde_json::Value>, anyhow::Error> {
        self.cache_manager.get(key).await
    }

    /// Store a single key-value pair with optional TTL (compatibility method - DEPRECATED)
    /// 
    /// **DEPRECATED**: Use `cache_manager().set_with_strategy()` instead for better semantics.
    
    /// Generic set method with TTL (compatibility - DEPRECATED)
    /// 
    /// **DEPRECATED**: Use `cache_manager().set_with_strategy()` instead for better semantics.
    /// 
    /// Example:
    /// ```
    /// // Old way (deprecated):
    /// cache_system.set("key", value, Some(Duration::from_secs(30))).await?;
    /// 
    /// // New way (recommended):
    /// cache_system.cache_manager().set_with_strategy("key", value, CacheStrategy::RealTime).await?;
    /// ```
    #[deprecated(note = "Use cache_manager().set_with_strategy() instead")]
    #[allow(dead_code)]
    pub async fn set(&self, key: &str, value: serde_json::Value, ttl: Option<Duration>) -> Result<(), anyhow::Error> {
        match ttl {
            Some(duration) => self.cache_manager.set_with_strategy(key, value, CacheStrategy::Custom(duration)).await,
            None => self.cache_manager.set_with_strategy(key, value, CacheStrategy::Default).await,
        }
    }
}
