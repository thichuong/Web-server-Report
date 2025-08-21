# Migration Plan: Cache to Layer 1 + Redis Streams

## 📋 Current State Analysis

### ✅ **Hiện tại đã có:**
- **Layer 1**: L1 Cache (Moka) + L2 Cache (Redis) + CacheManager
- **Layer 2**: Cache logic integration với External APIs
- **Layer 3**: Không có cache (đúng theo thiết kế)

### ⚠️ **Vấn đề cần giải quyết:**
- Cache logic scattered ở Layer 2 
- Thiếu Redis Streams cho real-time data
- Database là primary storage thay vì cache
- Không có event sourcing pattern

## 🎯 Target Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Layer 5: Business Logic                  │
├─────────────────────────────────────────────────────────────┤
│                    Layer 3: Communication                   │
├─────────────────────────────────────────────────────────────┤
│      Layer 2: External APIs (PURE BUSINESS LOGIC)          │
│           ↓ No Cache Logic ↓                               │
├─────────────────────────────────────────────────────────────┤
│    Layer 1: Infrastructure (ALL STORAGE CONCERNS)          │
│  ┌─────────────────────────────────────────────────────┐   │
│  │            🌊 Redis Streams (PRIMARY)                │   │
│  │  ├── crypto:market_data                             │   │
│  │  ├── crypto:btc_price                              │   │  
│  │  ├── crypto:dashboard                              │   │
│  │  └── system:health                                 │   │
│  └─────────────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────────────┐   │
│  │             💾 Cache Layers (HOT DATA)              │   │
│  │  ├── L1 Cache (Moka): 2000 entries, 5min          │   │
│  │  └── L2 Cache (Redis): 1hr TTL                     │   │
│  └─────────────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────────────┐   │
│  │            🗄️ PostgreSQL (BACKUP)                   │   │
│  │  ├── Historical data                                │   │
│  │  ├── Analytics                                     │   │
│  │  └── Disaster recovery                             │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

## 🚀 Migration Steps

### **Phase 1: Add Redis Streams to Layer 1** ✅ DONE
- [x] Created `RedisStreamManager`
- [x] Created `StreamDataService`  
- [x] Updated `CacheSystemIsland` with stream integration
- [x] Added stream interfaces alongside existing cache methods

### **Phase 2: Refactor Layer 2 (Remove Cache Logic)**
```bash
# 1. Update External APIs Island
src/service_islands/layer2_external_services/external_apis_island/mod.rs
  - Remove cache_system field
  - Remove all cache.get() calls
  - Remove all cache.set() calls
  - Focus on pure API business logic

# 2. Update API Aggregator  
src/service_islands/layer2_external_services/external_apis_island/api_aggregator.rs
  - Remove cache integration
  - Return raw API data
  - Let Layer 1 handle caching

# 3. Update Market Data API
src/service_islands/layer2_external_services/external_apis_island/market_data_api.rs  
  - Remove cache logic
  - Focus on API calls and data validation
```

### **Phase 3: Update Layer 3 Communication**
```bash
# 1. Update Layer 2 Adapters
src/service_islands/layer3_communication/layer2_adapters/
  - Remove cache-related code
  - Forward API data directly to Layer 1 streams

# 2. Update Data Communication Service
src/service_islands/layer3_communication/data_communication/
  - Integrate with Layer 1 streams
  - Use stream APIs for real-time data
```

### **Phase 4: Stream Integration Points**

#### **4.1 Data Write Flow**
```rust
// Layer 2: Fetch from External API
let api_data = external_apis.fetch_dashboard_summary().await?;

// Layer 3: Forward to Layer 1
let cache_system = &state.cache_system;
let event_id = cache_system.store_dashboard_summary(api_data).await?;

// Layer 1: Stream + Cache + Background DB Backup
// (automatic via StreamDataService)
```

#### **4.2 Data Read Flow** 
```rust
// Layer 3: Try Stream first, then fallback
let cache_system = &state.cache_system;

// Primary: Get from stream
if let Some(data) = cache_system.get_latest_market_data().await? {
    return Ok(data);
}

// Fallback: Get from traditional cache
if let Some(data) = cache_system.get("market_data").await? {
    return Ok(data);
}

// Last resort: Database query
// ... fallback to database
```

### **Phase 5: WebSocket Stream Integration**
```rust
// WebSocket service consumes from streams
let dashboard_updates = cache_system
    .get_dashboard_for_websocket().await?;

for update in dashboard_updates {
    broadcast_tx.send(update.to_string())?;
}
```

## 📊 Benefits Analysis

### **🔥 Performance Benefits**
| Aspect | Before | After |
|--------|--------|-------|
| **Read Latency** | DB Query (~50ms) | Stream Read (~1ms) |
| **Write Latency** | DB Insert (~20ms) | Stream Write (~0.5ms) |  
| **Throughput** | ~100 ops/sec | ~10,000 ops/sec |
| **Memory Usage** | DB Buffer | Redis Stream Buffer |

### **🏗️ Architecture Benefits**
- ✅ **Single Responsibility**: Layer 2 = API logic only
- ✅ **Event Sourcing**: All changes tracked in streams
- ✅ **Real-time**: Stream-based WebSocket updates
- ✅ **Scalability**: Horizontal consumer scaling  
- ✅ **Resilience**: Multi-storage redundancy

### **🛠️ Development Benefits**
- ✅ **Easier Testing**: API logic isolated from cache
- ✅ **Clear Boundaries**: Layer responsibilities well-defined
- ✅ **Stream Processing**: Event-driven architecture
- ✅ **Monitoring**: Stream metrics and health checks

## 🔧 Implementation Priority

### **High Priority (Week 1)**
1. ✅ Create Redis Stream infrastructure
2. ⏳ Refactor Layer 2 External APIs (remove cache)
3. ⏳ Update Layer 3 adapters for stream integration

### **Medium Priority (Week 2)** 
1. ⏳ WebSocket stream consumers
2. ⏳ Background database backup service
3. ⏳ Stream monitoring and alerts

### **Low Priority (Week 3+)**
1. ⏳ Stream analytics and reporting
2. ⏳ Performance optimization
3. ⏳ Advanced stream features (partitioning, etc.)

## 🚦 Migration Safety

### **Backward Compatibility**
- ✅ Keep existing cache methods during transition
- ✅ Gradual migration - component by component
- ✅ Fallback mechanisms if streams unavailable
- ✅ Comprehensive testing at each phase

### **Risk Mitigation**
- 🛡️ **Dual Storage**: Redis Streams + PostgreSQL backup
- 🛡️ **Graceful Degradation**: Fallback to traditional cache
- 🛡️ **Monitoring**: Stream health and performance metrics
- 🛡️ **Rollback Plan**: Can revert to current architecture

## 🎉 Expected Outcomes

### **Immediate Benefits**
- 📈 **50x faster read performance** (Redis vs DB)
- 🚀 **Real-time WebSocket updates** via streams
- 🧹 **Cleaner Layer 2 code** (business logic only)

### **Long-term Benefits**  
- 📊 **Event sourcing capabilities** for analytics
- 🔄 **Horizontal scaling** via consumer groups
- 💾 **Reduced database load** (backup only)
- ⚡ **Sub-millisecond data access** for hot data

---

## 📋 Next Steps

1. **Review** current Redis Stream implementation ✅
2. **Plan** Layer 2 refactoring approach
3. **Test** stream performance vs current cache
4. **Implement** gradual migration strategy
5. **Monitor** system performance during transition

**Ready to proceed with Layer 2 refactoring!** 🚀
