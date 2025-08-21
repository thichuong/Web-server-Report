# Service Islands Architecture Documentation

## Overview

Service Islands Architecture là một kiến trúc phân tầng được thiết kế để tách biệt các concerns và tạo ra hệ thống có khả năng mở rộng, bảo trì và test dễ dàng.

## Architecture Layers

### 📊 Layer Structure
```
┌─────────────────────────────────────────────────────────────┐
│                    Layer 5: Business Logic                 │
│  ┌─────────────────┐    ┌─────────────────────────────────┐│
│  │  Dashboard      │    │     Crypto Reports              ││
│  │  Island         │    │     Island                      ││
│  └─────────────────┘    └─────────────────────────────────┘│
└─────────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────────┐
│                   Layer 4: Observability                   │
│  ┌─────────────────────────────────────────────────────────┐│
│  │              Health System Island                      ││
│  └─────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────────┐
│                   Layer 3: Communication                   │
│  ┌─────────────────┐    ┌─────────────────────────────────┐│
│  │  WebSocket      │    │    Data Communication          ││
│  │  Service        │    │    Service                      ││
│  └─────────────────┘    └─────────────────────────────────┘│
└─────────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────────┐
│                  Layer 2: External Services                │
│  ┌─────────────────────────────────────────────────────────┐│
│  │              External APIs Island                      ││
│  │  • Market Data API  • Rate Limiter  • Circuit Breaker ││
│  └─────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────────┐
│                  Layer 1: Infrastructure                   │
│  ┌─────────────────┐    ┌─────────────────────────────────┐│
│  │  Shared         │    │    Cache System                 ││
│  │  Components     │    │    Island                       ││
│  │  Island         │    │  • L1 Cache (Moka)             ││
│  │                 │    │  • L2 Cache (Redis)            ││
│  └─────────────────┘    └─────────────────────────────────┘│
└─────────────────────────────────────────────────────────────┘
```

## Layer Details

### Layer 1: Infrastructure (`layer1_infrastructure/`)

#### 🧩 Shared Components Island
- **Template Registry**: Quản lý Tera templates
- **Model Registry**: Data models và structures
- **Utility Functions**: Helper functions và shared utilities

#### 🏗️ Cache System Island  
- **L1 Cache (Moka)**: In-memory cache với 2000 entries, 5min TTL
- **L2 Cache (Redis)**: Distributed cache với 1hr default TTL
- **Cache Manager**: Unified interface với intelligent promotion
- **Cache Strategies**: RealTime, ShortTerm, MediumTerm, LongTerm

### Layer 2: External Services (`layer2_external_services/`)

#### 🌐 External APIs Island
- **Market Data API**: CoinGecko, TaApi.io integration
- **Rate Limiter**: API rate limiting với exponential backoff
- **API Aggregator**: Multi-source data aggregation
- **Circuit Breaker**: Fault tolerance cho external APIs

### Layer 3: Communication (`layer3_communication/`)

#### 📡 WebSocket Service Island
- **Real-time communication**: WebSocket connections
- **Broadcasting**: Multi-client message broadcasting
- **Layer 2 Integration**: External APIs data streaming

#### 🗄️ Data Communication Service
- **Database Operations**: PostgreSQL queries
- **Cache Integration**: L2 cache cho database queries
- **Data Models**: Database-specific structures

### Layer 4: Observability (`layer4_observability/`)

#### 🔍 Health System Island
- **Health Checks**: Component health monitoring
- **System Status**: Overall system health reporting
- **Dependency Checking**: Inter-layer health validation

### Layer 5: Business Logic (`layer5_business_logic/`)

#### 📊 Dashboard Island
- **Market Data Processing**: Business logic cho dashboard
- **Layer 3 Integration**: WebSocket communication

#### 📈 Crypto Reports Island
- **Report Management**: Crypto report business logic
- **Template Orchestration**: Report rendering
- **Cache Integration**: Multi-tier caching strategy

## Data Flow Architecture

### 🔄 Request Flow
```
HTTP Request → Layer 5 (Business Logic)
                    ↓
              Layer 3 (Communication) 
                    ↓
              Layer 2 (External Services)
                    ↓
              Layer 1 (Infrastructure/Cache)
                    ↓
                Database/APIs
```

### 📊 WebSocket Flow
```
Client Connection → Layer 3 WebSocket Service
                         ↓
                   Layer 5 Business Logic
                         ↓
                   Layer 2 External APIs
                         ↓
                   Layer 1 Cache System
                         ↓
                    Real-time Updates
```

## Cache Configuration

### 🎯 Cache Strategies

#### **RealTime Strategy**
- **TTL**: 30 seconds
- **Use Case**: Live price updates, real-time indicators
- **Examples**: BTC price, market status

#### **ShortTerm Strategy**  
- **TTL**: 5 minutes
- **Use Case**: Frequently changing data
- **Examples**: Fear & Greed Index, market summaries

#### **MediumTerm Strategy**
- **TTL**: 1 hour
- **Use Case**: Semi-static data
- **Examples**: Latest crypto reports, global market data

#### **LongTerm Strategy**
- **TTL**: 3 hours  
- **Use Case**: Static/historical data
- **Examples**: Individual reports, RSI technical indicators

### 🔧 Cache Implementation

#### **L1 Cache (Moka)**
```rust
// Configuration
max_capacity: 2000 entries
time_to_live: 5 minutes (default)
time_to_idle: 2 minutes
```

#### **L2 Cache (Redis)**
```rust
// Configuration  
default_ttl: 1 hour
connection: Multiplexed async
fallback: Graceful degradation
```

#### **Cache Manager**
```rust
// Operations
get(key) -> L1 → L2 → miss
set_with_strategy(key, value, strategy)
intelligent_promotion: L2 → L1 for hot data
```

## Component Integration

### 🔗 Dependency Rules

1. **Layer 5** can access **Layer 3, 2, 1**
2. **Layer 4** can access **all layers** (monitoring)
3. **Layer 3** can access **Layer 2, 1**
4. **Layer 2** can access **Layer 1**
5. **Layer 1** is independent (no dependencies)

### 📋 Integration Patterns

#### **Cache-First Pattern**
```rust
// 1. Check L1 Cache
if let Some(data) = l1_cache.get(key).await {
    return Ok(data);
}

// 2. Check L2 Cache  
if let Some(data) = l2_cache.get(key).await {
    l1_cache.set(key, data.clone()).await; // Promote to L1
    return Ok(data);
}

// 3. Fetch from source
let data = external_source.fetch().await?;
l1_cache.set(key, data.clone()).await;
l2_cache.set(key, data.clone()).await;
Ok(data)
```

#### **Service Islands Communication**
```rust
// Layer 5 → Layer 3 → Layer 2 → Layer 1
let market_data = business_logic_island
    .request_via_communication_layer()
    .fetch_via_external_apis()
    .with_infrastructure_caching();
```

## Performance Optimizations

### ⚡ Cache Performance
- **L1 Hit Rate**: ~80% (in-memory, <1ms)
- **L2 Hit Rate**: ~15% (Redis, ~5ms)  
- **Cache Miss**: ~5% (Database/API, ~50ms)

### 🚀 Rate Limiting
- **Exponential Backoff**: 1s → 2s → 4s delays
- **Circuit Breaker**: Temporary API shutdown on failures
- **Graceful Degradation**: Cache serves stale data during outages

### 📈 Database Optimizations
- **L2 Cache Integration**: Database queries cached in Redis
- **Query Optimization**: Indexed queries với proper pagination
- **Cache Invalidation**: Smart cache clearing on data updates

## Monitoring & Observability

### 📊 Health Check Endpoints
```
GET /health/system        - Overall system health
GET /health/cache         - Cache system status  
GET /health/external-apis - External APIs status
GET /health/database      - Database connectivity
```

### 📈 Cache Metrics
```
L1 Cache: entries, hits, misses, hit_rate
L2 Cache: memory_usage, connection_pool, latency
Cache Manager: promotions, fallbacks, strategies
```

### 🔍 Logging Standards
```
🏝️ Island initialization
🔄 Inter-layer communication  
💾 Cache operations
⚠️ Error handling
✅ Success operations
```

## Development Guidelines

### 🏗️ Adding New Islands

1. **Create Island Structure**
```
src/service_islands/layerX_name/
├── mod.rs              // Island entry point
├── component1.rs       // Core components
├── component2.rs       
└── README.md          // Island documentation
```

2. **Implement Standard Interface**
```rust
impl NewIsland {
    pub async fn new() -> Result<Self> { ... }
    pub async fn health_check(&self) -> bool { ... }
    pub async fn get_statistics(&self) -> Value { ... }
}
```

3. **Add to Layer Module**
```rust
// In layerX_name/mod.rs
pub mod new_island;
pub use new_island::NewIsland;
```

### 🧪 Testing Strategy

#### **Unit Tests**
- Individual component testing
- Mock external dependencies
- Cache strategy validation

#### **Integration Tests**  
- Inter-layer communication
- End-to-end data flow
- Cache coherence testing

#### **Performance Tests**
- Cache hit/miss ratios
- Response time benchmarks
- Load testing scenarios

### 🔧 Configuration Management

#### **Environment Variables**
```bash
# Cache Configuration
REDIS_URL=redis://127.0.0.1:6379

# External APIs
COINGECKO_API_URL=https://api.coingecko.com/api/v3
TAAPI_SECRET=your_secret_here

# Database  
DATABASE_URL=postgresql://...
```

#### **Cache Tuning**
```rust
// L1 Cache Tuning
L1_MAX_CAPACITY=2000
L1_DEFAULT_TTL=300  // 5 minutes

// L2 Cache Tuning  
L2_DEFAULT_TTL=3600 // 1 hour
L2_CONNECTION_POOL=10
```

## Troubleshooting

### 🚨 Common Issues

#### **Cache Miss Storm**
```
Symptoms: High database load, slow response times
Solution: Increase cache TTL, implement cache warming
```

#### **Rate Limiting**
```
Symptoms: 429 errors, API timeouts
Solution: Exponential backoff working, check logs for retry patterns
```

#### **Layer Communication Failure**
```  
Symptoms: Service island health check failures
Solution: Check dependency injection, verify layer access patterns
```

### 📋 Debug Checklist

1. **Check Health Endpoints**: `/health/system`
2. **Verify Cache Status**: L1/L2 hit rates in logs
3. **Monitor Rate Limits**: External API status
4. **Validate Data Flow**: Layer → Layer communication
5. **Database Connectivity**: Connection pool status

## Future Enhancements

### 🚀 Planned Features

1. **Auto-scaling**: Dynamic cache sizing based on load
2. **Multi-region**: Distributed cache replication
3. **Event Sourcing**: Real-time data streaming improvements
4. **AI Integration**: Intelligent cache prefetching
5. **Metrics Dashboard**: Real-time monitoring UI

### 🔄 Migration Path

Service Islands Architecture được thiết kế để migration dần dần từ legacy code:

1. **Phase 1**: Infrastructure Layer (Cache System) ✅
2. **Phase 2**: External Services Layer (APIs) ✅  
3. **Phase 3**: Communication Layer (WebSocket) ✅
4. **Phase 4**: Business Logic Layer (Reports) ✅
5. **Phase 5**: Full Integration & Optimization 🔄

---

**Service Islands Architecture** cung cấp foundation mạnh mẽ cho cryptocurrency reporting system với focus vào performance, scalability, và maintainability.
