# Web Server Report - High-Performance Crypto Dashboard

🚀 **Ultra-fast Rust web server** achieving **500+ RPS** with **2ms latency** for crypto investment reports with advanced multi-threading and real-time features.

## ✨ Key Features

### 🎯 Core Functionality
- **Interactive Crypto Reports**: Dynamic investment reports with Chart.js visualizations
- **Multi-language Support**: Vietnamese/English with seamless switching
- **Responsive Design**: Mobile-first, adaptive UI
- **PDF Generation**: Export reports to PDF format
- **Real-time Updates**: WebSocket integration for live data

### ⚡ Performance Optimizations
- **Multi-tier Cache System**: L1 (In-Memory) + L2 (Redis) with automatic promotion
- **Multi-threaded Architecture**: Thread-safe operations with concurrent processing  
- **Concurrent Request Processing**: Handle 500+ RPS with 2ms average latency
- **Lock-free Operations**: Atomic counters and non-blocking data structures
- **Parallel CPU Tasks**: Background template rendering with spawn_blocking
- **Unified Cache Manager**: Single API for all caching operations
- **Database Connection Pool**: Optimized for 16-core systems (32 max connections)
- **Chart Module Bundling**: Optimized JavaScript asset delivery

### 🔧 Technical Stack
- **Backend**: Rust + Axum (high-performance async web framework)
- **Database**: PostgreSQL with optimized connection pooling (32 max connections)
- **Caching**: Multi-tier L1 (moka) + L2 (Redis) with unified CacheManager
- **Concurrency**: Rayon ThreadPool + tokio async runtime
- **Real-time**: Redis + WebSocket for live updates
- **Templates**: Tera template engine with background rendering
- **Frontend**: Vanilla JS with Chart.js and modern CSS

## 🚀 Quick Start

### Prerequisites
- Rust 1.70+ ([Install Rust](https://rustup.rs/))
- PostgreSQL database
- Redis server (optional, for WebSocket features)

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
│  │ • Market Data API (CoinGecko, TaApi.io)               ││
│  │ • Rate Limiter with Exponential Backoff               ││
│  │ • API Aggregator (Multi-source data)                  ││
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
              │   Layer 2: External    │ ──► CoinGecko API
              │   • APIs Island        │     TaApi.io API
              │   • Rate Limiter       │     Circuit Breaker
              └─────────┬───────────────┘
                        │ Cache Integration
                        ▼
              ┌─────────────────────────┐
              │   Layer 1: Cache       │ ──► L1 (moka) ⚡<1ms
              │   • Generic Strategies │     L2 (Redis) 🔥2-5ms
              │   • Unified Manager    │     Cache Miss 💻200ms+
              └─────────────────────────┘
```

### Service Islands Performance Metrics

#### Cache Performance (Layer 1 Infrastructure)
- **L1 Hit Rate**: ~90% (sub-millisecond response)
- **L2 Hit Rate**: ~75% (2-5ms with automatic L1 promotion)
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
- **Rate Limiter**: Exponential backoff cho API protection
- **Circuit Breaker**: Fault tolerance cho external APIs
- **API Aggregator**: Multi-source data với intelligent failover

#### Infrastructure Performance (Layer 1)
- **🚄 500+ RPS**: Handle 500+ concurrent requests per second
- **⚡ Sub-1ms L1 Cache**: Moka in-memory cache hits
- **🔥 2-5ms L2 Cache**: Redis distributed cache với automatic promotion
- **🔄 Multi-threaded**: Rayon ThreadPool + tokio async runtime
- **📊 95% Cache Coverage**: Generic cache strategies reduce API calls
- **🏗️ Service Islands**: Clean separation of concerns across 5 layers

### Benchmark Results
```
📊 Performance Test Results (16 CPU cores):

Light Load:   50 RPS  | 20ms avg latency
Medium Load: 200 RPS  |  5ms avg latency  
Heavy Load:  500 RPS  |  2ms avg latency
Extreme:     500 RPS  |  2ms avg latency

Multi-tier Cache Performance:
• L1 Cache Hit:    <1ms (90% hit rate)
• L2 Cache Hit:  2-5ms (75% hit rate, promotes to L1)  
• Cache Miss:   200ms+ (fresh API fetch + dual cache)
• Overall Coverage: 95% (drastically reduced API calls)
```

### Service Islands Request Flow
1. **Client Request** → Axum Router → Layer 5 Business Logic
2. **Dashboard Island** → Market data processing → Layer 3 Communication
3. **Data Communication** → PostgreSQL/Cache lookup → Layer 2 External Services
4. **External APIs Island** → Rate-limited API calls → Layer 1 Infrastructure  
5. **Cache System Island** → Generic cache strategies (L1: <1ms, L2: 2-5ms)
6. **Response** → Multi-tier cache storage → Client delivery

#### Cache Strategy Mapping
- **BTC Price**: `ShortTerm` strategy (5min TTL) - Fast-changing data
- **Technical Indicators**: `LongTerm` strategy (3hr TTL) - RSI, MACD
- **Fear & Greed**: `RealTime` strategy (30s TTL) - Market sentiment
- **Global Data**: `MediumTerm` strategy (1hr TTL) - Market stats

## 📡 API Reference

### Core Endpoints
| Method | Endpoint | Description | Performance |
|--------|----------|-------------|-------------|
| `GET` | `/` | Homepage with latest report | 500+ RPS |
| `GET` | `/health` | Server health check + metrics | - |
| `GET` | `/metrics` | Performance metrics | - |
| `GET` | `/crypto_report` | Latest crypto report | 500+ RPS |
| `GET` | `/crypto_report/:id` | Specific report by ID | 500+ RPS |
| `GET` | `/pdf-template/:id` | PDF-optimized report view | ✅ Cached |
| `GET` | `/crypto_reports_list` | Paginated report list | - |

### Admin & Monitoring
| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/health` | Server health + unified cache metrics |
| `GET` | `/cache-stats` | Detailed L1/L2 cache statistics |
| `POST` | `/clear-cache` | Clear all cache tiers (L1+L2) |

### Real-time & API
| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/ws` | WebSocket connection for real-time updates |
| `GET` | `/api/crypto/dashboard-summary` | Cached dashboard data (JSON) |
| `GET` | `/api/crypto/dashboard-summary/refresh` | Force refresh dashboard |

### Static Assets
| Path | Description |
|------|-------------|
| `/shared_assets/js/chart_modules.js` | Bundled chart JavaScript |
| `/shared_assets/css/` | Stylesheets |
| `/crypto_dashboard/assets/` | Dashboard-specific assets |

## 🗂️ Service Islands Cache System

Hệ thống implement **Generic Cache Architecture** với Layer Separation để tách biệt business logic khỏi cache infrastructure:

### Layer 1: Infrastructure (Generic Cache)
- **L1 Cache**: `moka::future::Cache` - Ultra-fast in-memory (2000 entries, 5min TTL)
- **L2 Cache**: Redis - Distributed cache with persistence (1hr default TTL)
- **Generic Strategies**: ShortTerm, MediumTerm, LongTerm, RealTime, Custom
- **Unified API**: Pure caching infrastructure, không business knowledge

### Layer 2: Business Logic (API-Specific)
- **Business Wrappers**: API-specific implementations using generic Layer 1
- **Strategy Mapping**: Business needs mapped to generic cache strategies
- **Cache Keys**: Business-aware cache key generation

### Cache Architecture Benefits
- **Separation of Concerns**: Layer 1 pure caching, Layer 2 business logic
- **Extensibility**: Add new APIs chỉ cần thay đổi Layer 2
- **Maintainability**: Không hardcoded business keys trong Layer 1
- **Testability**: Layer 1 unit test độc lập, Layer 2 business logic isolated

### Cache Usage Patterns

#### 1. **Generic Cache Helper (Layer 2)**
```rust
async fn cache_api_data<F, T>(
    cache_key: &str,
    strategy: CacheStrategy,  // Generic strategy
    fetch_fn: F
) -> Result<Value>
```

#### 2. **Business-Specific Wrappers (Layer 2)**
```rust
fetch_btc_price() → cache_api_data("btc_coingecko", ShortTerm, api_call)
fetch_rsi_data() → cache_api_data("rsi_taapi", LongTerm, api_call)
fetch_fear_greed() → cache_api_data("fear_greed", RealTime, api_call)
```

#### 3. **WebSocket Broadcasting (Layer 3)**
```rust
WebSocketService → Redis pub/sub → Real-time updates
```

### Cache Monitoring
- **Health**: `/health` endpoint shows L1/L2 status and hit rates
- **Statistics**: `/cache-stats` provides detailed cache metrics  
- **Management**: `/clear-cache` clears all cache tiers
- **Performance**: 95% cache coverage, <1ms L1 hits, 2-5ms L2 hits

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
│       ├── 🏗️ layer1_infrastructure/     # Generic cache + shared components
│       │   ├── cache_system_island.rs    # L1/L2 cache với generic strategies
│       │   └── shared_components_island.rs # Template registry + utilities
│       ├── 🌐 layer2_external_services/   # External APIs + rate limiting
│       │   └── external_apis_island.rs    # CoinGecko, TaApi.io + circuit breaker
│       ├── 📡 layer3_communication/       # WebSocket + data communication
│       │   ├── websocket_service.rs       # Real-time communication
│       │   └── data_communication.rs      # Database operations + cache
│       ├── 🔍 layer4_observability/       # Health monitoring + metrics
│       │   └── health_system_island.rs    # Component health + system status
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

## 🔧 Development & Troubleshooting

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

#### 🔄 Cache Issues
- **L1 Cache**: In-memory cache auto-expires after 5 minutes (TTL)
- **L2 Cache**: Redis cache expires after 1 hour, shared across instances
- **Cache Clearing**: Use `/clear-cache` endpoint to clear all tiers
- **Cache Stats**: Monitor hit rates via `/health` and `/cache-stats` endpoints
- **Restart server**: Clears L1 cache, L2 persists: `pkill web-server-report && cargo run`

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
- RPS benchmarks: Run `./scripts/simple_rps_test.sh`
- WebSocket status: Check Redis connection logs
- **L1 cache metrics**: Monitor moka cache in `/health` response
- **L2 cache status**: Redis health and connection status in `/health`
- Response times: Enable `DEBUG=1` for timing logs

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

