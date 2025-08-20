# 🚨 BUSINESS LOGIC: RATE LIMITING & CIRCUIT BREAKER SPECIFICATION

## 📊 Overview
This document specifies the intelligent rate limiting and circuit breaker system for external API calls, specifically designed for cryptocurrency data fetching with sophisticated error handling and recovery mechanisms.

## 🏗️ Architecture Components

### 1. Rate Limiting System
**Purpose**: Prevent API rate limit violations by enforcing minimum intervals between calls

#### Core Components:
```rust
// Atomic timestamp tracking
last_btc_fetch: Arc<AtomicU64>  // Unix timestamp of last BTC API call
btc_api_circuit_breaker: Arc<AtomicBool>  // Circuit breaker state
```

#### Rate Limiting Rules:
- **BTC Price API**: Minimum 3 seconds between calls
- **Other APIs**: Use retry logic without rate limiting
- **Enforcement**: Sleep if called too frequently

### 2. Circuit Breaker Pattern
**Purpose**: Temporarily disable API calls after rate limit errors to allow recovery

#### States:
- **CLOSED** (Normal): `btc_api_circuit_breaker = false`
- **OPEN** (Blocked): `btc_api_circuit_breaker = true`

#### Trigger Conditions:
- HTTP 429 (Too Many Requests) response
- Error message contains "Too Many Requests"

#### Recovery:
- **Automatic**: 5-minute timeout (300 seconds)
- **Mechanism**: Tokio spawn background task to reset breaker

## 🔧 Implementation Details

### Rate Limit Status Monitoring
```rust
pub struct RateLimitStatus {
    pub btc_api_circuit_breaker_open: bool,
    pub seconds_since_last_btc_fetch: u64,
    pub can_fetch_btc_now: bool,
}

pub fn get_rate_limit_status(&self) -> RateLimitStatus {
    let now = chrono::Utc::now().timestamp() as u64;
    let last_fetch = self.last_btc_fetch.load(Ordering::Relaxed);
    let circuit_breaker_open = self.btc_api_circuit_breaker.load(Ordering::Relaxed);
    
    RateLimitStatus {
        btc_api_circuit_breaker_open: circuit_breaker_open,
        seconds_since_last_btc_fetch: if last_fetch > 0 { now - last_fetch } else { 0 },
        can_fetch_btc_now: !circuit_breaker_open && (last_fetch == 0 || (now - last_fetch) >= 3),
    }
}
```

### BTC Price Fetch with Rate Limiting
```rust
async fn fetch_btc_price_with_rate_limit(&self) -> Result<(f64, f64)> {
    // Step 1: Circuit breaker check
    if self.btc_api_circuit_breaker.load(Ordering::Relaxed) {
        println!("⚠️ BTC API circuit breaker is open, skipping API call");
        anyhow::bail!("BTC API circuit breaker is active");
    }

    // Step 2: Rate limit enforcement
    let now = chrono::Utc::now().timestamp() as u64;
    let last_fetch = self.last_btc_fetch.load(Ordering::Relaxed);
    
    if last_fetch > 0 && (now - last_fetch) < 3 {
        let wait_time = 3 - (now - last_fetch);
        println!("⏳ Rate limiting: waiting {}s before BTC API call", wait_time);
        tokio::time::sleep(Duration::from_secs(wait_time)).await;
    }

    // Step 3: Update timestamp before call
    self.last_btc_fetch.store(now, Ordering::Relaxed);

    // Step 4: Execute call with retry logic
    match self.retry_with_backoff(|| self.fetch_btc_price(), 3).await {
        Ok(result) => {
            // Success: Reset circuit breaker
            self.btc_api_circuit_breaker.store(false, Ordering::Relaxed);
            Ok(result)
        }
        Err(err) => {
            // Rate limit error: Open circuit breaker
            if err.to_string().contains("429") || err.to_string().contains("Too Many Requests") {
                println!("🚨 BTC API rate limited - opening circuit breaker for 5 minutes");
                self.btc_api_circuit_breaker.store(true, Ordering::Relaxed);
                
                // Schedule automatic reset
                let circuit_breaker = self.btc_api_circuit_breaker.clone();
                tokio::spawn(async move {
                    tokio::time::sleep(Duration::from_secs(300)).await; // 5 minutes
                    circuit_breaker.store(false, Ordering::Relaxed);
                    println!("🔄 BTC API circuit breaker reset");
                });
            }
            Err(err)
        }
    }
}
```

### Retry Logic with Exponential Backoff
```rust
async fn retry_with_backoff<T, F, Fut>(&self, mut operation: F, max_retries: u32) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let mut retries = 0;
    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(err) => {
                retries += 1;
                if retries >= max_retries {
                    return Err(err);
                }
                
                let delay = if err.to_string().contains("429") {
                    // Rate limit error: Longer backoff (2m, 4m, 8m)
                    Duration::from_secs(120 * 2u64.pow(retries - 1))
                } else {
                    // Network/other error: Normal backoff (10s, 20s, 40s)  
                    Duration::from_secs(10 * 2u64.pow(retries - 1))
                };
                
                println!("⏳ Retry {}/{} after {}s for error: {}", 
                         retries, max_retries, delay.as_secs(), err);
                tokio::time::sleep(delay).await;
            }
        }
    }
}
```

## 🎯 API Endpoints Integration

### Individual API Methods
Each API method has specific retry and error handling:

#### Global Market Data
```rust
async fn fetch_global_data(&self) -> Result<(f64, f64)> {
    // URL: https://api.coingecko.com/api/v3/global
    // Returns: (market_cap_usd, volume_24h_usd)
    // Retry: Standard 3-retry with exponential backoff
    // Rate limiting: None (global data changes slowly)
}
```

#### BTC Price Data  
```rust
async fn fetch_btc_price(&self) -> Result<(f64, f64)> {
    // URL: https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=usd&include_24hr_change=true
    // Returns: (price_usd, change_24h_percent)
    // Rate limiting: 3-second minimum interval + circuit breaker
    // Retry: 3 attempts with exponential backoff
}
```

#### Fear & Greed Index
```rust
async fn fetch_fear_greed(&self) -> Result<u32> {
    // URL: https://api.alternative.me/fng/
    // Returns: fear_greed_index (0-100)
    // Retry: Standard 3-retry
    // Rate limiting: None
}
```

#### RSI Technical Indicator
```rust
async fn fetch_rsi(&self) -> Result<f64> {
    // URL: https://api.taapi.io/rsi with secret key
    // Returns: rsi_14_value
    // Retry: Standard 3-retry  
    // Rate limiting: None (TAAPI has generous limits)
}
```

## 🚨 Error Handling Strategy

### Error Categories
1. **Rate Limit Errors (429)**
   - Trigger circuit breaker for BTC API
   - Use longer exponential backoff (2m, 4m, 8m)
   - Log with specific rate limit messaging

2. **Network/SSL Errors**
   - Standard retry with normal backoff (10s, 20s, 40s)
   - Context-wrapped error messages
   - No circuit breaker activation

3. **JSON Parsing Errors**
   - No retry (indicates API format change)
   - Immediate failure with context

4. **HTTP Status Errors (4xx, 5xx)**
   - Standard retry for 5xx (server errors)
   - Immediate failure for 4xx (client errors, except 429)

### Logging Patterns
```rust
// Success logs
println!("📊 Market Cap: ${:.2}, Volume 24h: ${:.2}", market_cap, volume_24h);
println!("₿ BTC Price: ${:.2}, Change 24h: {:.2}%", price, change_24h);
println!("😨 Fear & Greed Index: {}", value);
println!("📈 RSI 14: {:.2}", rsi_data.value);

// Error/Warning logs
println!("⚠️ BTC API circuit breaker is open, skipping API call");
println!("⏳ Rate limiting: waiting {}s before BTC API call", wait_time);
println!("🚨 BTC API rate limited - opening circuit breaker for 5 minutes");
println!("🔄 BTC API circuit breaker reset");
println!("⏳ Retry {}/{} after {}s for error: {}", retries, max_retries, delay.as_secs(), err);
```

## 🔌 Integration Requirements

### Dependencies
```rust
use std::sync::{Arc, atomic::{AtomicU64, AtomicBool, Ordering}};
use tokio::time::{sleep, Duration};
use chrono::Utc;
use anyhow::{Context, Result};
use reqwest::Client;
```

### Configuration Constants
```rust
const BTC_RATE_LIMIT_SECONDS: u64 = 3;
const CIRCUIT_BREAKER_TIMEOUT_SECONDS: u64 = 300; // 5 minutes
const MAX_RETRIES: u32 = 3;
const BASE_RETRY_DELAY_SECONDS: u64 = 10;
const RATE_LIMIT_RETRY_DELAY_SECONDS: u64 = 120; // 2 minutes base
```

## 📊 Monitoring & Observability

### Health Check Integration
The `get_rate_limit_status()` method should be exposed via:
- `/health` endpoint for overall system health
- `/api/crypto/rate-limit-status` for detailed monitoring
- WebSocket updates for real-time dashboard updates

### Key Metrics to Track
- `seconds_since_last_btc_fetch`: For rate limiting compliance
- `btc_api_circuit_breaker_open`: For circuit breaker monitoring  
- `can_fetch_btc_now`: For system readiness
- API call success/failure rates per endpoint
- Circuit breaker activation frequency
- Average retry counts per API call

## 🎯 Migration Considerations

### Feature Isolation Requirements
When migrating to feature-based architecture:

1. **Shared HTTP Client**: Move to `shared/http_client.rs`
2. **Rate Limiting State**: Keep in dedicated service or shared state
3. **Circuit Breaker**: Should be configurable per API endpoint
4. **Monitoring Interface**: Expose via health system feature
5. **Cache Integration**: Rate limiting should respect cache layers

### Backwards Compatibility
- All public methods maintain same signatures
- Logging format should remain consistent
- Error types and messages preserved
- Performance characteristics maintained

---

**📝 Generated**: August 20, 2025  
**🔄 Version**: 1.0  
**📊 Source Lines**: 60+ lines of complex rate limiting logic  
**🎯 Migration Target**: `features/external_apis/` or `features/data_service/`
