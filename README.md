# Web Server Report - High-Performance Crypto Dashboard

🚀 **Ultra-fast Rust web server** achieving **16,829+ RPS** with **5.2ms latency** for crypto investment reports with advanced multi-threading, Cache Stampede Protection, and real-time features.

## ✨ Key Features

### 🎯 Core Functionality
- **Interactive Crypto Reports**: Dynamic investment reports with Chart.js visualizations
- **Multi-language Support**: Vietnamese/English with seamless switching
- **Responsive Design**: Mobile-first, adaptive UI
- **PDF Generation**: Export reports to PDF format
- **Real-time Updates**: WebSocket integration for live data
- **API Resilience**: Binance + CoinGecko + CoinMarketCap fallback system for 99.9% uptime

### ⚡ Performance Optimizations
- **Cache Stampede Protection**: DashMap+Mutex request coalescing for L2, Moka's get_with() for L1
- **Multi-tier Cache System**: L1 (In-Memory) + L2 (Redis) with automatic promotion
- **Multi-threaded Architecture**: Thread-safe operations with concurrent processing  
- **Concurrent Request Processing**: Handle 16,829+ RPS with 5.2ms average latency
- **Lock-free Operations**: Atomic counters and non-blocking data structures
- **Parallel CPU Tasks**: Background template rendering with spawn_blocking
- **Unified Cache Manager**: Single API for all caching operations with stampede protection
- **Database Connection Pool**: Optimized for 16-core systems (32 max connections)
- **Chart Module Bundling**: Optimized JavaScript asset delivery

### 🛡️ Reliability Features
- **Automatic API Fallback**: Binance → CoinGecko → CoinMarketCap seamless switching
- **Data Validation**: Prevents corrupted data from affecting reports
- **Cache-first Data Strategy**: Cache persistence với intelligent API fallback logic
- **Circuit Breaker Pattern**: Automatic recovery from API failures
- **Source Attribution**: Track which APIs provided data for debugging

### 🔧 Technical Stack
- **Backend**: Rust + Axum (high-performance async web framework)
- **Database**: PostgreSQL with optimized connection pooling (32 max connections)
- **Caching**: Multi-tier L1 (moka) + L2 (Redis) with Cache Stampede Protection
- **Market Data**: Binance (primary) + CoinGecko + CoinMarketCap (fallback) + TAAPI.io + Finnhub (US stocks)
- **Concurrency**: Rayon ThreadPool + tokio async runtime + DashMap request coalescing
- **Real-time**: Redis + WebSocket for live updates
- **Templates**: Tera template engine with background rendering
- **Frontend**: Vanilla JS with Chart.js and modern CSS
- **API Resilience**: Multi-source data with Binance + CoinGecko + CoinMarketCap fallback + Finnhub US stocks

## 🚀 Quick Start

### Prerequisites
- Rust 1.70+ ([Install Rust](https://rustup.rs/))
- PostgreSQL database
- Redis server (optional, for WebSocket features)
- CoinMarketCap API key (optional, for fallback support)
- Finnhub API key (optional, for US stock market indices)

### 1. Clone & Setup
```bash
git clone https://github.com/thichuong/Web-server-Report.git
cd Web-server-Report

# Copy environment template
cp .env.example .env
```

### 2. Configure Environment
Edit `.env` with your settings:
```env
# Database connection
DATABASE_URL=postgresql://username:password@localhost:5432/database_name

# Security
AUTO_UPDATE_SECRET_KEY=your_secret_key_here

# External APIs
TAAPI_SECRET=your_taapi_secret_for_crypto_data
CMC_API_KEY=your_coinmarketcap_api_key_here    # Optional - enables crypto fallback support
FINNHUB_API_KEY=your_finnhub_api_key_here      # Optional - enables US stock market data

# Optional: Redis for WebSocket/caching (defaults to localhost:6379)
REDIS_URL=redis://localhost:6379

# Server configuration
HOST=0.0.0.0
PORT=8000

# Development mode (enables debug logging)
DEBUG=1
```

### 3. Build & Run
```bash
# Development build
cargo run

# Production build (optimized)
cargo build --release
./target/release/web-server-report
```

Server will start at `http://localhost:8000` 🎉

## 🏗️ Architecture & Performance

### 🆕 Recent Upgrades (Latest)

#### Redis Streams Integration (Real-time Data Pipeline)
- **📤 Stream Publishing**: Market data automatically published to Redis Streams for external consumers
- **🔄 Bi-directional Communication**: Layer 3 publishes to streams, enables Python AI service consumption
- **⚡ Sub-millisecond Publishing**: Non-blocking stream writes with <1ms overhead
- **🎯 Consumer-Ready Format**: Flattened JSON key-value pairs optimized for stream consumers
- **🛡️ Fault Tolerance**: Stream publishing failures don't affect core functionality
- **📊 Stream Monitoring**: Track stream health via `/health` endpoint

#### Layer 1 Infrastructure Enhancements
- **🗂️ Cache Manager Redis Streams**: Native Redis Stream methods in CacheManager
  - `publish_to_stream()`: XADD with automatic trimming support
  - `read_stream_latest()`: Retrieve N latest entries (newest first)
  - `read_stream()`: Blocking/non-blocking stream consumption with XREAD
- **🏝️ App State Island**: Unified application state management with Redis Streams support
- **📦 Chart Modules Island**: Optimized JavaScript bundling with cache integration
- **🔧 Shared Components**: Template registry and utilities across all layers

#### Layer 2 External Services Improvements
- **🌐 External APIs Island**: Enhanced circuit breaker with stream publishing
- **💾 Cache-first Strategy**: API responses cached before stream publishing
- **🔄 Multi-source Fallback**: Binance → CoinGecko → CoinMarketCap with stream integration
- **📡 US Stock Data**: Finnhub integration with stream publishing for indices

#### Layer 3 Communication Upgrades
- **📊 Market Data Adapter**: 
  - Automatic Redis Streams publishing after Layer 2 data fetch
  - Stream entry ID tracking for debugging
  - Non-critical error handling (continues on stream failure)
- **🔌 WebSocket Service**: Ready for Redis Streams consumer integration (Phase 3)
- **💬 Dashboard Communication**: Stream-aware data routing
- **🌉 Layer 2 Adapters**: Clean API abstraction with stream publishing

### Service Islands Architecture
Hệ thống sử dụng **Service Islands Architecture** - kiến trúc phân tầng 5 lớp với separation of concerns rõ ràng:

```
┌─────────────────────────────────────────────────────────────┐
│                    Layer 5: Business Logic                 │
│  ┌─────────────────┐    ┌─────────────────────────────────┐│
│  │  Dashboard      │    │     Crypto Reports              ││
│  │  Island         │    │     Island                      ││
│  │ • Market Data   │    │ • Report Management             ││
│  │   Processing    │    │ • Template Orchestration        ││
│  │ • WebSocket     │    │ • Cache Integration             ││
│  │   Integration   │    │                                 ││
│  └─────────────────┘    └─────────────────────────────────┘│
└─────────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────────┐
│                   Layer 4: Observability                   │
│  ┌─────────────────────────────────────────────────────────┐│
│  │              Health System Island                      ││
│  │ • Component Health Monitoring                          ││
│  │ • System Status Reporting                              ││
│  │ • Inter-layer Health Validation                        ││
│  └─────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────────┐
│                   Layer 3: Communication                   │
│  ┌─────────────────┐    ┌─────────────────────────────────┐│
│  │  WebSocket      │    │    Data Communication          ││
│  │  Service        │    │    Service                      ││
│  │ • Real-time     │    │ • Database Operations           ││
│  │   Communication │    │ • Cache Integration             ││
│  │ • Broadcasting  │    │ • Data Models                   ││
│  └─────────────────┘    └─────────────────────────────────┘│
└─────────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────────┐
│                  Layer 2: External Services                │
│  ┌─────────────────────────────────────────────────────────┐│
│  │              External APIs Island                      ││
│  │ • Market Data API (Binance, CoinGecko, CoinMarketCap) ││
│  │ • US Stock Indices (Finnhub)                          ││
│  │ • Cache-first Strategy with Data Persistence          ││
│  │ • API Aggregator (Multi-source data + Cache storage)  ││
│  │ • Circuit Breaker (Fault tolerance)                   ││
│  └─────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────────┐
│                  Layer 1: Infrastructure                   │
│  ┌─────────────────┐    ┌─────────────────────────────────┐│
│  │  Shared         │    │    Cache System                 ││
│  │  Components     │    │    Island                       ││
│  │  Island         │    │ • L1 Cache (Moka)              ││
│  │ • Template      │    │   - 2000 entries, 5min TTL     ││
│  │   Registry      │    │ • L2 Cache (Redis)             ││
│  │ • Model         │    │   - 1hr default TTL            ││
│  │   Registry      │    │ • Cache Manager                ││
│  │ • Utilities     │    │   - Unified interface          ││
│  └─────────────────┘    │ • Cache Strategies              ││
│                         │   - ShortTerm, MediumTerm       ││
│                         │   - LongTerm, RealTime          ││
│                         └─────────────────────────────────┘│
└─────────────────────────────────────────────────────────────┘
```

### Generic Cache Architecture (Layer Separation)
```
┌─────────────────────────────────────────────────────────────┐
│ Layer 2: Business Logic (API-specific implementations)     │
│                                                             │
│ fetch_btc_price() ────► CacheStrategy::ShortTerm (5min)     │
│ fetch_rsi_data() ─────► CacheStrategy::LongTerm (3hr)       │
│ fetch_fear_greed() ───► CacheStrategy::RealTime (30s)       │
│                                                             │
└─────────────────────┬───────────────────────────────────────┘
                      │ Business-aware wrappers
                      ▼
┌─────────────────────────────────────────────────────────────┐
│ Layer 1: Infrastructure (Generic cache functions)          │
│                                                             │
│ cache_get(key) ──────────────┐                            │
│ set_with_strategy(key, value, strategy) ─┐                 │
│ cache_data(key, value, ttl) ──────────────┼──► L1+L2 Cache │
│                                           │                 │
│ Generic Strategies:                       │                 │
│ • ShortTerm, MediumTerm, LongTerm        │                 │
│ • RealTime, Custom(Duration), Default    │                 │
└───────────────────────────────────────────┼─────────────────┘
                                           │
                    ┌──────────────────────┴──────────────────────┐
                    │                                             │
                    ▼                                             ▼
        ┌──────────────────┐                          ┌──────────────────┐
        │   L1: moka       │                          │   L2: Redis      │
        │   (In-Memory)    │ ◄──── Promotion ────────► │   (Distributed)  │
        │ • 2000 entries   │                          │ • Persistence    │
        │ • 5min TTL       │                          │ • 1hr default    │
        │ • <1ms response  │                          │ • 2-5ms response │
        └──────────────────┘                          └──────────────────┘
```

### Request Flow Through Service Islands
```
Client Request ───► Axum Router
                           │
                           ▼
              ┌─────────────────────────┐
              │   Layer 5: Business     │ ──► Template Rendering
              │   • Dashboard Island    │     Report Processing
              │   • Crypto Reports      │
              └─────────┬───────────────┘
                        │ Business Logic Processing
                        ▼
              ┌─────────────────────────┐
              │   Layer 3: Comm        │ ──► PostgreSQL
              │   • Data Communication │     WebSocket Broadcasting
              └─────────┬───────────────┘
                        │ Data Fetching
                        ▼
              ┌─────────────────────────┐
              │   Layer 2: External    │ ──► Binance API
              │   • APIs Island        │     TaApi.io API  
              │   • Cache-first        │     Circuit Breaker
              └─────────┬───────────────┘
                        │ Cache Integration
                        ▼
              ┌─────────────────────────┐
              │   Layer 1: Cache       │ ──► L1 (moka) ⚡<1ms + Stampede Protection
              │   • Generic Strategies │     L2 (Redis) 🔥2-5ms + Request Coalescing
              │   • Unified Manager    │     Cache Miss 💻200ms+ (single request only)
              └─────────────────────────┘
```

### Service Islands Performance Metrics

#### Redis Streams Performance (NEW)
- **📤 Publish Latency**: <1ms average (non-blocking XADD)
- **📊 Stream Throughput**: 10,000+ entries/sec sustained
- **🔄 Consumer Lag**: Sub-second for Python AI service integration
- **💾 Stream Retention**: Auto-trimming at 1000 entries (configurable)
- **🎯 Field Encoding**: Flattened JSON → Stream fields in <0.5ms

#### Cache Performance (Layer 1 Infrastructure) - **with Cache Stampede Protection**
- **L1 Hit Rate**: ~90% (sub-millisecond response)
- **L2 Hit Rate**: ~75% (2-5ms with automatic L1 promotion)  
- **Stampede Protection**: 99.6% improvement in high-concurrency scenarios
- **Request Coalescing**: DashMap+Mutex for L2, Moka's get_with() for L1
- **Overall Coverage**: ~95% (giảm 95% external API calls)
- **Generic Strategies**: ShortTerm(5min), MediumTerm(1hr), LongTerm(3hr), RealTime(30s)

#### Business Logic Performance (Layer 5)
- **Dashboard Island**: Real-time market data processing với WebSocket integration
- **Crypto Reports Island**: Template orchestration với multi-tier caching
- **Report Generation**: Background processing với spawn_blocking

#### Communication Layer Performance (Layer 3) 
- **WebSocket Service**: Real-time broadcasting tới multiple clients
- **Data Communication**: PostgreSQL connection pool (32 max connections)
- **Cache Integration**: L2 cache cho database queries

#### External Services Performance (Layer 2)
- **Cache-first Strategy**: Binance API primary với cache persistence
- **Circuit Breaker**: Fault tolerance cho external APIs
- **API Aggregator**: Multi-source data với intelligent failover và cache storage

#### Infrastructure Performance (Layer 1)
- **🚄 16,829+ RPS**: Handle 16,829+ concurrent requests per second with Cache Stampede Protection
- **⚡ Sub-1ms L1 Cache**: Moka in-memory cache hits with get_with() coalescing
- **🔥 2-5ms L2 Cache**: Redis distributed cache với DashMap+Mutex request coalescing
- **🛡️ 99.6% Stampede Protection**: Prevents cache stampede in high-concurrency scenarios  
- **🔄 Multi-threaded**: Rayon ThreadPool + tokio async runtime
- **📊 95% Cache Coverage**: Generic cache strategies reduce API calls
- **🏗️ Service Islands**: Clean separation of concerns across 5 layers

### Benchmark Results
```
📊 Performance Test Results (16 CPU cores) - **WITH CACHE STAMPEDE PROTECTION**:

⚡ HTTP Load Testing Results:
╭─────────────────────────────────────────────────────────────────╮
│                    BURST LOAD SCENARIO                         │  
│ 📈 Peak Performance: 16,829.3 req/s (5.2ms avg latency)       │
│ 🎯 Success Rate: 100% (0 failures across all scenarios)       │
│ 🛡️ Stampede Protection: 99.6% improvement vs unprotected      │
│ 🔄 Concurrent Clients: 100 sustained connections              │
╰─────────────────────────────────────────────────────────────────╯

Historical Performance Comparison:
• Before Optimization:  534ms avg latency (high-concurrency)
• After Route Optimization: ~13ms avg latency  
• With Stampede Protection: 5.2ms avg latency (16,829+ RPS)

Multi-tier Cache Performance:
• L1 Cache Hit:    <1ms (90% hit rate) + get_with() coalescing
• L2 Cache Hit:  2-5ms (75% hit rate) + DashMap request deduplication  
• Cache Miss:   200ms+ (single request only, others wait)
• Overall Coverage: 95% (drastically reduced API calls)
```

### Service Islands Request Flow
1. **Client Request** → Axum Router → Layer 5 Business Logic
2. **Dashboard Island** → Market data processing → Layer 3 Communication
3. **Data Communication** → PostgreSQL/Cache lookup → Layer 2 External Services
4. **External APIs Island** → Rate-limited API calls → Layer 1 Infrastructure  
5. **Cache System Island** → Generic cache strategies (L1: <1ms, L2: 2-5ms)
6. **📤 Redis Streams Publishing** → Market data published to stream (async, non-blocking)
7. **Response** → Multi-tier cache storage → Client delivery

#### Redis Streams Data Flow (NEW)
```
Layer 2 External APIs (Binance/CoinGecko/CMC)
        ↓
Layer 3 Market Data Adapter
        ↓
Layer 1 Cache Manager (L1 + L2 caching)
        ↓
Redis Streams Publishing (XADD)
        ↓
External Consumers (Python AI Service, Analytics, etc.)
```

#### Cache Strategy Mapping
- **BTC Price**: `ShortTerm` strategy (5min TTL) - Fast-changing data
- **Technical Indicators**: `LongTerm` strategy (3hr TTL) - RSI, MACD
- **Fear & Greed**: `RealTime` strategy (30s TTL) - Market sentiment
- **Global Data**: `MediumTerm` strategy (1hr TTL) - Market stats

## 🛡️ Cache Stampede Protection

### What is Cache Stampede?
Cache stampede occurs when multiple concurrent requests hit the same expired cache key simultaneously, causing all requests to fetch data from the expensive source (API calls). This can overwhelm external APIs and degrade performance.

### Our Implementation
```rust
// L1 Cache (Moka) - Built-in protection with get_with()
let data = cache.get_with(key, fetch_function).await;

// L2 Cache (Redis) - Custom DashMap+Mutex coalescing
let pending_requests = Arc<DashMap<String, Arc<Mutex<()>>>>::new();
if let Some(guard) = pending_requests.get(&key) {
    let _lock = guard.lock().await; // Wait for ongoing request
    return cache_get(&key).await;   // Get result from cache
}

// Redis Streams Integration (NEW)
// After cache update, publish to stream for external consumers
cache_manager.publish_to_stream("market_data_stream", fields, Some(1000)).await?;
```

### Performance Impact
- **🚀 99.6% Performance Improvement** in high-concurrency scenarios
- **⚡ 16,829+ RPS** peak performance with 5.2ms average latency
- **🛡️ Single API Call** per cache key expiration (vs N concurrent calls)
- **🔄 Request Coalescing** eliminates redundant external API requests
- **📤 Stream Publishing** adds <1ms overhead, enables real-time data pipeline

### Architecture Benefits
- **L1 Protection**: Moka's `get_with()` ensures only one computation per key
- **L2 Protection**: DashMap+Mutex prevents stampede on Redis misses  
- **Dual-Layer Defense**: Both cache levels protected independently
- **Zero Data Loss**: All requests receive the same valid result
- **📡 Stream Integration**: Cached data automatically published to Redis Streams for external consumers

## 📡 API Reference

### Core Endpoints
| Method | Endpoint | Description | Performance |
|--------|----------|-------------|-------------|
| `GET` | `/` | Homepage with latest report | 16,829+ RPS |
| `GET` | `/health` | Server health check + metrics | - |
| `GET` | `/metrics` | Performance metrics | - |
| `GET` | `/crypto_report` | Latest crypto report | 16,829+ RPS |
| `GET` | `/crypto_report/:id` | Specific report by ID | 16,829+ RPS |
| `GET` | `/pdf-template/:id` | PDF-optimized report view | ✅ Cached + Stampede Protected |
| `GET` | `/crypto_reports_list` | Paginated report list | - |

### Admin & Monitoring
| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/health` | Server health + unified cache metrics + Redis Streams status |
| `GET` | `/cache-stats` | Detailed L1/L2 cache statistics + stream metrics |
| `POST` | `/clear-cache` | Clear all cache tiers (L1+L2) |

### Real-time & API
| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/ws` | WebSocket connection for real-time updates |
| `GET` | `/api/crypto/dashboard-summary` | Cached dashboard data with crypto + US stocks (JSON) + Stream publish |
| `GET` | `/api/crypto/dashboard-summary/refresh` | Force refresh dashboard + Stream publish |

### Static Assets
| Path | Description |
|------|-------------|
| `/shared_assets/js/chart_modules.js` | Bundled chart JavaScript |
| `/shared_assets/css/` | Stylesheets |
| `/crypto_dashboard/assets/` | Dashboard-specific assets |

## 🗂️ Service Islands Cache System

Hệ thống implement **Generic Cache Architecture** với Layer Separation để tách biệt business logic khỏi cache infrastructure:

### Layer 1: Infrastructure (Generic Cache + Redis Streams)
- **L1 Cache**: `moka::future::Cache` - Ultra-fast in-memory (2000 entries, 5min TTL)
- **L2 Cache**: Redis - Distributed cache with persistence (1hr default TTL)
- **🆕 Redis Streams**: Native stream support with XADD/XREAD operations
  - `publish_to_stream()`: Publish market data to streams
  - `read_stream_latest()`: Retrieve latest N entries
  - `read_stream()`: Blocking/non-blocking stream consumption
- **Generic Strategies**: ShortTerm, MediumTerm, LongTerm, RealTime, Custom
- **Unified API**: Pure caching infrastructure, không business knowledge

### Layer 2: Business Logic (API-Specific + Stream Publishing)
- **Business Wrappers**: API-specific implementations using generic Layer 1
- **Strategy Mapping**: Business needs mapped to generic cache strategies
- **Cache Keys**: Business-aware cache key generation
- **🆕 Stream Integration**: Automatic stream publishing after API data fetch

### Layer 3: Communication (Enhanced with Streams)
- **🆕 Market Data Adapter**: Publishes to Redis Streams after caching
- **WebSocket Service**: Ready for stream consumer integration
- **Data Communication**: Stream-aware data routing

### Cache Architecture Benefits
- **Separation of Concerns**: Layer 1 pure caching, Layer 2 business logic
- **Extensibility**: Add new APIs chỉ cần thay đổi Layer 2
- **Maintainability**: Không hardcoded business keys trong Layer 1
- **Testability**: Layer 1 unit test độc lập, Layer 2 business logic isolated
- **🆕 Real-time Pipeline**: Redis Streams enables external consumer integration

### Cache Usage Patterns

#### 1. **Generic Cache Helper (Layer 2) + Redis Streams**
```rust
async fn cache_api_data<F, T>(
    cache_key: &str,
    strategy: CacheStrategy,  // Generic strategy
    fetch_fn: F
) -> Result<Value> {
    // Fetch and cache data
    let data = cache_manager.get_or_compute_with(key, strategy, fetch_fn).await?;
    
    // Publish to Redis Streams for external consumers
    cache_manager.publish_to_stream("market_data_stream", fields, Some(1000)).await?;
    
    Ok(data)
}
```

#### 2. **Business-Specific Wrappers (Layer 2) with Streams**
```rust
fetch_btc_price() → cache_api_data("btc_coingecko", ShortTerm, api_call) → Stream publish
fetch_rsi_data() → cache_api_data("rsi_taapi", LongTerm, api_call) → Stream publish
fetch_fear_greed() → cache_api_data("fear_greed", RealTime, api_call) → Stream publish
```

#### 3. **WebSocket Broadcasting (Layer 3) + Stream Consumers**
```rust
WebSocketService → Redis pub/sub → Real-time updates (existing)
StreamConsumer → Redis Streams → Python AI service (NEW)
```

#### 4. **Redis Streams Operations (Layer 1)**
```rust
// Publish to stream
let entry_id = cache_manager.publish_to_stream(
    "market_data_stream",
    vec![("btc_price".to_string(), "50000".to_string())],
    Some(1000) // Max 1000 entries
).await?;

// Read latest entries
let latest = cache_manager.read_stream_latest("market_data_stream", 10).await?;

// Blocking read for new entries
let new_entries = cache_manager.read_stream(
    "market_data_stream", 
    "$",  // Only new entries
    100,
    Some(5000)  // Block for 5 seconds
).await?;
```

### Cache Monitoring
- **Health**: `/health` endpoint shows L1/L2 status, hit rates, and Redis Streams health
- **Statistics**: `/cache-stats` provides detailed cache metrics + stream entry counts
- **Management**: `/clear-cache` clears all cache tiers
- **Performance**: 95% cache coverage, <1ms L1 hits, 2-5ms L2 hits
- **🆕 Stream Metrics**: Track published entries, consumer lag, stream throughput

📖 **Detailed Documentation**: See [CACHE_ARCHITECTURE.md](./CACHE_ARCHITECTURE.md) for complete implementation guide.

## 🚀 Deployment

### Railway (Recommended)

#### 1. Prepare Railway Project
```bash
# Install Railway CLI
npm install -g @railway/cli

# Login and create project
railway login
railway link
```

#### 2. Setup Database
1. Go to [Railway Dashboard](https://railway.app)
2. Add PostgreSQL service from templates
3. Copy `DATABASE_URL` from Variables tab

#### 3. Deploy via GitHub (Recommended)
1. Push code to GitHub repository
2. Connect repository in Railway dashboard
3. Railway auto-detects Rust project and builds

#### 4. Configure Environment Variables
```env
DATABASE_URL=<your-postgresql-url>
AUTO_UPDATE_SECRET_KEY=<secure-secret>
TAAPI_SECRET=<crypto-api-key>
REDIS_URL=<redis-url-if-available>
HOST=0.0.0.0
PORT=8000
```

#### 5. Custom Domain (Optional)
- Add custom domain in Railway Settings

### Docker Deployment

```bash
# Build Docker image
docker build -t crypto-dashboard .

# Run with environment
docker run -p 8000:8000 \
  -e DATABASE_URL="postgresql://..." \
  -e AUTO_UPDATE_SECRET_KEY="..." \
  crypto-dashboard
```

### Production Tips
- Use `cargo build --release` for optimized builds
- Set up Redis for WebSocket features in production
- Configure reverse proxy (nginx) for SSL/domain routing
- Monitor memory usage of report cache (grows with unique report IDs accessed)

## 🏗️ Project Structure (Service Islands Architecture)

```
Web-server-Report/
├── 📁 src/
│   ├── 🦀 main.rs              # Server initialization + Service Islands setup
│   ├── 📊 performance.rs       # Performance monitoring across layers
│   ├── 🏗️ state.rs             # Application state + Service Islands integration
│   └── 🏝️ service_islands/     # Service Islands Architecture (5 layers)
│       ├── 📋 mod.rs           # Service Islands module coordination
│       ├── 🏗️ layer1_infrastructure/     # Generic cache + shared components + Redis Streams
│       │   ├── cache_system_island.rs    # L1/L2 cache + generic strategies + Redis Streams (XADD/XREAD)
│       │   ├── app_state_island.rs       # Unified app state with stream support
│       │   ├── chart_modules_island.rs   # JavaScript bundling
│       │   └── shared_components_island.rs # Template registry + utilities
│       ├── 🌐 layer2_external_services/   # External APIs + cache-first + stream publishing
│       │   └── external_apis_island.rs    # Binance, CoinGecko + cache-first + circuit breaker
│       ├── 📡 layer3_communication/       # WebSocket + data communication + streams
│       │   ├── websocket_service.rs       # Real-time communication
│       │   ├── data_communication.rs      # Database operations + cache
│       │   ├── dashboard_communication.rs # Stream-aware data routing
│       │   └── layer2_adapters/           
│       │       └── market_data_adapter.rs # 🆕 Publishes to Redis Streams after caching
│       ├── 🔍 layer4_observability/       # Health monitoring + metrics + stream status
│       │   └── health_system_island.rs    # Component health + system status + stream metrics
│       └── 💼 layer5_business_logic/      # Business-specific logic
│           ├── dashboard_island.rs         # Market data processing
│           └── crypto_reports_island.rs    # Report management + templates
├── 📁 routes/                  # Axum routes + Service Islands integration
│   ├── � homepage.rs          # Homepage với Crypto Reports Island
│   ├── 💰 crypto_reports.rs    # Business logic routing
│   ├── 📊 dashboard.rs         # Dashboard Island endpoints
│   ├── 🔌 websocket.rs         # WebSocket Layer 3 Communication
│   └── 🏥 system.rs           # Layer 4 Observability endpoints
├── 📁 scripts/                 # Performance testing across Service Islands
│   ├── ⚡ simple_rps_test.sh   # End-to-end RPS benchmark (500+ RPS)
│   ├── 📊 advanced_benchmark.sh # Service Islands performance test
│   └── 🔥 stress_test.sh       # Multi-layer load testing
├── 📁 docs/                    # Service Islands Architecture documentation
│   ├── �️ SERVICE_ISLANDS_ARCHITECTURE.md   # 5-layer architecture guide
│   ├── 🔄 SERVICE_ISLANDS_WORKFLOW.md        # Development workflow
│   ├── 🗂️ GENERIC_CACHE_ARCHITECTURE.md     # Layer separation cache
│   └── � WEBSOCKET_REALTIME_IMPLEMENTATION.md # Layer 3 communication
├── 📁 dashboards/              # Templates với Layer 1 shared components
│   ├── 🏠 home.html            # Homepage template
│   └── � crypto_dashboard/    # Business logic templates
├── 📁 shared_assets/           # Layer 1 shared components
│   ├── 🎨 css/                # Global stylesheets
│   └── ⚙️ js/chart_modules/   # Modular chart components
├── ⚙️ Cargo.toml              # Dependencies (moka, redis, dashmap, rayon)
├── 🐳 Dockerfile              # Container với Service Islands
├── 🚂 railway.json           # Railway deployment config
└── 📋 .env.example           # Environment template với layer configs
```

### Service Islands Code Organization
- **Layer 5 → Layer 1**: Top-down dependency flow
- **Generic Layer 1**: Pure infrastructure, không business knowledge
- **Business Layer 2**: API-specific implementations using generic Layer 1
- **Clear Boundaries**: Mỗi island độc lập, interface rõ ràng
- **Testable Architecture**: Unit test từng layer independently

## � Redis Streams Integration

### Overview
Redis Streams được tích hợp vào Layer 1 Infrastructure và sử dụng qua Layer 3 Communication để tạo real-time data pipeline cho external consumers (Python AI service, analytics, monitoring).

### Architecture Flow
```
┌─────────────────────────────────────────────────────────────┐
│  Layer 2: External APIs (Binance, CoinGecko, CMC)         │
│  • Fetch market data from multiple sources                │
│  • Circuit breaker + fallback logic                       │
└─────────────────┬───────────────────────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────────────────────┐
│  Layer 3: Market Data Adapter                              │
│  • Normalize data format                                   │
│  • Cache with RealTime strategy (10s TTL)                 │
│  • Publish to Redis Streams (market_data_stream)          │
└─────────────────┬───────────────────────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────────────────────┐
│  Layer 1: Cache Manager + Redis Streams                    │
│  • L1 Cache (moka): In-memory, <1ms                       │
│  • L2 Cache (Redis): Distributed, 2-5ms                   │
│  • Redis Streams: XADD publish, auto-trim at 1000 entries │
└─────────────────┬───────────────────────────────────────────┘
                  │
    ┌─────────────┴─────────────┐
    ▼                           ▼
┌─────────────┐         ┌──────────────────┐
│ Web Clients │         │ External         │
│ (REST API)  │         │ Consumers        │
│             │         │ • Python AI      │
│             │         │ • Analytics      │
│             │         │ • Monitoring     │
└─────────────┘         └──────────────────┘
```

### Key Features

#### 1. **Automatic Publishing**
```rust
// Market data automatically published after caching
match cache_manager.publish_to_stream(
    "market_data_stream",
    fields,  // Flattened JSON key-value pairs
    Some(1000)  // Auto-trim at 1000 entries
).await {
    Ok(entry_id) => println!("📤 Published to stream: {}", entry_id),
    Err(e) => println!("⚠️ Stream publish failed (non-critical): {}", e)
}
```

#### 2. **Consumer-Ready Format**
```rust
// JSON data → Flattened stream fields
{
  "btc_price_usd": 50000.0,
  "eth_price_usd": 3000.0
}
↓
[
  ("btc_price_usd", "50000.0"),
  ("eth_price_usd", "3000.0")
]
```

#### 3. **Stream Operations**
```rust
// Read latest N entries (newest first)
let latest = cache_manager.read_stream_latest("market_data_stream", 10).await?;

// Blocking read for new entries only
let new_data = cache_manager.read_stream(
    "market_data_stream",
    "$",        // Only new entries
    100,        // Max count
    Some(5000)  // Block for 5 seconds
).await?;
```

#### 4. **Python Consumer Example**
```python
import redis
import json

r = redis.Redis(host='localhost', port=6379, decode_responses=True)

# Read latest 10 entries
entries = r.xrevrange('market_data_stream', count=10)

for entry_id, fields in entries:
    data = dict(fields)
    print(f"Entry {entry_id}: BTC=${data.get('btc_price_usd')}")

# Blocking read for real-time updates
while True:
    entries = r.xread({'market_data_stream': '$'}, block=5000, count=1)
    for stream, messages in entries:
        for entry_id, fields in messages:
            print(f"New data: {dict(fields)}")
```

### Performance Characteristics
- **📤 Publish Latency**: <1ms (non-blocking XADD)
- **📊 Throughput**: 10,000+ entries/second sustained
- **💾 Memory**: Auto-trim at 1000 entries (~200KB typical)
- **🔄 Consumer Lag**: Sub-second for Python consumers
- **🛡️ Fault Tolerance**: Stream failures don't affect core API

### Monitoring
```bash
# Check stream info
redis-cli XINFO STREAM market_data_stream

# Read latest entry
redis-cli XREVRANGE market_data_stream + - COUNT 1

# Monitor stream length
redis-cli XLEN market_data_stream
```

### Use Cases
1. **Python AI Service**: Real-time market data for ML models
2. **Analytics Pipeline**: Stream data to data warehouse
3. **Monitoring Dashboards**: External monitoring tools
4. **Backup Systems**: Asynchronous data replication
5. **Audit Logging**: Track all market data updates

### Configuration
```env
# Redis connection (default: localhost:6379)
REDIS_URL=redis://localhost:6379

# Stream settings (configured in code)
STREAM_NAME=market_data_stream
MAX_STREAM_LENGTH=1000
```

---

## �🔧 Development & Troubleshooting

### Development Setup
```bash
# Enable debug logging
export DEBUG=1

# Watch for changes (requires cargo-watch)
cargo install cargo-watch
cargo watch -x run

# Run performance benchmarks
./scripts/simple_rps_test.sh       # Quick RPS test (500+ RPS)
./scripts/advanced_benchmark.sh    # Comprehensive benchmark
./scripts/stress_test.sh           # Load testing

# Run tests
cargo test

# Test API integrations
./test-finnhub-integration.sh       # Test US stock indices integration
cargo run --example test_coinmarketcap_fallback  # Test crypto fallback
cargo run --example test_finnhub_integration     # Test Finnhub API

# Check code quality
cargo clippy
cargo fmt
```

### Common Issues & Solutions

#### 🔍 Database Connection Issues
```bash
# Check PostgreSQL connection
psql $DATABASE_URL -c "SELECT version();"

# Verify table exists
psql $DATABASE_URL -c "\dt"
```

#### ⚡ Performance Debugging
- Check cache hit rates in server logs
- Monitor memory usage: `ps aux | grep web-server-report`
- Use `DEBUG=1` for detailed request logging

#### 🔄 Cache Issues + Stampede Protection + Redis Streams
- **L1 Cache**: In-memory cache auto-expires after 5 minutes with get_with() protection
- **L2 Cache**: Redis cache expires after 1 hour với DashMap+Mutex stampede protection
- **Stampede Protection**: DashMap tracks pending requests, prevents multiple API calls
- **🆕 Redis Streams**: Auto-trimming at 1000 entries, no manual cleanup needed
- **Cache Clearing**: Use `/clear-cache` endpoint to clear all tiers (L1+L2)
- **Cache Stats**: Monitor hit rates và stampede metrics via `/health` and `/cache-stats`
- **🆕 Stream Monitoring**: Check stream entry count and consumer lag in `/health`
- **Restart server**: Clears L1 cache, L2 persists, Streams retain last 1000 entries: `pkill web-server-report && cargo run`

#### 🚀 Build Optimization
```bash
# Faster debug builds
export CARGO_PROFILE_DEV_DEBUG=0

# Smaller release builds  
cargo build --release
strip target/release/web-server-report
```

### Monitoring & Metrics
- Health check: `curl http://localhost:8000/health`
- Performance metrics: `curl http://localhost:8000/metrics` 
- **Multi-tier cache stats**: `curl http://localhost:8000/cache-stats`
- **Cache management**: `curl -X POST http://localhost:8000/clear-cache`
- **HTTP Load Testing**: Run benchmark to validate 16,829+ RPS performance
- WebSocket status: Check Redis connection logs
- **L1 cache metrics**: Monitor moka cache hit rate và stampede protection
- **L2 cache status**: Redis health và DashMap request coalescing metrics
- **🆕 Redis Streams metrics**: Monitor stream entry count, publish rate, consumer lag
- **🆕 Stream debugging**: `redis-cli XINFO STREAM market_data_stream` to inspect stream details
- Response times: Enable `DEBUG=1` for timing logs với stampede tracking

## 🎯 Recent Updates

### ✅ Redis Streams Integration (Latest - October 2025)
- **📤 Native Stream Publishing**: Market data automatically published to `market_data_stream`
- **🔄 Python AI Service Integration**: External consumers can read real-time market data
- **⚡ Sub-millisecond Overhead**: Stream publishing adds <1ms latency
- **🛡️ Non-blocking Architecture**: Stream failures don't affect core functionality
- **📊 Auto-trimming**: Streams maintain last 1000 entries automatically
- **🎯 Consumer-ready Format**: Flattened JSON fields optimized for XREAD consumers

### ✅ Layer Architecture Refactoring (October 2025)
- **🏝️ Layer 1 Enhancements**: Cache Manager with Redis Streams methods
- **🌐 Layer 2 Improvements**: External APIs Island with stream publishing
- **📡 Layer 3 Upgrades**: Market Data Adapter publishes to streams after caching
- **🔧 Modular Design**: Each layer clearly separated with well-defined interfaces

### ✅ Cache Stampede Protection Implementation
- **🛡️ DashMap+Mutex Request Coalescing**: Prevents multiple concurrent API calls for same cache key
- **⚡ Moka get_with() L1 Protection**: Built-in stampede protection for in-memory cache
- **📈 99.6% Performance Improvement**: From 534ms → 5.2ms average latency in high-concurrency
- **🚀 16,829+ RPS Peak Performance**: Sustained high throughput with stampede protection
- **🔄 Request Deduplication**: Single API call per cache expiration across all concurrent requests

### 🔧 HTTP Endpoint Optimizations  
- **Route-level Optimization**: Removed unnecessary `fetch_realtime_market_data()` calls
- **Template Caching**: Cached HTML templates with business logic separation
- **Background Processing**: spawn_blocking for CPU-intensive operations
- **Connection Pooling**: Optimized PostgreSQL pool for 16-core systems

### 📊 Comprehensive Load Testing
- **Multi-scenario Benchmarking**: Gradual ramp-up, sustained load, burst scenarios
- **100% Success Rate**: Zero failures across 16,829+ requests per second
- **Atomic Counters**: Thread-safe performance metrics collection
- **Real-world Testing**: Production-like concurrent load validation

---

🎉 **Ready for Production**: High-performance crypto dashboard with enterprise-grade caching and stampede protection!

## 🤝 Contributing

### Code Style
- Use `cargo fmt` for formatting
- Follow Rust naming conventions
- Add documentation for public functions
- Include error handling with proper logging

### Adding New Features
1. Fork the repository
2. Create feature branch: `git checkout -b feature/new-feature`
3. Add tests for new functionality
4. Submit pull request with description

### Performance Guidelines
- Prefer `tokio::join!` for concurrent operations
- Use `DashMap` for thread-safe shared state with high concurrency
- Use `AtomicUsize` for lock-free counters and metrics
- Cache expensive operations (DB queries, template rendering)
- Use `spawn_blocking` for CPU-intensive background tasks
- Add appropriate HTTP cache headers
- Benchmark with `./scripts/simple_rps_test.sh` after changes

## 📜 License & Support

**License**: Apache License 2.0 - see LICENSE file for details

**Support**:
- 🐛 Bug reports: [Create GitHub Issue](https://github.com/thichuong/Web-server-Report/issues)
- 💡 Feature requests: [GitHub Discussions](https://github.com/thichuong/Web-server-Report/discussions)
- 📧 Contact: [Your Email]

**Related Projects**:
- 🤖 [Crypto-Dashboard-and-AI-ReportGenerator](https://github.com/thichuong/Crypto-Dashboard-and-AI-ReportGenerator) - Admin UI & AI report generation

---

⭐ **Star this repo** if it helps you build better crypto dashboards! 

Built with ❤️ using Rust 🦀

