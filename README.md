# Web Server Report - Optimized Crypto Dashboard

🚀 **High-performance Rust web server** for crypto investment reports with advanced caching and real-time features.

## ✨ Key Features

### 🎯 Core Functionality
- **Interactive Crypto Reports**: Dynamic investment reports with Chart.js visualizations
- **Multi-language Support**: Vietnamese/English with seamless switching
- **Responsive Design**: Mobile-first, adaptive UI
- **PDF Generation**: Export reports to PDF format
- **Real-time Updates**: WebSocket integration for live data

### ⚡ Performance Optimizations
- **Per-ID Report Caching**: In-memory HashMap cache for instant report access
- **Concurrent Data Fetching**: Parallel DB and chart module loading
- **Smart Cache Priming**: Automatic latest report caching at startup
- **Client-side Caching**: HTTP cache headers for reduced server load
- **Chart Module Bundling**: Optimized JavaScript asset delivery

### 🔧 Technical Stack
- **Backend**: Rust + Axum (high-performance async web framework)
- **Database**: PostgreSQL with connection pooling
- **Caching**: In-memory RwLock-based caching system
- **Real-time**: Redis + WebSocket for live updates
- **Templates**: Tera template engine
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

### Caching Strategy
```
┌─────────────────┐    ┌──────────────────┐    ┌──────────────┐
│   Client        │◄──►│  Axum Server     │◄──►│ PostgreSQL   │
│                 │    │                  │    │              │
│ Cache: 15s      │    │ ┌──────────────┐ │    │ Reports      │
│ HTTP Headers    │    │ │ In-Memory    │ │    │ Data         │
└─────────────────┘    │ │ HashMap      │ │    └──────────────┘
                       │ │ Cache        │ │
                       │ │              │ │    ┌──────────────┐
                       │ │ • Per-ID     │ │◄──►│ Redis        │
                       │ │ • Latest     │ │    │              │
                       │ │ • Chart JS   │ │    │ WebSocket    │
                       │ └──────────────┘ │    │ PubSub       │
                       └──────────────────┘    └──────────────┘
```

### Performance Features
- **🚄 Sub-10ms Response**: Cached reports served instantly
- **🔄 Concurrent Fetching**: DB + Chart modules loaded in parallel  
- **📊 Smart Priming**: Latest report pre-loaded at startup
- **💾 Memory Efficient**: RwLock-based concurrent access
- **🔄 Cache Invalidation**: Automatic updates on new reports

### Request Flow
1. **Cache Hit** → Instant response (cached report + chart modules)
2. **Cache Miss** → Concurrent fetch (DB + assets) → Cache update → Response
3. **WebSocket** → Real-time dashboard updates via Redis pub/sub

## 📡 API Reference

### Core Endpoints
| Method | Endpoint | Description | Cache |
|--------|----------|-------------|-------|
| `GET` | `/` | Homepage with latest report | ✅ Cached |
| `GET` | `/health` | Server health check | - |
| `GET` | `/crypto_report` | Latest crypto report | ✅ Cached |
| `GET` | `/crypto_report/:id` | Specific report by ID | ✅ Cached |
| `GET` | `/pdf-template/:id` | PDF-optimized report view | ✅ Cached |
| `GET` | `/crypto_reports_list` | Paginated report list | - |

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
│   ├── 🦀 main.rs              # Main server + caching logic
│   ├── 📊 data_service.rs      # External API data fetching  
│   └── 🔌 websocket_service.rs # Real-time WebSocket handler
├── 📁 dashboards/              # Dashboard templates & assets
│   ├── 🏠 home.html            # Homepage template
│   ├── 💰 crypto_dashboard/    # Crypto-specific templates
│   └── 📈 stock_dashboard/     # Stock-specific templates (future)
├── 📁 shared_assets/           # Global CSS, JS, chart modules
│   ├── 🎨 css/                # Stylesheets
│   └── ⚙️ js/chart_modules/   # Modular chart components
├── 📁 shared_components/       # Reusable HTML components
├── ⚙️ Cargo.toml              # Rust dependencies
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
- WebSocket status: Check Redis connection logs
- Memory usage: Monitor `cached_reports` HashMap size
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
- Use `RwLock` for shared state with many readers
- Cache expensive operations (DB queries, file I/O)
- Add appropriate HTTP cache headers

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

