# Generic Cache Architecture - Layer Separation

## 🎯 **Objective**
Separate business logic from cache infrastructure by making Layer 1 generic and moving business-specific knowledge to Layer 2.

## 🏗️ **Architecture Pattern**

### **Layer 1 - Infrastructure (Generic Functions)**
```rust
// Generic cache strategies - no business knowledge
enum CacheStrategy {
    ShortTerm,    // 5 minutes
    MediumTerm,   // 1 hour  
    LongTerm,     // 3 hours
    RealTime,     // 30 seconds
    Custom(Duration),
    Default
}

// Generic functions
fn cache_get(key: &str) -> Option<Value>
fn set_with_strategy(key: &str, value: Value, strategy: CacheStrategy)
fn cache_data(key: &str, value: Value, ttl_seconds: u64)
```

### **Layer 2 - Business Logic (Specific Implementations)**
```rust
// Business-specific implementations using generic Layer 1
async fn fetch_btc_with_cache() -> Result<Value> {
    cache_api_data("btc_coingecko_30s", CacheStrategy::ShortTerm, fetch_api_call)
}

async fn fetch_rsi_with_cache() -> Result<Value> {
    cache_api_data("rsi_taapi_3h", CacheStrategy::LongTerm, fetch_api_call)
}
```

## ✅ **Benefits**

### **1. Separation of Concerns**
- Layer 1: Pure caching infrastructure
- Layer 2: Business logic and API specifics

### **2. Extensibility**
- Adding new APIs only requires Layer 2 changes
- Layer 1 remains stable and reusable

### **3. Maintainability**
- No hardcoded business keys in Layer 1
- Clear responsibility boundaries

### **4. Testability**
- Layer 1 can be unit tested independently
- Layer 2 business logic isolated

## 🔄 **Implementation Pattern**

### **Generic Cache Helper (Layer 2)**
```rust
async fn cache_api_data<F, T>(
    cache_key: &str,
    strategy: CacheStrategy,
    fetch_fn: F
) -> Result<Value>
where
    F: Future<Output = Result<T>>,
    T: Serialize,
{
    // 1. Try cache first (generic get)
    if let Some(cached) = cache_get(cache_key) {
        return Ok(cached);
    }
    
    // 2. Fetch from API
    let data = fetch_fn.await?;
    
    // 3. Cache with strategy (generic set)
    cache_set_with_strategy(cache_key, data, strategy).await?;
    
    Ok(data)
}
```

### **Business-Specific Wrappers (Layer 2)**
```rust
async fn fetch_btc_price() -> Result<Value> {
    cache_api_data(
        "btc_coingecko_30s",
        CacheStrategy::ShortTerm,  // 5 min TTL
        market_api.fetch_btc_price()
    ).await
}

async fn fetch_rsi_data() -> Result<Value> {
    cache_api_data(
        "rsi_taapi_3h", 
        CacheStrategy::LongTerm,   // 3 hour TTL
        market_api.fetch_rsi()
    ).await
}
```

## 📊 **Strategy Mapping**

| Business Need | Generic Strategy | TTL | Use Case |
|---------------|------------------|-----|----------|
| BTC Price | `ShortTerm` | 5 min | Fast-changing price data |
| Fear & Greed | `ShortTerm` | 5 min | Market sentiment |
| Global Data | `RealTime` | 30 sec | Real-time updates |
| RSI/Technical | `LongTerm` | 3 hours | Technical indicators |
| Custom API | `Custom(Duration)` | Variable | Special requirements |

## 🚀 **Migration Path**

### **Before (Coupled)**
```rust
// Layer 1 knows about business logic
CacheStrategy::PriceData      // ❌ Business knowledge in infrastructure
CacheStrategy::TechnicalIndicators  // ❌ Business knowledge
```

### **After (Decoupled)**
```rust
// Layer 1 is generic
CacheStrategy::ShortTerm      // ✅ Generic time-based strategy
CacheStrategy::LongTerm       // ✅ Generic time-based strategy

// Layer 2 maps business to generic
fetch_btc_price() -> ShortTerm    // ✅ Business logic in Layer 2
fetch_rsi() -> LongTerm          // ✅ Business logic in Layer 2
```

## 🎯 **Result**
- **Layer 1**: Pure, reusable caching infrastructure
- **Layer 2**: Business-aware API integration
- **Clean Architecture**: Clear separation of concerns
- **Easy Extension**: Add new APIs without touching Layer 1
