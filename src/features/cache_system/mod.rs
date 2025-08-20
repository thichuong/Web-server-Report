// src/features/cache_system/mod.rs - Cache System Feature
//
// Multi-tier caching system with L1 (in-memory) and L2 (Redis) layers
// Provides unified interface for all caching operations

pub mod cache_manager;
pub mod multi_tier_cache;
pub mod cache_keys;
pub mod cache_stats;

pub use cache_manager::CacheManager;
pub use multi_tier_cache::MultiTierCache;
pub use cache_keys::CacheKeys;
pub use cache_stats::{CacheStats, CacheHealthCheck};

use crate::features::Feature;
use async_trait::async_trait;
use std::collections::HashMap;

/// Cache System Feature - Infrastructure Layer 1
pub struct CacheSystemFeature {
    cache_manager: Option<CacheManager>,
}

impl CacheSystemFeature {
    pub fn new() -> Self {
        Self {
            cache_manager: None,
        }
    }

    pub fn cache_manager(&self) -> Option<&CacheManager> {
        self.cache_manager.as_ref()
    }
}

#[async_trait]
impl Feature for CacheSystemFeature {
    fn name(&self) -> &'static str {
        "cache_system"
    }

    fn version(&self) -> &'static str {
        "1.0.0"
    }

    fn dependencies(&self) -> Vec<&'static str> {
        vec![] // No dependencies - infrastructure layer
    }

    async fn initialize(&mut self, _context: &HashMap<String, String>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let redis_url = _context
            .get("redis_url")
            .ok_or("redis_url not provided in context")?;

        let cache_manager = CacheManager::new(redis_url).await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        self.cache_manager = Some(cache_manager);
        println!("✅ Cache System Feature initialized");
        
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(_cache_manager) = self.cache_manager.take() {
            println!("🔽 Cache System Feature shutting down");
            // Cache manager cleanup happens on drop
        }
        Ok(())
    }

    fn is_healthy(&self) -> bool {
        self.cache_manager.is_some()
    }
}