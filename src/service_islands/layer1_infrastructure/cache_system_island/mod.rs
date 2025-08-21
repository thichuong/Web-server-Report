//! Cache System Island
//! 
//! This island provides a sophisticated multi-tier caching system with Redis Streams:
//! - Redis Streams: Primary storage for real-time data (event sourcing)
//! - L1 Cache: Moka in-memory cache (2000 entries, 5min TTL) for hot data
//! - L2 Cache: Redis distributed cache (1hr TTL) for shared data
//! - PostgreSQL: Backup storage for historical data
//! - Intelligent promotion: Stream → L1 → L2 for frequently accessed data
//! - Fallback logic: Graceful degradation when components unavailable

use std::sync::Arc;
use std::time::Duration;
use anyhow::Result;
use serde_json;

pub mod l1_cache;
pub mod l2_cache;
pub mod cache_manager;
pub mod redis_stream_manager;
pub mod stream_data_service;
// pub mod stream_db_sync; // Temporarily disabled - focus on streams as primary storage

use l1_cache::L1Cache;
use l2_cache::L2Cache;
use cache_manager::CacheManager;
use redis_stream_manager::RedisStreamManager;
use stream_data_service::StreamDataService;
// use stream_db_sync::StreamDbSyncService; // Temporarily disabled

/// Cache System Island with Redis Streams
/// 
/// Provides a unified storage and caching interface with multi-tier architecture:
/// Redis Streams (Primary) → L1 Cache (Hot) → L2 Cache (Warm) → PostgreSQL (Cold)
pub struct CacheSystemIsland {
    /// Stream Data Service - Primary storage for real-time data
    pub stream_service: Arc<StreamDataService>,
    /// Redis Stream Manager - Low-level stream operations
    pub stream_manager: Arc<RedisStreamManager>,
    /// L1 Cache - Moka in-memory cache for hot data
    pub l1_cache: Arc<L1Cache>,
    /// L2 Cache - Redis distributed cache for warm data
    pub l2_cache: Arc<L2Cache>,
    /// Cache Manager - Traditional cache operations (legacy compatibility)
    pub cache_manager: Arc<CacheManager>,
    // Stream to DB Sync Service - Background backup to PostgreSQL (temporarily disabled)
    // pub sync_service: Option<Arc<tokio::sync::Mutex<StreamDbSyncService>>>,
}

impl CacheSystemIsland {
    /// Initialize the Cache System Island with Redis Streams
    /// 
    /// Sets up the complete storage hierarchy including Redis Streams as primary storage,
    /// multi-tier caching system with intelligent promotion, and fallback mechanisms.
    pub async fn new() -> Result<Self> {
        println!("💾 Initializing Cache System Island with Redis Streams...");
        
        // Initialize Redis Stream components (primary storage)
        let stream_manager = Arc::new(RedisStreamManager::new().await?);
        println!("  ✅ Redis Stream Manager initialized");
        
        let stream_service = Arc::new(StreamDataService::new().await?);
        println!("  ✅ Stream Data Service initialized");
        
        // Initialize traditional cache layers (for compatibility and hot data)
        let l1_cache = Arc::new(L1Cache::new().await?);
        println!("  ✅ L1 Cache (Moka) initialized - 2000 entries, 5min TTL");
        
        let l2_cache: Arc<L2Cache> = Arc::new(L2Cache::new().await?);
        println!("  ✅ L2 Cache (Redis) initialized - 1hr TTL, fallback ready");
        
        // Initialize Cache Manager (legacy compatibility)
        let cache_manager = Arc::new(CacheManager::new(
            l1_cache.clone(),
            l2_cache.clone()
        ).await?);
        println!("  ✅ Cache Manager initialized - unified operations ready");
        
        println!("💾 Cache System Island initialization complete!");
        
        Ok(Self {
            stream_service,
            stream_manager,
            l1_cache,
            l2_cache,
            cache_manager,
            // sync_service: None, // Disabled - focus on streams as primary storage
        })
    }
    
    /// Start background stream processing
    /// 
    /// Starts automated background tasks for stream processing including
    /// database backup, analytics, and stream maintenance.
    pub async fn start_background_processing(&self) -> Result<()> {
        println!("🔄 Starting Cache System Island background processing...");
        self.stream_service.start_background_processing().await?;
        println!("✅ Background processing started");
        Ok(())
    }

    /// Initialize PostgreSQL backup sync service
    /// 
    /// Phase 3: Sets up background sync from Redis Streams to PostgreSQL backup
    /// Currently disabled - focusing on Redis Streams as primary storage
    pub async fn initialize_db_sync(&mut self, _db_pool: sqlx::PgPool) -> Result<()> {
        println!("🔄 PostgreSQL backup sync temporarily disabled");
        println!("   Focus: Redis Streams as primary storage");
        println!("   Future: Can enable backup sync when needed");
        
        // TODO: Enable when backup sync is required
        // let mut sync_service = StreamDbSyncService::new(
        //     self.stream_manager.clone(),
        //     db_pool
        // );
        // sync_service.start_background_sync().await?;
        // self.sync_service = Some(Arc::new(tokio::sync::Mutex::new(sync_service)));
        
        Ok(())
    }
    
    /// Perform health check on the Cache System Island with Streams
    /// 
    /// Tests Redis Streams, traditional cache layers, and all components
    pub async fn health_check(&self) -> bool {
        println!("🔍 Checking Cache System Island health (with Redis Streams)...");
        
        let stream_healthy = self.stream_manager.health_check().await;
        let l1_healthy = self.l1_cache.health_check().await;
        let l2_healthy = self.l2_cache.health_check().await;
        let manager_healthy = self.cache_manager.health_check().await;
        // let sync_healthy = self.sync_service.is_some(); // Disabled
        
        let all_healthy = stream_healthy && l1_healthy && l2_healthy && manager_healthy;
        
        if all_healthy {
            println!("✅ Cache System Island with Redis Streams is healthy!");
            println!("  📊 PostgreSQL backup sync: Disabled (streams-focused architecture)");
        } else {
            println!("❌ Cache System Island health issues detected:");
            if !stream_healthy { println!("  ❌ Redis Streams unhealthy"); }
            if !l1_healthy { println!("  ❌ L1 Cache (Moka) unhealthy"); }
            if !l2_healthy { println!("  ❌ L2 Cache (Redis) unhealthy"); }
            if !manager_healthy { println!("  ❌ Cache Manager unhealthy"); }
        }
        
        all_healthy
    }
    
    /// Get comprehensive system statistics including streams
    pub async fn get_statistics(&self) -> Result<serde_json::Value> {
        let l1_stats = self.l1_cache.get_statistics().await?;
        let l2_stats = self.l2_cache.get_stats();
        let manager_stats = self.cache_manager.get_statistics().await?;
        let stream_stats = self.stream_service.get_stream_health().await?;
        
        Ok(serde_json::json!({
            "island": "cache_system_with_streams",
            "status": "operational",
            "architecture": "stream_primary_cache_secondary",
            "components": {
                "primary_storage": stream_stats,
                "l1_cache": l1_stats,
                "l2_cache": l2_stats,
                "cache_manager": manager_stats
            },
            "data_flow": [
                "External API → Redis Streams (primary)",
                "Redis Streams → L1 Cache (hot data)",
                "L1 Cache → L2 Cache (warm data)", 
                "Background → PostgreSQL (backup)"
            ]
        }))
    }
    
    // === Legacy Cache Interface (for backward compatibility) ===
    
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
    
    // === Stream Interface (new primary storage methods) ===
    
    /// Store market data in Redis Stream (primary method for real-time data)
    pub async fn store_market_data(&self, data: serde_json::Value) -> Result<String> {
        self.stream_service.store_market_data(data).await
    }
    
    /// Store BTC price update in Redis Stream
    pub async fn store_btc_price(&self, price_data: serde_json::Value) -> Result<String> {
        self.stream_service.store_btc_price(price_data).await
    }
    
    /// Store dashboard summary in Redis Stream
    pub async fn store_dashboard_summary(&self, dashboard_data: serde_json::Value) -> Result<String> {
        self.stream_service.store_dashboard_summary(dashboard_data).await
    }
    
    /// Get latest market data from Stream (primary read method)
    pub async fn get_latest_market_data(&self) -> Result<Option<serde_json::Value>> {
        self.stream_service.get_latest_market_data().await
    }
    
    /// Get latest BTC price from Stream
    pub async fn get_latest_btc_price(&self) -> Result<Option<serde_json::Value>> {
        self.stream_service.get_latest_btc_price().await
    }
    
    /// Get dashboard data for WebSocket consumers
    pub async fn get_dashboard_for_websocket(&self) -> Result<Vec<serde_json::Value>> {
        self.stream_service.get_dashboard_for_websocket().await
    }
    
    /// Get access to the cache manager (for external usage)
    pub fn get_cache_manager(&self) -> Arc<CacheManager> {
        self.cache_manager.clone()
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
