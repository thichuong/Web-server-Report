# 💾 CACHE ARCHITECTURE: MULTI-TIER SYSTEM SPECIFICATION

## 📊 Overview
This document specifies a sophisticated multi-tier caching system with L1 (in-memory) and L2 (Redis) layers, providing unified caching interface across the entire application with intelligent fallback mechanisms.

## 🏗️ Architecture Components

### 1. Multi-Tier Cache Structure
```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Application   │───▶│  L1 Cache       │───▶│  L2 Cache       │
│   Layer         │    │  (In-Memory)    │    │  (Redis)        │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                              │                        │
                              ▼                        ▼
                       ┌─────────────┐         ┌─────────────┐
                       │ Moka Cache  │         │ Redis Pool  │
                       │ 2000 items  │         │ 10 conns    │
                       │ 300s TTL    │         │ 3600s TTL   │
                       └─────────────┘         └─────────────┘
```

### 2. Cache Manager Interface
**Purpose**: Single unified interface for all caching operations across the application

#### Core Components:
```rust
pub struct CacheManager {
    cache: Arc<MultiTierCache>,
}

pub struct MultiTierCache {
    l1_cache: Arc<MokaCache<String, String>>,  // In-memory cache (L1)
    redis_pool: Arc<RedisPool>,                // Redis connection pool (L2)
}
```

## 🔧 Configuration Constants

### Cache Capacity & TTL Settings
```rust
// L1 Cache (In-Memory) Configuration
const L1_MAX_CAPACITY: u64 = 2000;    // Maximum 2000 cached items
const L1_TTL_SECONDS: u64 = 300;      // 5 minutes TTL
const L1_IDLE_TIMEOUT: u64 = 150;     // 2.5 minutes idle timeout (TTL/2)

// L2 Cache (Redis) Configuration  
const L2_TTL_SECONDS: u64 = 3600;     // 1 hour TTL

// Data-Type Specific TTL
const DASHBOARD_TTL_SECONDS: u64 = 300;     // 5 minutes for dashboard data
const MARKET_DATA_TTL_SECONDS: u64 = 60;    // 1 minute for market data  
const REPORT_TTL_SECONDS: u64 = 1800;       // 30 minutes for reports
const USER_DATA_TTL_SECONDS: u64 = 900;     // 15 minutes for user data

// Redis Connection Pool
const REDIS_MAX_CONNECTIONS: u32 = 10;      // Maximum 10 connections
const REDIS_MIN_IDLE: u32 = 2;              // Minimum 2 idle connections
```

## 🎯 Cache Access Patterns

### 1. Cache-or-Compute Pattern (Primary)
**Purpose**: Generic pattern for cache-first data access with automatic fallback

```rust
pub async fn cache_or_compute<T, F, Fut>(
    &self, 
    key: &str, 
    ttl_seconds: u64, 
    compute_fn: F
) -> Result<T>
where
    T: Serialize + for<'de> Deserialize<'de> + Clone,
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<T>>,
{
    // Step 1: Try cache first (L1 → L2)
    if let Ok(Some(cached_data)) = self.cache.get::<T>(key).await {
        return Ok(cached_data);
    }

    // Step 2: Cache miss - compute fresh data
    println!("💻 Computing fresh data for key: {}", key);
    let fresh_data = compute_fn().await?;

    // Step 3: Cache result with custom TTL
    if let Err(e) = self.cache.set_with_ttl(key, &fresh_data, ttl_seconds).await {
        eprintln!("⚠️ Failed to cache computed data: {}", e);
    }

    Ok(fresh_data)
}
```

### 2. Specialized Cache Methods
**Purpose**: Domain-specific caching with optimal TTL values

#### Dashboard Data Caching
```rust
pub async fn cache_dashboard_data<T, F, Fut>(&self, compute_fn: F) -> Result<T>
where T: Serialize + for<'de> Deserialize<'de> + Clone + Send + 'static
{
    let key = CacheKeys::dashboard_summary();
    self.cache_or_compute(&key, DASHBOARD_TTL_SECONDS, compute_fn).await
}
```

#### Market Data Caching (Short TTL)
```rust
pub async fn cache_market_data<T, F, Fut>(&self, symbol: &str, compute_fn: F) -> Result<T>
{
    let key = CacheKeys::market_data(symbol);
    self.cache_or_compute(&key, MARKET_DATA_TTL_SECONDS, compute_fn).await
}
```

#### Report Data Caching (Medium TTL)
```rust
pub async fn cache_report_data<T, F, Fut>(
    &self, 
    symbol: &str, 
    timeframe: &str, 
    compute_fn: F
) -> Result<T>
{
    let key = CacheKeys::crypto_report(symbol, timeframe);
    self.cache_or_compute(&key, REPORT_TTL_SECONDS, compute_fn).await
}
```

## 🔄 Multi-Tier Cache Operations

### Cache Retrieval Flow
```rust
pub async fn get<T>(&self, key: &str) -> Result<Option<T>>
where T: for<'de> Deserialize<'de>
{
    // Step 1: Check L1 cache first (fastest - ~0.1ms)
    if let Some(cached_data) = self.l1_cache.get(key).await {
        println!("🎯 L1 Cache HIT for key: {}", key);
        let data: T = serde_json::from_str(&cached_data)?;
        return Ok(Some(data));
    }

    // Step 2: L1 miss - check L2 cache (fast - ~5ms)
    if let Ok(Some(redis_data)) = self.get_from_redis(key).await {
        println!("🔥 L2 Cache HIT for key: {}", key);
        
        // Promote to L1 cache for future requests
        self.l1_cache.insert(key.to_string(), redis_data.clone()).await;
        
        let data: T = serde_json::from_str(&redis_data)?;
        return Ok(Some(data));
    }

    // Step 3: Complete cache miss
    println!("💔 Cache MISS for key: {}", key);
    Ok(None)
}
```

### Cache Storage Flow
```rust
pub async fn set_with_ttl<T>(&self, key: &str, data: &T, ttl_seconds: u64) -> Result<()>
where T: Serialize
{
    // Serialize data once
    let serialized = serde_json::to_string(data)?;

    // Store in L1 cache (immediate)
    self.l1_cache.insert(key.to_string(), serialized.clone()).await;

    // Store in L2 cache (Redis) with custom TTL
    if let Err(e) = self.set_in_redis_with_ttl(key, &serialized, ttl_seconds).await {
        eprintln!("⚠️ Failed to cache in Redis (L2): {}", e);
        // Don't fail operation if Redis unavailable
    }

    println!("💾 Cached data for key: {} (TTL: {}s)", key, ttl_seconds);
    Ok(())
}
```

## 🔑 Cache Key Strategy

### Consistent Key Naming System
```rust
pub struct CacheKeys;

impl CacheKeys {
    // Dashboard keys
    pub fn dashboard_summary() -> String {
        "dashboard:summary".to_string()
    }
    
    pub fn dashboard_summary_non_btc() -> String {
        "dashboard:summary:non_btc".to_string()
    }
    
    // Crypto report keys
    pub fn crypto_report(symbol: &str, timeframe: &str) -> String {
        format!("crypto:report:{}:{}", symbol.to_lowercase(), timeframe)
    }
    
    // Market data keys
    pub fn market_data(symbol: &str) -> String {
        format!("market:{}", symbol.to_lowercase())
    }
    
    // Price data keys
    pub fn price_data(symbol: &str, interval: &str) -> String {
        format!("price:{}:{}", symbol.to_lowercase(), interval)
    }
    
    // Technical indicator keys
    pub fn technical_indicator(symbol: &str, indicator: &str, period: &str) -> String {
        format!("tech:{}:{}:{}", symbol.to_lowercase(), indicator, period)
    }
    
    // API data keys
    pub fn api_data(provider: &str, endpoint: &str) -> String {
        format!("api:{}:{}", provider.to_lowercase(), endpoint)
    }
    
    // User-specific keys
    pub fn user_report(user_id: u32, report_id: u32) -> String {
        format!("user:{}:report:{}", user_id, report_id)
    }
}
```

### Key Namespace Strategy
- `dashboard:*` - Dashboard aggregated data
- `market:*` - Real-time market data  
- `crypto:*` - Cryptocurrency reports and analysis
- `tech:*` - Technical indicators and analysis
- `price:*` - Price data and history
- `api:*` - External API responses
- `user:*` - User-specific cached data

## 🚨 Error Handling & Resilience

### Graceful Degradation Strategy
```rust
// L1 Cache Always Available - In-memory cannot fail
// L2 Cache Failures - Graceful fallback

async fn set_in_redis_with_ttl(&self, key: &str, value: &str, ttl_seconds: u64) -> Result<()> {
    let mut conn = self.redis_pool.get().await
        .context("Failed to get Redis connection from pool")?;
        
    let _: () = conn.set_ex(key, value, ttl_seconds).await
        .context("Failed to set data in Redis with TTL")?;
        
    Ok(())
}

// Usage with error handling
if let Err(e) = self.set_in_redis_with_ttl(key, &serialized, ttl_seconds).await {
    eprintln!("⚠️ Failed to cache in Redis (L2): {}", e);
    // Continue operation - L1 cache still works
}
```

### Circuit Breaker Integration
```rust
pub async fn health_check(&self) -> CacheHealthCheck {
    let l1_healthy = true; // L1 is always available
    let l2_healthy = self.check_redis_health().await;

    CacheHealthCheck {
        l1_healthy,
        l2_healthy,
        overall_healthy: l1_healthy, // System works as long as L1 is available
    }
}

async fn check_redis_health(&self) -> bool {
    match self.redis_pool.get().await {
        Ok(mut conn) => {
            match conn.get::<_, Option<String>>("__health_check__").await {
                Ok(_) => true,
                Err(_) => false,
            }
        }
        Err(_) => false,
    }
}
```

## 📊 Monitoring & Statistics

### Cache Statistics
```rust
#[derive(Debug, Clone, Serialize)]
pub struct CacheStats {
    pub l1_entry_count: u64,
    pub l1_hit_count: u64,
    pub l1_miss_count: u64,
    pub l1_hit_rate: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct CacheHealthCheck {
    pub l1_healthy: bool,
    pub l2_healthy: bool,
    pub overall_healthy: bool,
}

pub async fn stats(&self) -> CacheStats {
    let l1_entry_count = self.l1_cache.entry_count();
    // Note: Custom hit/miss tracking would be implemented here
    CacheStats {
        l1_entry_count,
        l1_hit_count: 0,    // Requires custom implementation
        l1_miss_count: 0,   // Requires custom implementation  
        l1_hit_rate: 0.0,   // Calculate from custom metrics
    }
}
```

### Administrative Operations
```rust
// Pattern-based cache clearing (Redis only)
pub async fn clear_pattern(&self, pattern: &str) -> Result<u32> {
    self.clear_redis_pattern(pattern).await
}

// Individual key invalidation
pub async fn invalidate(&self, key: &str) -> Result<()> {
    // Remove from L1 cache
    self.l1_cache.invalidate(key).await;

    // Remove from L2 cache (Redis)
    if let Err(e) = self.delete_from_redis(key).await {
        eprintln!("⚠️ Failed to invalidate Redis cache (L2): {}", e);
    }

    println!("🗑️ Invalidated cache for key: {}", key);
    Ok(())
}

// Clear all L1 cache (emergency)
pub async fn clear_all(&self) -> Result<()> {
    self.l1_cache.invalidate_all();
    println!("🧹 Cleared L1 cache (in-memory)");
    Ok(())
}
```

## 🔌 Integration Requirements

### Dependencies
```rust
// Core dependencies
use moka::future::Cache as MokaCache;
use bb8_redis::{bb8::Pool, RedisConnectionManager, redis::AsyncCommands};
use serde::{Deserialize, Serialize};
use anyhow::{Context, Result};
use std::{sync::Arc, time::Duration};

// Configuration
pub type RedisPool = Pool<RedisConnectionManager>;
```

### Initialization Pattern
```rust
impl CacheManager {
    pub async fn new(redis_url: &str) -> Result<Self> {
        // Initialize L1 cache (Moka)
        let l1_cache = Arc::new(
            MokaCache::builder()
                .max_capacity(L1_MAX_CAPACITY)
                .time_to_live(Duration::from_secs(L1_TTL_SECONDS))
                .time_to_idle(Duration::from_secs(L1_TTL_SECONDS / 2))
                .build()
        );

        // Initialize L2 cache (Redis Pool)
        let redis_manager = RedisConnectionManager::new(redis_url)?;
        let redis_pool = Arc::new(
            Pool::builder()
                .max_size(REDIS_MAX_CONNECTIONS)
                .min_idle(Some(REDIS_MIN_IDLE))
                .build(redis_manager)
                .await?
        );

        println!("✅ Multi-tier cache system initialized");
        println!("   L1 (In-Memory): {} max entries, {}s TTL", L1_MAX_CAPACITY, L1_TTL_SECONDS);
        println!("   L2 (Redis): {}s TTL", L2_TTL_SECONDS);

        Ok(Self { 
            cache: Arc::new(MultiTierCache { l1_cache, redis_pool })
        })
    }
}
```

## 📈 Performance Characteristics

### Expected Performance
```
Cache Operation    │ L1 Response Time │ L2 Response Time │ Miss Response Time
─────────────────── │ ──────────────── │ ──────────────── │ ────────────────────
Dashboard Data     │ <1ms             │ 2-5ms            │ 50-200ms
Market Data        │ <1ms             │ 2-5ms            │ 100-500ms  
Report Data        │ <1ms             │ 2-5ms            │ 200-800ms
Technical Indicators│ <1ms             │ 2-5ms            │ 300-1000ms
```

### Cache Hit Rate Expectations
- **L1 Hit Rate**: 85-95% for frequently accessed data
- **L2 Hit Rate**: 70-85% for less frequent data
- **Overall Hit Rate**: 95%+ for steady-state operations

## 🎯 Migration Considerations

### Feature Isolation Strategy
When migrating to feature-based architecture:

1. **Shared Cache Manager**: Move to `shared/cache.rs`
2. **Feature-Specific Keys**: Each feature manages own key namespaces
3. **TTL Configuration**: Feature-specific TTL constants
4. **Health Monitoring**: Expose via dedicated health feature

### Backwards Compatibility
- All public method signatures preserved
- Cache key formats maintained
- Performance characteristics unchanged
- Error handling patterns consistent

### Testing Strategy
```rust
// Unit tests for cache operations
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_cache_or_compute_pattern() {
        // Test L1 hit, L2 hit, and miss scenarios
    }
    
    #[tokio::test]
    async fn test_redis_fallback() {
        // Test graceful degradation when Redis unavailable
    }
    
    #[tokio::test]
    async fn test_key_generation() {
        // Validate consistent key naming
    }
}
```

---

**📝 Generated**: August 20, 2025  
**🔄 Version**: 1.0  
**📊 Source Lines**: 465 lines of multi-tier cache implementation  
**🎯 Migration Target**: `shared/cache.rs` + feature-specific cache utilities
