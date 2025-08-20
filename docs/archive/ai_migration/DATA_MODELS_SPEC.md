# 📊 DATA MODELS & STRUCTURES SPECIFICATION

## 📊 Overview
This document specifies all data models, database structures, API response formats, and serialization patterns used throughout the system. Models are organized by feature domain and include validation rules, relationships, and migration patterns.

## 📋 Database Models

### 1. Report Model (Primary Entity)
**Purpose**: Core crypto market analysis report with multi-language support

```rust
use serde::{Serialize, Deserialize};
use sqlx::FromRow;

#[derive(FromRow, Serialize, Deserialize, Debug, Clone)]
pub struct Report {
    pub id: i32,
    pub html_content: String,
    pub css_content: Option<String>,
    pub js_content: Option<String>,
    pub html_content_en: Option<String>,
    pub js_content_en: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
```

**Database Schema**:
```sql
CREATE TABLE crypto_report (
    id SERIAL PRIMARY KEY,
    html_content TEXT NOT NULL,
    css_content TEXT NULL,
    js_content TEXT NULL,
    html_content_en TEXT NULL,
    js_content_en TEXT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
```

**Features**:
- **Multi-language Content**: `html_content` (Vietnamese), `html_content_en` (English)
- **Embedded Assets**: CSS and JavaScript content for self-contained reports
- **Auto Timestamps**: PostgreSQL `created_at` with timezone support
- **SQLx Integration**: `FromRow` derive for direct database mapping
- **Cache Compatibility**: `Clone` trait for cache layer integration

**Usage Patterns**:
```rust
// Database operations
let report = sqlx::query_as::<_, Report>(
    "SELECT * FROM crypto_report WHERE id = $1"
).bind(id).fetch_one(&pool).await?;

// Template context
let mut context = tera::Context::new();
context.insert("report", &report);

// Cache operations
state.report_cache.insert(report.id, report.clone()).await;
```

### 2. Report Summary Models

#### ReportSummary (Database Query Result)
```rust
#[derive(FromRow, Serialize)]
pub struct ReportSummary {
    pub id: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
```

#### ReportListItem (Frontend Display)
```rust
#[derive(Serialize)]
pub struct ReportListItem {
    pub id: i32,
    pub created_date: String,  // Formatted: "20-08-2025"
    pub created_time: String,  // Formatted: "14:30"
}
```

**Transformation Pipeline**:
```rust
// Database → Display transformation
let items: Vec<ReportListItem> = summary_items
    .into_iter()
    .map(|summary| {
        let local_time = (summary.created_at + chrono::Duration::hours(7))
            .format("%d-%m-%Y %H:%M").to_string();
        let parts: Vec<&str> = local_time.split(' ').collect();
        
        ReportListItem {
            id: summary.id,
            created_date: parts[0].to_string(),
            created_time: parts[1].to_string(),
        }
    })
    .collect();
```

## 🌐 External API Models

### 1. Dashboard Summary (Unified Data Model)
**Purpose**: Consolidated market data from multiple external APIs

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DashboardSummary {
    pub market_cap: f64,
    pub volume_24h: f64,
    pub btc_price_usd: f64,
    pub btc_change_24h: f64,
    pub fng_value: u32,
    pub rsi_14: f64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}
```

**Data Sources**:
- `market_cap`, `volume_24h`: CoinGecko Global API
- `btc_price_usd`, `btc_change_24h`: CoinGecko Simple Price API
- `fng_value`: Alternative.me Fear & Greed Index API
- `rsi_14`: TAAPI.io Technical Indicators API

**Cache Strategy**:
```rust
// Multi-tier caching with different TTLs
// BTC price: 3-second cache (real-time)
let btc_cache_key = CacheKeys::price_data("btc", "realtime");
cache_manager.set_with_ttl(&btc_cache_key, &btc_data, 3).await;

// Non-BTC data: 10-minute cache (stable)
let non_btc_key = CacheKeys::dashboard_summary_non_btc();
cache_manager.set_with_ttl(&non_btc_key, &non_btc_summary, 600).await;
```

**Frontend JSON Response**:
```json
{
  "market_cap": 2450000000000.5,
  "volume_24h": 89500000000.25,
  "btc_price_usd": 67850.42,
  "btc_change_24h": 2.34,
  "fng_value": 72,
  "rsi_14": 58.5,
  "last_updated": "2025-08-20T10:30:00Z"
}
```

### 2. Market Data Model (Generic)
**Purpose**: General cryptocurrency market data structure

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MarketData {
    pub symbol: String,
    pub price: f64,
    pub volume_24h: f64,
    pub change_24h: f64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}
```

**Usage Example**:
```rust
let market_data = MarketData {
    symbol: "BTC/USD".to_string(),
    price: 67850.42,
    volume_24h: 28500000000.0,
    change_24h: 2.34,
    last_updated: chrono::Utc::now(),
};
```

### 3. Technical Indicator Model
**Purpose**: Technical analysis data from external sources

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TechnicalIndicator {
    pub symbol: String,
    pub indicator: String,
    pub period: String,
    pub value: f64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}
```

**Example Usage**:
```rust
let rsi_indicator = TechnicalIndicator {
    symbol: "BTC/USDT".to_string(),
    indicator: "RSI".to_string(),
    period: "1d".to_string(),
    value: 58.5,
    last_updated: chrono::Utc::now(),
};
```

## 🔒 Rate Limiting Models

### 1. Rate Limit Status
**Purpose**: Track API rate limiting state and circuit breaker status

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RateLimitStatus {
    pub btc_api_circuit_breaker_open: bool,
    pub seconds_since_last_btc_fetch: u64,
    pub can_fetch_btc_now: bool,
}
```

**Implementation**:
```rust
impl DataService {
    pub fn get_rate_limit_status(&self) -> RateLimitStatus {
        let now = chrono::Utc::now().timestamp() as u64;
        let last_fetch = self.last_btc_fetch.load(Ordering::Relaxed);
        let seconds_since_last = if last_fetch > 0 { now - last_fetch } else { 0 };
        let circuit_breaker_open = self.btc_api_circuit_breaker.load(Ordering::Relaxed);
        
        RateLimitStatus {
            btc_api_circuit_breaker_open: circuit_breaker_open,
            seconds_since_last_btc_fetch: seconds_since_last,
            can_fetch_btc_now: !circuit_breaker_open && seconds_since_last >= 3,
        }
    }
}
```

**API Response**:
```json
{
  "rate_limit_status": {
    "btc_api_circuit_breaker_open": false,
    "seconds_since_last_btc_fetch": 5,
    "can_fetch_btc_now": true
  },
  "timestamp": "2025-08-20T10:30:00Z",
  "server_info": {
    "total_requests": 15678,
    "uptime_seconds": 3600
  }
}
```

## 📡 External API Response Models

### 1. CoinGecko API Structures

#### Global Market Data
```rust
#[derive(Debug, Deserialize)]
struct CoinGeckoGlobal {
    data: CoinGeckoGlobalData,
}

#[derive(Debug, Deserialize)]
struct CoinGeckoGlobalData {
    total_market_cap: HashMap<String, f64>,
    total_volume: HashMap<String, f64>,
}
```

**API Response Example**:
```json
{
  "data": {
    "total_market_cap": {
      "usd": 2450000000000.5
    },
    "total_volume": {
      "usd": 89500000000.25
    }
  }
}
```

#### Bitcoin Price Data
```rust
#[derive(Debug, Deserialize)]
struct CoinGeckoBtcPrice {
    bitcoin: BtcPriceData,
}

#[derive(Debug, Deserialize)]
struct BtcPriceData {
    usd: f64,
    usd_24h_change: f64,
}
```

**API Response Example**:
```json
{
  "bitcoin": {
    "usd": 67850.42,
    "usd_24h_change": 2.34
  }
}
```

### 2. Fear & Greed Index API
```rust
#[derive(Debug, Deserialize)]
struct FearGreedResponse {
    data: Vec<FearGreedData>,
}

#[derive(Debug, Deserialize)]
struct FearGreedData {
    value: String,  // Note: String type, needs parsing to u32
}
```

**API Response Example**:
```json
{
  "data": [
    {
      "value": "72",
      "value_classification": "Greed"
    }
  ]
}
```

**Processing Logic**:
```rust
async fn fetch_fear_greed(&self) -> Result<u32> {
    let response: FearGreedResponse = self.client
        .get(BASE_FNG_URL)
        .send().await?.json().await?;
    
    if let Some(latest) = response.data.first() {
        latest.value.parse::<u32>()
            .context("Failed to parse Fear & Greed value")
    } else {
        anyhow::bail!("No Fear & Greed data available")
    }
}
```

### 3. TAAPI Technical Indicators
```rust
#[derive(Debug, Deserialize)]
struct TaapiRsiResponse {
    value: f64,
}
```

**API Response Example**:
```json
{
  "value": 58.5
}
```

## 🏗️ Application State Models

### 1. AppState (Central State Container)
```rust
pub struct AppState {
    // Database
    pub db: PgPool,
    
    // Caching Systems
    pub cache_manager: Arc<CacheManager>,
    pub report_cache: MultiLevelCache<i32, Report>,
    pub chart_modules_cache: RwLock<Option<String>>,
    pub cached_latest_id: AtomicUsize,
    
    // External Services
    pub data_service: DataService,
    pub websocket_service: Arc<WebSocketService>,
    
    // Template Engine
    pub tera: Tera,
    
    // Metrics & Monitoring
    pub metrics: Arc<PerformanceMetrics>,
    pub request_counter: AtomicUsize,
    pub start_time: Instant,
}
```

### 2. DataService State
```rust
#[derive(Clone)]
pub struct DataService {
    client: reqwest::Client,
    taapi_secret: String,
    
    // Unified cache manager for all caching operations
    cache_manager: Option<Arc<CacheManager>>,
    
    // Rate limiting protection
    last_btc_fetch: Arc<AtomicU64>,
    btc_api_circuit_breaker: Arc<AtomicBool>,
}
```

## 📄 Template Context Models

### 1. Report Template Context
```rust
// Tera template context for report rendering
let mut context = Context::new();
context.insert("report", &report);
context.insert("chart_modules_content", &chart_modules_content);

// Formatted created date in UTC+7 timezone for display
let created_display = (report.created_at + chrono::Duration::hours(7))
    .format("%d-%m-%Y %H:%M").to_string();
context.insert("created_at_display", &created_display);
```

### 2. Report List Template Context
```rust
let reports = json!({
    "items": items,
    "total": total,
    "per_page": per_page,
    "page": page,
    "pages": pages,
    "has_prev": page > 1,
    "has_next": page < pages,
    "prev_num": if page > 1 { page - 1 } else { 1 },
    "next_num": if page < pages { page + 1 } else { pages },
    "page_numbers": page_numbers,
    "display_start": display_start,
    "display_end": display_end,
});

let mut context = Context::new();
context.insert("reports", &reports);
```

**Pagination Logic**:
```rust
let pages = (total + per_page - 1) / per_page;
let start_page = std::cmp::max(1, page - 2);
let end_page = std::cmp::min(pages, start_page + 4);
let page_numbers: Vec<i64> = (start_page..=end_page).collect();

let display_start = if total == 0 { 0 } else { offset + 1 };
let display_end = offset + (items.len() as i64);
```

## 🔄 WebSocket Message Models

### 1. Dashboard Update Messages
```rust
// WebSocket message structure for dashboard updates
{
  "type": "dashboard_update",
  "data": {
    "market_cap": 2450000000000.5,
    "volume_24h": 89500000000.25,
    "btc_price_usd": 67850.42,
    "btc_change_24h": 2.34,
    "fng_value": 72,
    "rsi_14": 58.5,
    "last_updated": "2025-08-20T10:30:00Z"
  }
}
```

### 2. Connection Status Messages
```rust
// WebSocket connection establishment
{
  "type": "connection_established",
  "message": "WebSocket connected successfully"
}

// Periodic heartbeat
{
  "type": "ping",
  "timestamp": "2025-08-20T10:30:00Z"
}
```

## 🚀 Performance & Metrics Models

### 1. Cache Statistics
```rust
// From unified cache system
{
  "cache_system": "Unified Multi-Tier (L1: In-Memory + L2: Redis)",
  "l1_cache": {
    "type": "moka::future::Cache",
    "entry_count": 145,
    "hit_count": 2847,
    "miss_count": 312,
    "hit_rate_percent": 90.1,
    "max_capacity": 2000,
    "ttl_seconds": 300,
    "healthy": true
  },
  "l2_cache": {
    "type": "Redis",
    "ttl_seconds": 3600,
    "healthy": true,
    "status": "connected"
  },
  "report_cache": {
    "entry_count": 145,
    "hit_rate_percent": 89.7,
    "latest_report_id": 1456
  },
  "overall_health": true
}
```

### 2. Performance Metrics
```rust
{
  "performance": {
    "total_requests": 15678,
    "avg_response_time_ms": 12.4,
    "cache_size": 145,
    "cache_hit_rate": 89.7
  },
  "system": {
    "available_cpus": 8,
    "thread_pool_active": true
  }
}
```

## 🎯 Migration-Friendly Patterns

### 1. Feature-Domain Organization
**Current Structure**: Monolithic models in `src/models.rs`

**Target Structure**: Feature-based model organization
```rust
// features/crypto_reports/models.rs
pub struct Report { ... }
pub struct ReportSummary { ... }
pub struct ReportListItem { ... }

// features/dashboard/models.rs  
pub struct DashboardSummary { ... }
pub struct MarketData { ... }

// features/external_apis/models.rs
pub struct CoinGeckoGlobal { ... }
pub struct BtcPriceData { ... }
pub struct FearGreedResponse { ... }

// features/rate_limiting/models.rs
pub struct RateLimitStatus { ... }
```

### 2. Trait-Based Abstractions
```rust
// Cacheable data trait
pub trait CacheableData: Serialize + DeserializeOwned + Clone + Send + Sync + 'static {
    fn cache_key(&self) -> String;
    fn cache_ttl(&self) -> Option<u64>;
}

// External API response trait
pub trait ExternalApiResponse: DeserializeOwned + Send + Sync {
    type Output;
    fn into_domain_model(self) -> Result<Self::Output>;
}

// Database entity trait
pub trait DatabaseEntity: FromRow + Serialize + Clone + Send + Sync {
    type Id;
    fn id(&self) -> Self::Id;
    fn table_name() -> &'static str;
}
```

### 3. Validation & Transformation Patterns
```rust
// Input validation
impl Report {
    pub fn validate(&self) -> Result<()> {
        if self.html_content.trim().is_empty() {
            anyhow::bail!("HTML content cannot be empty");
        }
        Ok(())
    }
    
    pub fn sanitize_content(&mut self) {
        self.html_content = html_escape::encode_safe(&self.html_content).to_string();
    }
}

// Data transformation pipelines
impl From<ReportSummary> for ReportListItem {
    fn from(summary: ReportSummary) -> Self {
        let local_time = (summary.created_at + chrono::Duration::hours(7))
            .format("%d-%m-%Y %H:%M").to_string();
        let parts: Vec<&str> = local_time.split(' ').collect();
        
        ReportListItem {
            id: summary.id,
            created_date: parts[0].to_string(),
            created_time: parts[1].to_string(),
        }
    }
}
```

## 📊 Database Migration Considerations

### 1. Schema Evolution
```sql
-- Add new columns (backward compatible)
ALTER TABLE crypto_report ADD COLUMN metadata JSONB;
ALTER TABLE crypto_report ADD COLUMN version INTEGER DEFAULT 1;

-- Create indexes for performance
CREATE INDEX idx_crypto_report_created_at ON crypto_report(created_at DESC);
CREATE INDEX idx_crypto_report_metadata ON crypto_report USING GIN(metadata);
```

### 2. Data Migration Scripts
```rust
// Migration helper functions
pub async fn migrate_report_format(pool: &PgPool) -> Result<()> {
    let reports = sqlx::query_as::<_, Report>(
        "SELECT * FROM crypto_report WHERE version < 2"
    ).fetch_all(pool).await?;
    
    for mut report in reports {
        // Transform old format to new format
        report.sanitize_content();
        
        sqlx::query(
            "UPDATE crypto_report SET html_content = $1, version = 2 WHERE id = $2"
        )
        .bind(&report.html_content)
        .bind(report.id)
        .execute(pool).await?;
    }
    
    Ok(())
}
```

---

**📝 Generated**: August 20, 2025  
**🔄 Version**: 1.0  
**📊 Source Lines**: 97 lines models + 563 lines data service structures  
**🎯 Migration Target**: `features/*/models.rs` + trait-based abstractions
