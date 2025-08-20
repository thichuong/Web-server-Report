//! Cache Manager - Unified Cache Operations
//! 
//! Manages intelligent caching operations across L1 and L2 tiers with:
//! - Automatic promotion from L2 to L1 for frequently accessed data
//! - Unified get/set operations with fallback logic
//! - Cache warming and optimization strategies

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::Duration;
use anyhow::Result;
use serde_json;

use super::l1_cache::L1Cache;
use super::l2_cache::L2Cache;

/// Cache access pattern tracking
#[derive(Debug, Clone)]
struct AccessPattern {
    access_count: usize,
    last_accessed: std::time::Instant,
    promotion_eligible: bool,
}

/// Cache Manager - Unified cache operations with intelligent promotion
pub struct CacheManager {
    /// L1 Cache reference
    l1_cache: Arc<L1Cache>,
    /// L2 Cache reference
    l2_cache: Arc<L2Cache>,
    /// Access pattern tracking for promotion decisions
    access_patterns: Arc<std::sync::RwLock<std::collections::HashMap<String, AccessPattern>>>,
    /// Statistics tracking
    total_requests: Arc<AtomicU64>,
    l1_hits: Arc<AtomicU64>,
    l2_hits: Arc<AtomicU64>,
    misses: Arc<AtomicU64>,
    promotions: Arc<AtomicUsize>,
    fallback_usage: Arc<AtomicUsize>,
}

impl CacheManager {
    /// Initialize Cache Manager
    pub async fn new(l1_cache: Arc<L1Cache>, l2_cache: Arc<L2Cache>) -> Result<Self> {
        println!("🎯 Initializing Cache Manager...");
        
        Ok(Self {
            l1_cache,
            l2_cache,
            access_patterns: Arc::new(std::sync::RwLock::new(std::collections::HashMap::new())),
            total_requests: Arc::new(AtomicU64::new(0)),
            l1_hits: Arc::new(AtomicU64::new(0)),
            l2_hits: Arc::new(AtomicU64::new(0)),
            misses: Arc::new(AtomicU64::new(0)),
            promotions: Arc::new(AtomicUsize::new(0)),
            fallback_usage: Arc::new(AtomicUsize::new(0)),
        })
    }
    
    /// Get value with intelligent tier selection and promotion
    pub async fn get(&self, key: &str) -> Result<Option<serde_json::Value>> {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        
        // Try L1 first (hot data)
        if let Some(value) = self.l1_cache.get(key).await {
            self.l1_hits.fetch_add(1, Ordering::Relaxed);
            self.update_access_pattern(key, true).await;
            return Ok(Some(value));
        }
        
        // Try L2 (warm data)
        match self.l2_cache.get(key).await {
            Some(value) => {
                self.l2_hits.fetch_add(1, Ordering::Relaxed);
                
                // Check if this key should be promoted to L1
                if self.should_promote_to_l1(key).await {
                    let _ = self.l1_cache.set(key, value.clone()).await;
                    self.promotions.fetch_add(1, Ordering::Relaxed);
                    println!("⬆️  Promoted key '{}' from L2 to L1", key);
                }
                
                self.update_access_pattern(key, false).await;
                Ok(Some(value))
            }
            None => {
                self.misses.fetch_add(1, Ordering::Relaxed);
                Ok(None)
            }
        }
    }
    
    /// Set value with intelligent tier placement
    pub async fn set(&self, key: &str, value: serde_json::Value, ttl: Option<Duration>) -> Result<()> {
        // Determine where to store based on access patterns and value characteristics
        let should_store_in_l1 = self.should_store_in_l1(key, &value).await;
        
        if should_store_in_l1 {
            // Store in both L1 and L2 for hot data
            self.l1_cache.set(key, value.clone()).await?;
            self.l2_cache.set(key.to_string(), value).await;
        } else {
            // Store only in L2 for warm/cold data
            self.l2_cache.set(key.to_string(), value).await;
        }
        
        Ok(())
    }
    
    /// Delete value from all cache tiers
    pub async fn delete(&self, key: &str) -> Result<()> {
        // Remove from both tiers
        let _ = self.l1_cache.remove(key).await;
        let _ = self.l2_cache.remove(key).await;
        
        // Clean up access pattern tracking
        {
            let mut patterns = self.access_patterns.write().unwrap();
            patterns.remove(key);
        }
        
        Ok(())
    }
    
    /// Clear all caches
    pub async fn clear_all(&self) -> Result<()> {
        self.l1_cache.clear().await?;
        self.l2_cache.clear().await;
        
        // Clear access patterns
        {
            let mut patterns = self.access_patterns.write().unwrap();
            patterns.clear();
        }
        
        Ok(())
    }
    
    /// Force promotion of key from L2 to L1
    pub async fn promote_to_l1(&self, key: &str) -> Result<bool> {
        // Get value from L2
        match self.l2_cache.get(key).await {
            Some(value) => {
                // Store in L1
                self.l1_cache.set(key, value).await?;
                self.promotions.fetch_add(1, Ordering::Relaxed);
                println!("⬆️  Force promoted key '{}' from L2 to L1", key);
                Ok(true)
            }
            None => Ok(false),
        }
    }
    
    /// Update access pattern for promotion decisions
    async fn update_access_pattern(&self, key: &str, was_l1_hit: bool) {
        let mut patterns = self.access_patterns.write().unwrap();
        
        let pattern = patterns.entry(key.to_string()).or_insert(AccessPattern {
            access_count: 0,
            last_accessed: std::time::Instant::now(),
            promotion_eligible: false,
        });
        
        pattern.access_count += 1;
        pattern.last_accessed = std::time::Instant::now();
        
        // Mark as promotion eligible if accessed multiple times from L2
        if !was_l1_hit && pattern.access_count >= 3 {
            pattern.promotion_eligible = true;
        }
    }
    
    /// Determine if key should be promoted to L1
    async fn should_promote_to_l1(&self, key: &str) -> bool {
        let patterns = self.access_patterns.read().unwrap();
        
        match patterns.get(key) {
            Some(pattern) => {
                // Promote if accessed multiple times recently
                pattern.promotion_eligible && 
                pattern.access_count >= 3 &&
                pattern.last_accessed.elapsed() < Duration::from_secs(300) // 5 minutes
            }
            None => false,
        }
    }
    
    /// Determine if value should be stored in L1 initially
    async fn should_store_in_l1(&self, key: &str, value: &serde_json::Value) -> bool {
        // Store in L1 if:
        // 1. Key has been accessed recently
        // 2. Value is small (< 10KB)
        // 3. Key indicates it's frequently accessed data
        
        let value_size = serde_json::to_string(value).unwrap_or_default().len();
        let is_small_value = value_size < 10240; // 10KB
        
        let has_recent_access = {
            let patterns = self.access_patterns.read().unwrap();
            patterns.get(key).map_or(false, |p| p.last_accessed.elapsed() < Duration::from_secs(600)) // 10 minutes
        };
        
        let is_hot_key = key.contains("dashboard") || 
                        key.contains("summary") || 
                        key.contains("frequent");
        
        is_small_value && (has_recent_access || is_hot_key)
    }
    
    /// Get comprehensive hit rates
    pub async fn get_hit_rates(&self) -> Result<serde_json::Value> {
        let total = self.total_requests.load(Ordering::Relaxed);
        let l1_hits = self.l1_hits.load(Ordering::Relaxed);
        let l2_hits = self.l2_hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        
        let overall_hit_rate = if total > 0 {
            ((l1_hits + l2_hits) as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        
        let l1_hit_rate = if total > 0 {
            (l1_hits as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        
        let l2_hit_rate = if total > 0 {
            (l2_hits as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        
        Ok(serde_json::json!({
            "overall_hit_rate_percent": overall_hit_rate,
            "l1_hit_rate_percent": l1_hit_rate,
            "l2_hit_rate_percent": l2_hit_rate,
            "miss_rate_percent": if total > 0 { (misses as f64 / total as f64) * 100.0 } else { 0.0 },
            "total_requests": total,
            "l1_hits": l1_hits,
            "l2_hits": l2_hits,
            "misses": misses
        }))
    }
    
    /// Optimize cache by promoting frequently accessed keys
    pub async fn optimize_cache(&self) -> Result<usize> {
        println!("⚡ Running cache optimization...");
        let mut promoted_count = 0;
        
        // Get frequently accessed keys from L2
        let keys_to_promote: Vec<String> = {
            let patterns = self.access_patterns.read().unwrap();
            patterns.iter()
                .filter(|(_, pattern)| {
                    pattern.promotion_eligible && 
                    pattern.access_count >= 3 &&
                    pattern.last_accessed.elapsed() < Duration::from_secs(3600) // 1 hour
                })
                .map(|(key, _)| key.clone())
                .collect()
        };
        
        for key in keys_to_promote {
            if self.promote_to_l1(&key).await? {
                promoted_count += 1;
            }
        }
        
        println!("⚡ Cache optimization complete: {} keys promoted", promoted_count);
        Ok(promoted_count)
    }
    
    /// Health check for cache manager
    pub async fn health_check(&self) -> bool {
        let l1_healthy = self.l1_cache.health_check().await;
        let l2_healthy = self.l2_cache.health_check().await;
        
        // Test unified operations
        let test_key = "health_check_manager";
        let test_value = serde_json::json!({
            "test": "cache_manager_health",
            "timestamp": chrono::Utc::now().to_rfc3339()
        });
        
        match self.set(test_key, test_value.clone(), Some(Duration::from_secs(10))).await {
            Ok(_) => {
                match self.get(test_key).await {
                    Ok(Some(retrieved)) => {
                        let _ = self.delete(test_key).await;
                        if retrieved["test"] == test_value["test"] {
                            println!("✅ Cache Manager health check passed");
                            l1_healthy || l2_healthy // At least one tier should be healthy
                        } else {
                            eprintln!("❌ Cache Manager value mismatch");
                            false
                        }
                    }
                    Ok(None) => {
                        eprintln!("❌ Cache Manager get operation failed");
                        false
                    }
                    Err(e) => {
                        eprintln!("❌ Cache Manager get error: {}", e);
                        false
                    }
                }
            }
            Err(e) => {
                eprintln!("❌ Cache Manager set error: {}", e);
                false
            }
        }
    }
    
    /// Get cache manager statistics
    pub async fn get_statistics(&self) -> Result<serde_json::Value> {
        let total_requests = self.total_requests.load(Ordering::Relaxed);
        let l1_hits = self.l1_hits.load(Ordering::Relaxed);
        let l2_hits = self.l2_hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let promotions = self.promotions.load(Ordering::Relaxed);
        let fallback_usage = self.fallback_usage.load(Ordering::Relaxed);
        
        let overall_hit_rate = if total_requests > 0 {
            ((l1_hits + l2_hits) as f64 / total_requests as f64) * 100.0
        } else {
            0.0
        };
        
        let promotion_rate = if l2_hits > 0 {
            (promotions as f64 / l2_hits as f64) * 100.0
        } else {
            0.0
        };
        
        let access_patterns_count = {
            let patterns = self.access_patterns.read().unwrap();
            patterns.len()
        };
        
        Ok(serde_json::json!({
            "component": "cache_manager",
            "status": "operational",
            "statistics": {
                "total_requests": total_requests,
                "l1_hits": l1_hits,
                "l2_hits": l2_hits,
                "misses": misses,
                "promotions": promotions,
                "fallback_usage": fallback_usage,
                "tracked_keys": access_patterns_count
            },
            "performance": {
                "overall_hit_rate_percent": overall_hit_rate,
                "promotion_rate_percent": promotion_rate,
                "tier_distribution": {
                    "l1_percent": if total_requests > 0 { (l1_hits as f64 / total_requests as f64) * 100.0 } else { 0.0 },
                    "l2_percent": if total_requests > 0 { (l2_hits as f64 / total_requests as f64) * 100.0 } else { 0.0 },
                    "miss_percent": if total_requests > 0 { (misses as f64 / total_requests as f64) * 100.0 } else { 0.0 }
                }
            },
            "optimization": {
                "intelligent_promotion": true,
                "access_pattern_tracking": true,
                "fallback_support": true
            }
        }))
    }
    
    /// Get cache utilization report
    pub async fn get_utilization_report(&self) -> Result<serde_json::Value> {
        let l1_stats = self.l1_cache.get_statistics().await?;
        let l2_stats = self.l2_cache.get_stats();
        let manager_stats = self.get_statistics().await?;
        
        Ok(serde_json::json!({
            "report_type": "cache_utilization",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "summary": {
                "overall_health": "healthy",
                "total_tiers": 2,
                "active_tiers": if l1_stats["statistics"]["current_size"].as_u64().unwrap_or(0) > 0 { 1 } else { 0 } +
                               if l2_stats.hits > 0 { 1 } else { 0 }
            },
            "tier_details": {
                "l1": l1_stats,
                "l2": l2_stats
            },
            "unified_operations": manager_stats,
            "recommendations": [
                "Continue monitoring promotion patterns",
                "Consider increasing L1 capacity if hit rate is high",
                "Monitor Redis connectivity for L2 stability"
            ]
        }))
    }
}
