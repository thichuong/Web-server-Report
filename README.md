# Web Server Report - High-Performance Crypto Dashboard (Main Service)

🚀 **Ultra-fast Rust web server** achieving **16,829+ RPS** with **5.2ms latency** for crypto investment reports with advanced multi-threading, Cache Stampede Protection, and microservices architecture.

> **🏗️ Microservices Architecture**: This is the **Main Service** that handles web presentation and data consumption. External APIs and WebSocket functionality are handled by the separate [Web-server-Report-websocket](https://github.com/thichuong/Web-server-Report-websocket) service.

## ✨ Key Features

### 🎯 Core Functionality
- **Interactive Crypto Reports**: Dynamic investment reports with Chart.js visualizations
- **Multi-language Support**: Vietnamese/English with seamless switching
- **Responsive Design**: Mobile-first, adaptive UI
- **Redis Streams Integration**: Consume real-time data from websocket service
- **Microservices Architecture**: Main service focuses on presentation, websocket service handles external APIs

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
- **Microservices Separation**: Main service isolated from external API failures
- **Redis Streams Consumer**: Reliable data consumption from websocket service
- **Data Validation**: Prevents corrupted data from affecting reports
- **Cache-first Strategy**: Read from cache and Redis Streams, minimal external dependencies
- **Service Health Monitoring**: Independent health checks for each service

### 🔧 Technical Stack
- **Backend**: Rust + Axum (high-performance async web framework)
- **Database**: PostgreSQL with optimized connection pooling (32 max connections)
- **Caching**: Multi-tier L1 (moka) + L2 (Redis) with Cache Stampede Protection
- **Data Source**: Redis Streams (consumes data from websocket service)
- **Concurrency**: Rayon ThreadPool + tokio async runtime + DashMap request coalescing
- **Inter-service Communication**: Redis Streams for real-time data from websocket service
- **Templates**: Tera template engine with background rendering
- **Frontend**: Vanilla JS with Chart.js and modern CSS
- **Architecture**: Microservices - Main service (presentation) + Websocket service (external APIs)

## 🚀 Quick Start

### Prerequisites
- Rust 1.70+ ([Install Rust](https://rustup.rs/))
- PostgreSQL database
- Redis server (required for cache and inter-service communication)
- **Web-server-Report-websocket** service running (provides market data via Redis Streams)

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

# Redis for cache and inter-service communication (REQUIRED)
REDIS_URL=redis://localhost:6379

# Server configuration
HOST=0.0.0.0
PORT=8000

# Development mode (enables debug logging)
DEBUG=1

# Note: External API keys (TAAPI, CMC, Finnhub) are configured in the websocket service
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

### 🏗️ Microservices Architecture

This project uses a **microservices architecture** with clear separation of concerns:

#### **Main Service** (Web-server-Report) - This repository
- **Purpose**: Web presentation, report generation, data consumption
- **Responsibilities**:
  - Serve HTTP endpoints for crypto reports and dashboard
  - Render HTML templates with Tera
  - Read data from Redis Streams (published by websocket service)
  - Cache data in L1 (Moka) and L2 (Redis)
  - Database operations (PostgreSQL)
- **Does NOT handle**: External API calls, WebSocket connections

#### **Websocket Service** (Web-server-Report-websocket) - Separate service
- **Purpose**: External API integration, WebSocket broadcasting, data publishing
- **Responsibilities**:
  - Fetch data from external APIs (Binance, CoinGecko, CoinMarketCap, etc.)
  - Manage WebSocket connections for real-time updates
  - Publish market data to Redis Streams
  - Circuit breaker and API fallback logic
- **Repository**: [Web-server-Report-websocket](../Web-server-Report-websocket)

### 🔄 Inter-service Communication

```
┌─────────────────────────────────────────────────────────────┐
│         Web-server-Report-websocket (Separate Service)     │
│                                                             │
│  Layer 2: External APIs                                    │
│  • Binance, CoinGecko, CoinMarketCap                       │
│  • TAAPI.io, Finnhub (US stocks)                           │
│  • Circuit breaker + fallback logic                        │
│                                                             │
│  Layer 3: WebSocket + Market Data Adapter                  │
│  • WebSocket broadcasting                                  │
│  • Publish to Redis Streams                                │
└─────────────────────┬───────────────────────────────────────┘
                      │
                      │ Redis Streams (market_data_stream)
                      ▼
┌─────────────────────────────────────────────────────────────┐
│         Web-server-Report (Main Service - This Repo)       │
│                                                             │
│  Layer 3: Communication                                    │
│  • Redis Stream Reader (consumes data)                     │
│  • Data Communication (PostgreSQL)                         │
│  • Dashboard Communication                                 │
│                                                             │
│  Layer 5: Business Logic                                   │
│  • Crypto Reports (template rendering)                     │
│  • Dashboard (data aggregation)                            │
└─────────────────────────────────────────────────────────────┘
```

### Service Islands Architecture (Main Service)

The **Main Service** uses a **simplified 4-layer Service Islands Architecture** focused on presentation and data consumption:

```
┌─────────────────────────────────────────────────────────────┐
│                    Layer 5: Business Logic                 │
│  ┌─────────────────┐    ┌─────────────────────────────────┐│
│  │  Dashboard      │    │     Crypto Reports              ││
│  │  Island         │    │     Island                      ││
│  │ • Market Data   │    │ • Report Management             ││
│  │   Aggregation   │    │ • Template Orchestration        ││
│  │ • Data          │    │ • Cache Integration             ││
│  │   Processing    │    │                                 ││
│  └─────────────────┘    └─────────────────────────────────┘│
└─────────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────────┐
│                   Layer 4: Observability                   │
│  ┌─────────────────────────────────────────────────────────┐│
│  │              Health System Island                      ││
│  │ • Component Health Monitoring                          ││
│  │ • System Status Reporting                              ││
│  │ • Inter-service Health Validation                      ││
│  └─────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────────┐
│                   Layer 3: Communication                   │
│  ┌──────────────────┐  ┌──────────────────┐  ┌────────────┐│
│  │ Redis Stream     │  │ Data             │  │ Dashboard  ││
│  │ Reader           │  │ Communication    │  │ Comm       ││
│  │ • Consume from   │  │ • PostgreSQL Ops │  │ • Data     ││
│  │   websocket svc  │  │ • Cache Integ    │  │   Routing  ││
│  │ • Real-time data │  │ • DB Models      │  │            ││
│  └──────────────────┘  └──────────────────┘  └────────────┘│
└─────────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────────┐
│                  Layer 1: Infrastructure                   │
│  ┌─────────────────┐  ┌──────────────────┐  ┌─────────────┐│
│  │  Shared         │  │  Cache System    │  │ App State   ││
│  │  Components     │  │  Island          │  │ Island      ││
│  │ • Template      │  │ • L1 (Moka)      │  │ • DB Pool   ││
│  │   Registry      │  │ • L2 (Redis)     │  │ • Redis     ││
│  │ • Chart Modules │  │ • Stampede Prot  │  │ • Templates ││
│  │ • Utilities     │  │ • Strategies     │  │             ││
│  └─────────────────┘  └──────────────────┘  └─────────────┘│
└─────────────────────────────────────────────────────────────┘

Note: Layer 2 (External Services) and WebSocket are in separate
      Web-server-Report-websocket service
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

### Request Flow (Main Service)
```
Client Request ───► Axum Router (Main Service)
                           │
                           ▼
              ┌─────────────────────────┐
              │   Layer 5: Business     │ ──► Template Rendering
              │   • Dashboard Island    │     Report Processing
              │   • Crypto Reports      │
              └─────────┬───────────────┘
                        │ Data Needed
                        ▼
              ┌─────────────────────────┐
              │   Layer 3: Comm         │ ──► PostgreSQL
              │   • Redis Stream Reader │     Read from Cache
              │   • Data Communication  │     (L1 + L2)
              │   • Dashboard Comm      │
              └─────────┬───────────────┘
                        │ Cache Lookup / Stream Read
                        ▼
              ┌─────────────────────────┐
              │   Layer 1: Cache       │ ──► L1 (moka) ⚡<1ms
              │   • Generic Strategies │     L2 (Redis) 🔥2-5ms
              │   • Unified Manager    │     Redis Streams (read)
              └─────────────────────────┘
                        ▲
                        │ Data published by websocket service
                        │
              ┌─────────────────────────┐
              │ Web-server-Report-      │ ──► External APIs
              │ websocket (Separate)    │     (Binance, CoinGecko, etc.)
              │ • Fetch from APIs       │     WebSocket broadcasting
              │ • Publish to Streams    │     Circuit breaker
              └─────────────────────────┘
```

### Service Islands Performance Metrics (Main Service)

#### Redis Streams Consumer Performance
- **📥 Read Latency**: <2ms average (XREAD from websocket service)
- **📊 Consumer Throughput**: Process 10,000+ entries/sec
- **🔄 Stream Lag**: Sub-second consumption from websocket service
- **💾 Reliability**: Automatic reconnection and error recovery

#### Cache Performance (Layer 1 Infrastructure) - **with Cache Stampede Protection**
- **L1 Hit Rate**: ~90% (sub-millisecond response)
- **L2 Hit Rate**: ~75% (2-5ms with automatic L1 promotion)
- **Stampede Protection**: 99.6% improvement in high-concurrency scenarios
- **Request Coalescing**: DashMap+Mutex for L2, Moka's get_with() for L1
- **Overall Coverage**: ~95% (minimal external service dependencies)
- **Generic Strategies**: ShortTerm(5min), MediumTerm(1hr), LongTerm(3hr), RealTime(30s)

#### Business Logic Performance (Layer 5)
- **Dashboard Island**: Data aggregation from cache and streams
- **Crypto Reports Island**: Template orchestration with multi-tier caching
- **Report Generation**: Background processing with spawn_blocking

#### Communication Layer Performance (Layer 3)
- **Redis Stream Reader**: Consume real-time data from websocket service
- **Data Communication**: PostgreSQL connection pool (32 max connections)
- **Cache Integration**: L2 cache for database queries

#### Infrastructure Performance (Layer 1)
- **🚄 16,829+ RPS**: Handle 16,829+ concurrent requests per second with Cache Stampede Protection
- **⚡ Sub-1ms L1 Cache**: Moka in-memory cache hits with get_with() coalescing
- **🔥 2-5ms L2 Cache**: Redis distributed cache with DashMap+Mutex request coalescing
- **🛡️ 99.6% Stampede Protection**: Prevents cache stampede in high-concurrency scenarios
- **🔄 Multi-threaded**: Rayon ThreadPool + tokio async runtime
- **📊 95% Cache Coverage**: Minimal dependency on external services
- **🏗️ Microservices**: Clean separation between main and websocket services

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

### Service Islands Request Flow (Main Service)
1. **Client Request** → Axum Router → Layer 5 Business Logic
2. **Dashboard Island** → Data aggregation → Layer 3 Communication
3. **Layer 3 Communication** → Check L1/L2 cache → Read from Redis Streams
4. **Cache System Island** → Generic cache strategies (L1: <1ms, L2: 2-5ms)
5. **Redis Stream Reader** → Consume data published by websocket service
6. **Response** → Template rendering → Client delivery

#### Data Flow Between Services
```
Websocket Service (External Service)
        ↓
External APIs (Binance/CoinGecko/CMC)
        ↓
Market Data Adapter (Websocket Service)
        ↓
Redis Streams Publishing (XADD)
        ↓
Main Service (This Repo)
        ↓
Redis Stream Reader (Layer 3)
        ↓
Cache Manager (L1 + L2 caching)
        ↓
Business Logic (Dashboard, Reports)
        ↓
Client (Web Browser)
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

## 📡 API Reference (Main Service)

### Core Endpoints
| Method | Endpoint | Description | Performance |
|--------|----------|-------------|-------------|
| `GET` | `/` | Homepage with latest report | 16,829+ RPS |
| `GET` | `/health` | Server health check + metrics | - |
| `GET` | `/metrics` | Performance metrics | - |
| `GET` | `/crypto_report` | Latest crypto report | 16,829+ RPS |
| `GET` | `/crypto_report/:id` | Specific report by ID | 16,829+ RPS Stampede Protected |
| `GET` | `/crypto_reports_list` | Paginated report list | - |

### Admin & Monitoring
| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/health` | Main service health + cache metrics + Redis Streams reader status |
| `GET` | `/cache-stats` | Detailed L1/L2 cache statistics |
| `POST` | `/clear-cache` | Clear all cache tiers (L1+L2) |

### Data API
| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/api/crypto/dashboard-summary` | Cached dashboard data (from cache or Redis Streams) |
| `GET` | `/api/crypto/dashboard-summary/refresh` | Force cache refresh and re-read from streams |

### Static Assets
| Path | Description |
|------|-------------|
| `/shared_assets/js/chart_modules.js` | Bundled chart JavaScript |
| `/shared_assets/css/` | Stylesheets |
| `/crypto_dashboard/assets/` | Dashboard-specific assets |

> **Note**: WebSocket endpoint (`/ws`) is served by the separate **Web-server-Report-websocket** service (port 3001)

## 🗂️ Service Islands Cache System (Main Service)

Main service implements **Generic Cache Architecture** focused on data consumption and caching:

### Layer 1: Infrastructure (Generic Cache + Redis Streams Reader)
- **L1 Cache**: `moka::future::Cache` - Ultra-fast in-memory (2000 entries, 5min TTL)
- **L2 Cache**: Redis - Distributed cache with persistence (1hr default TTL)
- **Redis Streams Reader**: Consume data from websocket service
  - `read_stream_latest()`: Retrieve latest N entries from websocket service
  - `read_stream()`: Blocking/non-blocking stream consumption
- **Generic Strategies**: ShortTerm, MediumTerm, LongTerm, RealTime, Custom
- **Unified API**: Pure caching infrastructure, no external API knowledge

### Layer 3: Communication (Data Consumption)
- **Redis Stream Reader**: Consume market data published by websocket service
- **Data Communication**: PostgreSQL operations with cache integration
- **Dashboard Communication**: Data routing from cache and streams

### Cache Architecture Benefits
- **Separation of Concerns**: Layer 1 pure caching, Layer 3 data consumption
- **Microservices**: Main service isolated from external API complexities
- **Maintainability**: Clear boundaries between services
- **Testability**: Each layer independently testable
- **Real-time Pipeline**: Redis Streams for inter-service communication

### Cache Usage Patterns (Main Service)

#### 1. **Consume Data from Websocket Service**
```rust
// Read latest market data from Redis Streams
let stream_data = redis_stream_reader
    .read_stream_latest("market_data_stream", 10)
    .await?;

// Cache the consumed data for fast access
cache_manager.cache_data("dashboard_summary", stream_data, ttl).await?;
```

#### 2. **Cache-First Data Access**
```rust
// Try L1 cache first
if let Some(data) = cache_manager.get_l1("btc_price").await? {
    return Ok(data);
}

// Try L2 cache second
if let Some(data) = cache_manager.get_l2("btc_price").await? {
    // Promote to L1
    cache_manager.set_l1("btc_price", data.clone()).await?;
    return Ok(data);
}

// Read from Redis Streams as fallback
let data = redis_stream_reader.read_stream_latest("market_data_stream", 1).await?;
cache_manager.set_with_strategy("btc_price", data, CacheStrategy::ShortTerm).await?;
```

#### 3. **Business Logic Integration**
```rust
// Dashboard Island - aggregate data from cache and streams
DashboardIsland → Check L1/L2 cache → Read from Redis Streams → Cache result

// Crypto Reports Island - template rendering with cached data
CryptoReportsIsland → Fetch from cache → Render with Tera → Return HTML
```

### Cache Monitoring (Main Service)
- **Health**: `/health` endpoint shows L1/L2 status, hit rates, and Redis Streams reader health
- **Statistics**: `/cache-stats` provides detailed cache metrics
- **Management**: `/clear-cache` clears all cache tiers (L1+L2)
- **Performance**: 95% cache coverage, <1ms L1 hits, 2-5ms L2 hits
- **Stream Metrics**: Track consumer lag, read throughput from websocket service

📖 **Detailed Documentation**: See service-specific documentation in [docs/](./docs) folder.

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

## 🏗️ Project Structure (Main Service)

```
Web-server-Report/ (Main Service - Port 8000)
├── 📁 src/
│   ├── 🦀 main.rs              # Server initialization + Service Islands setup
│   ├── 📊 performance.rs       # Performance monitoring
│   ├── 🏗️ state.rs             # Application state
│   └── 🏝️ service_islands/     # Service Islands Architecture (4 layers)
│       ├── 📋 mod.rs           # Service Islands coordination
│       ├── 🏗️ layer1_infrastructure/     # Generic cache + shared components
│       │   ├── cache_system_island/     # L1/L2 cache + Redis Streams reader
│       │   ├── app_state_island/        # Unified app state
│       │   ├── chart_modules_island/    # JavaScript bundling
│       │   └── shared_components_island/ # Template registry + utilities
│       ├── 📡 layer3_communication/      # Data communication (NO WebSocket)
│       │   ├── redis_stream_reader/     # 🔥 Consume from websocket service
│       │   ├── data_communication/      # Database operations + cache
│       │   └── dashboard_communication/ # Data routing from cache/streams
│       ├── 🔍 layer4_observability/      # Health monitoring
│       │   └── health_system/           # Component health + system status
│       └── 💼 layer5_business_logic/     # Business logic
│           ├── dashboard/               # Data aggregation (from cache/streams)
│           └── crypto_reports/          # Report management + templates
├── 📁 routes/                  # Axum routes
│   ├── 🏠 homepage.rs          # Homepage
│   ├── 💰 crypto_reports.rs    # Crypto reports endpoints
│   ├── 📊 api.rs               # API endpoints (dashboard data)
│   ├── 🗂️ static_files.rs     # Static assets serving
│   └── 🏥 system.rs            # Health & monitoring endpoints
├── 📁 scripts/                 # Performance testing
│   ├── ⚡ simple_rps_test.sh   # RPS benchmark
│   └── 🔥 stress_test.sh       # Load testing
├── 📁 docs/                    # Documentation
│   └── (Main service specific docs)
├── 📁 dashboards/              # HTML templates
│   ├── 🏠 home.html            # Homepage template
│   └── 💹 crypto_dashboard/    # Dashboard templates
├── 📁 shared_assets/           # Static assets
│   ├── 🎨 css/                 # Stylesheets
│   └── ⚙️ js/chart_modules/    # Chart components
├── ⚙️ Cargo.toml               # Dependencies (NO tonic/prost)
├── 🐳 Dockerfile               # Container config
└── 📋 .env.example             # Environment template

Note: Layer 2 (External APIs) and WebSocket moved to:
      ../Web-server-Report-websocket/ (Separate service - Port 3001)
```

### Service Islands Code Organization (Main Service)
- **4-Layer Architecture**: Layer 1 (Infrastructure), Layer 3 (Communication), Layer 4 (Observability), Layer 5 (Business)
- **No Layer 2**: External APIs handled by websocket service
- **Redis Streams Consumer**: Layer 3 consumes data published by websocket service
- **Clear Boundaries**: Each layer independent, well-defined interfaces
- **Testable Architecture**: Unit test each layer independently
- **Microservices**: Main service focuses on presentation, websocket service handles external APIs

## 📡 Redis Streams Integration (Main Service)

### Overview
Main service consumes market data from Redis Streams published by the **websocket service**. This enables real-time data access without direct external API dependencies.

### Architecture Flow (Consumer Role)
```
┌─────────────────────────────────────────────────────────────┐
│  Websocket Service (Separate - Port 3001)                  │
│  • Fetch from External APIs (Binance, CoinGecko, etc.)    │
│  • Publish to Redis Streams (market_data_stream)          │
│  • XADD with auto-trim at 1000 entries                    │
└─────────────────┬───────────────────────────────────────────┘
                  │ Redis Streams
                  │ (market_data_stream)
                  ▼
┌─────────────────────────────────────────────────────────────┐
│  Main Service (This Repo - Port 8000)                     │
│                                                             │
│  Layer 3: Redis Stream Reader                              │
│  • XREAD / XREVRANGE to consume data                      │
│  • Read latest entries for dashboard                       │
│  • Monitor consumer lag                                    │
│                                                             │
│  Layer 1: Cache Manager                                    │
│  • L1 Cache (moka): In-memory, <1ms                       │
│  • L2 Cache (Redis): Distributed, 2-5ms                   │
│  • Cache consumed stream data                              │
│                                                             │
│  Layer 5: Business Logic                                   │
│  • Dashboard: Aggregate cached + stream data               │
│  • Reports: Render templates with data                     │
└─────────────────┬───────────────────────────────────────────┘
                  │
                  ▼
            Web Clients (Browser)
```

### Key Features (Consumer Side)

#### 1. **Consuming from Streams**
```rust
// Main service reads data published by websocket service
let stream_data = redis_stream_reader
    .read_stream_latest("market_data_stream", 10)
    .await?;

// Cache the consumed data
cache_manager.cache_data("dashboard_summary", stream_data, ttl).await?;
```

#### 2. **Data Format**
```rust
// Websocket service publishes flattened fields
// Main service consumes and reconstructs JSON
[
  ("btc_price_usd", "50000.0"),
  ("eth_price_usd", "3000.0"),
  ("updated_at", "2025-01-15T10:30:00Z")
]
↓ (Main service reconstructs)
{
  "btc_price_usd": 50000.0,
  "eth_price_usd": 3000.0,
  "updated_at": "2025-01-15T10:30:00Z"
}
```

#### 3. **Stream Reader Operations**
```rust
// Read latest N entries (newest first)
let latest = redis_stream_reader
    .read_stream_latest("market_data_stream", 10)
    .await?;

// Blocking read for new entries (if needed)
let new_data = redis_stream_reader
    .read_stream("market_data_stream", "$", 100, Some(5000))
    .await?;
```

### Performance Characteristics (Consumer)
- **📥 Read Latency**: <2ms (XREAD/XREVRANGE)
- **📊 Throughput**: Process 10,000+ entries/second
- **🔄 Consumer Lag**: Sub-second from websocket service
- **💾 Cache Integration**: Stream data cached in L1+L2
- **🛡️ Fault Tolerance**: Fallback to cache if stream unavailable

### Monitoring (Main Service)
```bash
# Check consumer health
curl http://localhost:8000/health

# Verify stream reader status
curl http://localhost:8000/cache-stats

# Monitor Redis Streams (from Redis CLI)
redis-cli XINFO STREAM market_data_stream
redis-cli XREVRANGE market_data_stream + - COUNT 1
```

### Configuration
```env
# Redis connection (REQUIRED for stream consumer)
REDIS_URL=redis://localhost:6379

# Websocket service should be running on port 3001
# and publishing to 'market_data_stream'
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

### ✅ Microservices Architecture Refactoring (Latest - January 2025)
- **🏗️ Service Separation**: Split into Main Service (this repo) and Websocket Service
- **📥 Redis Streams Consumer**: Main service consumes data from websocket service
- **🔧 Layer Simplification**: 4-layer architecture (removed Layer 2 from main service)
- **🎯 Clear Responsibilities**: Main service = presentation, Websocket service = external APIs
- **🚀 Independent Deployment**: Each service can be deployed and scaled independently
- **📡 Inter-service Communication**: Redis Streams for real-time data pipeline

### ✅ Main Service Optimization (January 2025)
- **🏝️ Layer 1 (Infrastructure)**: Cache Manager with Redis Streams reader
- **📡 Layer 3 (Communication)**: Redis Stream Reader, Data Communication, Dashboard Communication
- **🔍 Layer 4 (Observability)**: Health monitoring for main service components
- **💼 Layer 5 (Business Logic)**: Dashboard aggregation and Crypto Reports rendering
- **🔧 No External APIs**: All API calls handled by websocket service

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

**Related Services** (Microservices Architecture):
- 🔌 **[Web-server-Report-websocket](../Web-server-Report-websocket)** - Websocket service handling external APIs (Binance, CoinGecko, etc.) and WebSocket connections
- 🤖 **[Crypto-Dashboard-and-AI-ReportGenerator](https://github.com/thichuong/Crypto-Dashboard-and-AI-ReportGenerator)** - Admin UI & AI report generation

---

⭐ **Star this repo** if it helps you build better crypto dashboards!

Built with ❤️ using Rust 🦀 | **Microservices Architecture**

