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
- **Multi-threaded Architecture**: Thread-safe DashMap cache with Rayon thread pool
- **Concurrent Request Processing**: Handle 500+ RPS with 2ms average latency
- **Lock-free Operations**: Atomic counters and non-blocking data structures
- **Parallel CPU Tasks**: Background template rendering with spawn_blocking
- **Smart Cache Strategy**: Thread-safe caching with 90% faster cache hits
- **Database Connection Pool**: Optimized for 16-core systems (32 max connections)
- **Chart Module Bundling**: Optimized JavaScript asset delivery

### 🔧 Technical Stack
- **Backend**: Rust + Axum (high-performance async web framework)
- **Database**: PostgreSQL with optimized connection pooling (32 max connections)
- **Caching**: Thread-safe DashMap with atomic operations
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

### Multi-threading Architecture
```
┌─────────────────┐    ┌──────────────────────────────────────┐
│ Concurrent      │    │           Axum Server               │
│ Clients         │◄──►│                                     │
│                 │    │ ┌─────────────┐ ┌─────────────────┐ │
│ 500+ RPS        │    │ │ DashMap     │ │ Rayon ThreadPool│ │
│ 2ms latency     │    │ │ Cache       │ │                 │ │
└─────────────────┘    │ │             │ │ CPU Tasks       │ │
                       │ │ Thread-Safe │ │ • Template      │ │
                       │ │ • Reports   │ │   Rendering     │ │
                       │ │ • Chart JS  │ │ • Data          │ │
                       │ │ • Atomic    │ │   Processing    │ │
                       │ │   Counters  │ │ • Parallel      │ │
                       │ └─────────────┘ │   Operations    │ │
                       │                 └─────────────────┘ │
                       │ ┌─────────────┐ ┌─────────────────┐ │
                       │ │ Connection  │ │ tokio Runtime   │ │
                       │ │ Pool        │ │                 │ │
                       │ │             │ │ Async I/O       │ │
                       │ │ 32 Max      │ │ • HTTP          │ │
                       │ │ 8 Min       │ │ • Database      │ │
                       │ │ PostgreSQL  │ │ • WebSocket     │ │
                       │ └─────────────┘ └─────────────────┘ │
                       └──────────────────────────────────────┘
```

### Caching Strategy
```
┌─────────────────┐    ┌──────────────────┐    ┌──────────────┐
│   Client        │◄──►│  Axum Server     │◄──►│ PostgreSQL   │
│                 │    │                  │    │              │
│ Cache: 15s      │    │ ┌──────────────┐ │    │ Reports      │
│ HTTP Headers    │    │ │ Thread-Safe  │ │    │ Data         │
└─────────────────┘    │ │ DashMap      │ │    └──────────────┘
                       │ │ Cache        │ │
                       │ │              │ │    ┌──────────────┐
                       │ │ • Per-ID     │ │◄──►│ Redis        │
                       │ │ • Latest     │ │    │              │
                       │ │ • Chart JS   │ │    │ WebSocket    │
                       │ │ • Atomic Ops │ │    │ PubSub       │
                       │ └──────────────┘ │    └──────────────┘
                       │                  │
                       │ Rayon ThreadPool │
                       │ (16 CPU cores)   │
                       └──────────────────┘
```

### Performance Features
- **🚄 500+ RPS**: Handle 500+ concurrent requests per second
- **⚡ 2ms Latency**: Sub-2ms average response time under high load
- **🔄 Multi-threaded**: 16-core CPU utilization with Rayon ThreadPool
- **📊 90% Cache Boost**: Cache hits are 90% faster than DB queries
- **💾 Thread-Safe**: Lock-free atomic operations and DashMap caching
- **🔄 Smart Invalidation**: Automatic cache updates with new reports

### Benchmark Results
```
📊 Performance Test Results (16 CPU cores):

Light Load:   50 RPS  | 20ms avg latency
Medium Load: 200 RPS  |  5ms avg latency  
Heavy Load:  500 RPS  |  2ms avg latency
Extreme:     500 RPS  |  2ms avg latency

Cache Performance:
• Cache Miss: 148ms (first request)
• Cache Hit:   13ms (90% improvement)
```

### Request Flow
1. **Cache Hit** → Instant response (cached report + chart modules)
2. **Cache Miss** → Concurrent fetch (DB + assets) → Cache update → Response
3. **WebSocket** → Real-time dashboard updates via Redis pub/sub

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
| `GET` | `/admin/cache/stats` | Cache statistics and performance |
| `POST` | `/admin/cache/clear` | Clear all caches |

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

## 🏗️ Project Structure

```
Web-server-Report/
├── 📁 src/
│   ├── 🦀 main.rs              # Multi-threaded server + DashMap cache
│   ├── 📊 data_service.rs      # External API data fetching  
│   └── 🔌 websocket_service.rs # Real-time WebSocket handler
├── 📁 scripts/                 # Performance testing & benchmarks
│   ├── ⚡ simple_rps_test.sh   # RPS benchmark (500+ RPS)
│   ├── 📊 advanced_benchmark.sh # Comprehensive performance test
│   └── 🔥 stress_test.sh       # Load testing script
├── 📁 dashboards/              # Dashboard templates & assets
│   ├── 🏠 home.html            # Homepage template
│   ├── 💰 crypto_dashboard/    # Crypto-specific templates
│   └── 📈 stock_dashboard/     # Stock-specific templates (future)
├── 📁 shared_assets/           # Global CSS, JS, chart modules
│   ├── 🎨 css/                # Stylesheets
│   └── ⚙️ js/chart_modules/   # Modular chart components
├── 📁 shared_components/       # Reusable HTML components
├── ⚙️ Cargo.toml              # Rust dependencies (rayon, dashmap, etc)
├── 🐳 Dockerfile              # Container configuration
├── 🚂 railway.json           # Railway deployment config
├── 📋 nixpacks.toml          # Build configuration
└── 🌱 .env.example           # Environment template
```

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
- Cache is automatically primed at startup
- New reports update cache on first access
- Restart server to clear all caches: `pkill web-server-report && cargo run`

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
- Cache statistics: `curl http://localhost:8000/admin/cache/stats`
- RPS benchmarks: Run `./scripts/simple_rps_test.sh`
- WebSocket status: Check Redis connection logs
- Memory usage: Monitor DashMap cache size in `/metrics`
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

**License**: MIT License - see LICENSE file for details

**Support**:
- 🐛 Bug reports: [Create GitHub Issue](https://github.com/thichuong/Web-server-Report/issues)
- 💡 Feature requests: [GitHub Discussions](https://github.com/thichuong/Web-server-Report/discussions)
- 📧 Contact: [Your Email]

**Related Projects**:
- 🤖 [Crypto-Dashboard-and-AI-ReportGenerator](https://github.com/thichuong/Crypto-Dashboard-and-AI-ReportGenerator) - Admin UI & AI report generation

---

⭐ **Star this repo** if it helps you build better crypto dashboards! 

Built with ❤️ using Rust 🦀

