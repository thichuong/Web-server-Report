# Unified Cache System Architecture

## Overview
Hệ thống cache thống nhất L1 (In-Memory) + L2 (Redis) được thiết kế để cung cấp một interface đơn giản và nhất quán cho tất cả các hoạt động cache trong ứng dụng.

## Architecture

### 🚀 CacheManager - Unified Interface
`CacheManager` là interface chính cho tất cả các hoạt động cache. Nó cung cấp:

- **Cache-or-compute pattern**: Tự động kiểm tra cache trước khi thực hiện computation
- **Specialized caching methods**: Các phương thức chuyên biệt cho từng loại dữ liệu
- **TTL management**: Quản lý thời gian sống khác nhau cho các loại dữ liệu
- **Health monitoring**: Kiểm tra sức khỏe của cả L1 và L2

### 🔄 Multi-Tier Cache Flow
```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   Client    │───▶│ CacheManager│───▶│   DataAPI   │
└─────────────┘    └─────────────┘    └─────────────┘
                          │
                          ▼
               ┌─────────────────────┐
               │   MultiTierCache    │
               └─────────────────────┘
                          │
                ┌─────────┴─────────┐
                ▼                   ▼
        ┌─────────────┐     ┌─────────────┐
        │ L1 (Moka)   │     │ L2 (Redis)  │
        │ In-Memory   │     │ Distributed │
        │ 5m TTL      │     │ 1h TTL      │
        └─────────────┘     └─────────────┘
```

## Usage Examples

### 1. Dashboard Data Caching
```rust
// Automatic cache-or-compute với 5 phút TTL
let summary = cache_manager.cache_dashboard_data(|| {
    fetch_from_apis()
}).await?;
```

### 2. Market Data với Short TTL
```rust
// Market data với 1 phút TTL để đảm bảo tính real-time
let market_data = cache_manager.cache_market_data("BTC", || {
    fetch_btc_price()
}).await?;
```

### 3. Report Data với Medium TTL
```rust
// Report data với 30 phút TTL
let report = cache_manager.cache_report_data("BTC", "1D", || {
    generate_report()
}).await?;
```

### 4. Custom Cache Operations
```rust
// Generic cache-or-compute với custom TTL
let data = cache_manager.cache_or_compute(
    "custom:key", 
    900, // 15 minutes
    || fetch_custom_data()
).await?;
```

## TTL Configuration

| Data Type | L1 TTL | L2 TTL | Use Case |
|-----------|--------|--------|----------|
| Dashboard | 5m | 5m | Real-time summary |
| Market Data | 1m | 1m | Price updates |
| Reports | 30m | 30m | Analysis reports |
| User Data | 15m | 15m | User preferences |
| Technical Indicators | 5m | 5m | TA calculations |

## DataService Integration

### Before (Old Pattern)
```rust
// Manual cache management
if let Some(cached) = cache.get("key").await? {
    return Ok(cached);
}
let fresh = fetch_data().await?;
cache.set("key", &fresh).await?;
Ok(fresh)
```

### After (Unified Pattern)
```rust
// Automatic cache management
cache_manager.cache_dashboard_data(|| {
    self.fetch_dashboard_summary_direct()
}).await
```

## Key Features

### 🎯 Automatic Cache Management
- Tự động kiểm tra cache trước khi fetch data
- Transparent promotion từ L2 lên L1
- Graceful fallback khi Redis không khả dụng

### 🔧 Flexible TTL Configuration
- Different TTL cho different data types
- Custom TTL support cho special cases
- Automatic expiration management

### 📊 Health Monitoring
```rust
let health = cache_manager.health_check().await;
// {
//   "l1_healthy": true,
//   "l2_healthy": true,
//   "overall_healthy": true
// }
```

### 🧹 Pattern-based Cache Clearing
```rust
// Clear specific patterns
cache_manager.clear_pattern("dashboard:*").await?;
cache_manager.clear_pattern("market:btc:*").await?;
```

## Migration Guide

### 1. Update Dependency Injection
```rust
// Old
let data_service = DataService::with_cache(secret, cache);

// New  
let cache_manager = Arc::new(CacheManager::new(redis_url).await?);
let data_service = DataService::with_cache_manager(secret, cache_manager);
```

### 2. Update Method Calls
```rust
// Old
data_service.fetch_dashboard_summary_cached(&cache).await?;

// New
data_service.fetch_dashboard_summary().await?; // Uses cache automatically
```

### 3. Update State Management
```rust
// Old
pub struct AppState {
    pub cache: Arc<MultiTierCache>,
}

// New
pub struct AppState {
    pub cache_manager: Arc<CacheManager>,
}
```

## Performance Benefits

### 🚀 Reduced Latency
- L1 cache: ~0.1ms average lookup time
- L2 cache: ~1-2ms average lookup time
- API calls: ~100-500ms average

### 📈 Improved Hit Rates
- Intelligent promotion từ L2 lên L1
- Optimized TTL cho different data types
- Reduced external API calls

### 🔄 Better Resource Utilization
- Shared cache pool connection management
- Automatic cleanup của expired entries
- Memory-efficient serialization

## Monitoring & Debugging

### Cache Statistics
```bash
GET /admin/cache/stats
```

Response:
```json
{
  "cache_system": "Unified Multi-Tier (L1: In-Memory + L2: Redis)",
  "l1_cache": {
    "type": "moka::future::Cache",
    "entry_count": 245,
    "hit_count": 1832,
    "miss_count": 423,
    "hit_rate_percent": 81.2
  },
  "l2_cache": {
    "type": "Redis",
    "healthy": true,
    "status": "connected"
  }
}
```

### Cache Management
```bash
# Clear all cache
POST /admin/cache/clear

# Health check
GET /health
```

## Best Practices

### 1. Use Specialized Methods
- `cache_dashboard_data()` cho dashboard
- `cache_market_data()` cho market prices
- `cache_report_data()` cho reports

### 2. Handle Cache Errors Gracefully
```rust
// Cache errors không làm fail request
match cache_manager.get("key").await {
    Ok(Some(data)) => return Ok(data),
    Ok(None) => {}, // Cache miss - proceed to fetch
    Err(e) => eprintln!("Cache error: {}", e), // Log but continue
}
```

### 3. Use Appropriate TTLs
- Short TTL (1-5m): Real-time data (prices, indicators)
- Medium TTL (15-30m): Reports, analysis
- Long TTL (1h+): Configuration, static data

### 4. Monitor Cache Performance
- Track hit rates through `/admin/cache/stats`
- Monitor health status through `/health`
- Set up alerts cho cache degradation

## Future Enhancements

### 🎯 Planned Features
- [ ] Custom hit/miss counters cho moka cache
- [ ] Cache warming strategies
- [ ] Distributed cache invalidation
- [ ] Automatic cache size optimization
- [ ] Cache compression for large payloads

### 🔧 Possible Improvements
- Background refresh cho critical data
- Circuit breaker pattern cho Redis failures
- Cache analytics và usage patterns
- Multi-region cache replication
