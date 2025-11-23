# Web Server Report - High-Performance Crypto Dashboard

[![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange?logo=rust)](https://www.rust-lang.org/)
[![Axum](https://img.shields.io/badge/Axum-0.6-blue)](https://github.com/tokio-rs/axum)
[![Redis](https://img.shields.io/badge/Redis-Streams-red?logo=redis)](https://redis.io/)
[![Docker](https://img.shields.io/badge/Docker-Ready-2496ED?logo=docker)](https://www.docker.com/)
[![License](https://img.shields.io/badge/License-Apache%202.0-green)](LICENSE)

**Ultra-fast Rust web server** achieving **16,829+ RPS** with **5.2ms latency** for crypto investment reports. Built with **Service Islands Architecture**, multi-tier caching with stampede protection, and Declarative Shadow DOM rendering.

> **Microservices Architecture**: This is the **Main Service** handling web presentation and data consumption. External APIs and WebSocket are handled by [Web-server-Report-websocket](https://github.com/thichuong/Web-server-Report-websocket).

---

## Table of Contents

- [Key Features](#-key-features)
- [Architecture Overview](#-architecture-overview)
- [Performance Metrics](#-performance-metrics)
- [Getting Started](#-getting-started)
- [API Reference](#-api-reference)
- [Project Structure](#-project-structure)
- [Deployment](#-deployment)
- [Documentation](#-documentation)
- [Contributing](#-contributing)
- [License](#-license)

---

## Key Features

### Service Islands Architecture

Five-layer separation of concerns for maintainability and scalability:

| Layer | Name | Responsibility |
|-------|------|----------------|
| **Layer 5** | Business Logic | Dashboard processing, Report rendering (DSD), GEO optimization |
| **Layer 4** | Observability | Health checks, Performance monitoring, SSL validation |
| **Layer 3** | Communication | Redis Streams consumer, PostgreSQL operations, Data routing |
| **Layer 2** | External Services | *Moved to [Websocket Service](https://github.com/thichuong/Web-server-Report-websocket)* |
| **Layer 1** | Infrastructure | Cache system (L1+L2), Template registry, App state |

### Declarative Shadow DOM (DSD)

Modern rendering approach replacing legacy iframe-based architecture:

- **30-40% faster** page load vs iframe
- **SEO-friendly** - Content indexable by search engines
- **Style isolation** - Native Shadow DOM encapsulation
- **No postMessage** - Direct JavaScript access
- Browser support: Chrome 90+, Edge 91+, Safari 16.4+, Firefox 123+

### Multi-tier Cache System

Sophisticated 2-tier cache with **Cache Stampede Protection**:

```
┌─────────────────────────────────────────────────────────────┐
│  L1 Cache (Moka)          │  L2 Cache (Redis)              │
├───────────────────────────┼─────────────────────────────────┤
│  In-memory, 2000 entries  │  Distributed, persistent       │
│  5min TTL, <1ms response  │  1hr TTL, 2-5ms response       │
│  90% hit rate             │  75% hit rate                  │
│  get_with() coalescing    │  DashMap+Mutex coalescing      │
└───────────────────────────┴─────────────────────────────────┘
                    ↓
         99.6% performance improvement
         (534ms → 5.2ms under high concurrency)
```

### Real-time Data Pipeline

Redis Streams integration for microservices communication:

- **Consumer role**: Main service reads from `market_data_stream`
- **Publisher**: Separate Websocket Service handles external APIs
- **Throughput**: 10,000+ entries/second
- **Latency**: <2ms read operations

### GEO Optimization

AI and SEO enhancements for maximum discoverability:

- **Schema.org JSON-LD** - Article + FinancialAnalysis structured data
- **Open Graph** - Facebook, LinkedIn social sharing
- **Twitter Cards** - X/Twitter, Grok AI optimization
- **Dynamic Sitemap** - Auto-generated `/sitemap.xml`
- **RSS 2.0 Feed** - `/rss.xml` for feed readers and crawlers

### Additional Features

- **Breadcrumb Navigation** - Structured navigation with JSON-LD schema
- **Related Reports** - Internal linking for engagement optimization
- **Multi-language** - Vietnamese (vi) and English (en) support
- **Gzip Compression** - Automatic response compression
- **WebSocket Support** - Real-time updates via dedicated service

---

## Architecture Overview

### Microservices Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Websocket Service (Port 3001)                    │
│  ┌───────────────────────────────────────────────────────────────┐  │
│  │  Layer 2: External APIs                                       │  │
│  │  • Binance, CoinGecko, CoinMarketCap, TAAPI.io, Finnhub      │  │
│  │  • Circuit breaker + Fallback logic                          │  │
│  │  • WebSocket broadcasting                                     │  │
│  └───────────────────────────────────────────────────────────────┘  │
│                              │                                       │
│                              ▼ XADD                                  │
│                    Redis Streams (market_data_stream)                │
└──────────────────────────────┬──────────────────────────────────────┘
                               │ XREAD
                               ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    Main Service (Port 8000) - This Repo             │
│  ┌───────────────────────────────────────────────────────────────┐  │
│  │  Layer 5: Business Logic                                      │  │
│  │  • Dashboard Island (data aggregation)                        │  │
│  │  • Crypto Reports Island (DSD/Tera rendering)                 │  │
│  │  • GEO Metadata, Breadcrumbs, RSS Creator                    │  │
│  └───────────────────────────────────────────────────────────────┘  │
│                              │                                       │
│                              ▼                                       │
│  ┌───────────────────────────────────────────────────────────────┐  │
│  │  Layer 4: Observability                                       │  │
│  │  • Health checks, Performance monitor, SSL tester             │  │
│  └───────────────────────────────────────────────────────────────┘  │
│                              │                                       │
│                              ▼                                       │
│  ┌───────────────────────────────────────────────────────────────┐  │
│  │  Layer 3: Communication                                       │  │
│  │  • Redis Stream Reader (consume from websocket service)       │  │
│  │  • Data Communication (PostgreSQL operations)                 │  │
│  │  • Dashboard Communication (data routing)                     │  │
│  └───────────────────────────────────────────────────────────────┘  │
│                              │                                       │
│                              ▼                                       │
│  ┌───────────────────────────────────────────────────────────────┐  │
│  │  Layer 1: Infrastructure                                      │  │
│  │  • Cache System (L1 Moka + L2 Redis + Stampede Protection)   │  │
│  │  • Template Registry (Tera templates)                         │  │
│  │  • Chart Modules Island (JS bundling)                        │  │
│  │  • App State (DB pool, Redis connection)                     │  │
│  └───────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────┘
```

### Request Flow

```
Client Request
      │
      ▼
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│  Axum Router    │ ──► │  Layer 5        │ ──► │  Layer 3        │
│  (routes/*.rs)  │     │  Business Logic │     │  Communication  │
└─────────────────┘     └─────────────────┘     └─────────────────┘
                                                        │
                        ┌───────────────────────────────┘
                        ▼
      ┌─────────────────────────────────────────────────────────┐
      │                    Layer 1: Cache                        │
      │  L1 (Moka) ◄──── Cache Miss ────► L2 (Redis)            │
      │      │                                  │                │
      │      └──────── Promotion ───────────────┘                │
      │                     │                                    │
      │                     ▼ (if miss)                          │
      │           Redis Streams / PostgreSQL                     │
      └─────────────────────────────────────────────────────────┘
                        │
                        ▼
               Response (HTML/JSON)
```

---

## Performance Metrics

### Benchmark Results

```
╔═══════════════════════════════════════════════════════════════════╗
║                    PERFORMANCE SUMMARY                             ║
╠═══════════════════════════════════════════════════════════════════╣
║  Peak Throughput      │  16,829.3 req/s (sustained high load)     ║
║  Average Latency      │  5.2ms (with stampede protection)         ║
║  Success Rate         │  100% (zero failures under load)          ║
║  Cache Hit Rate       │  95%+ overall (L1: 90%, L2: 75%)          ║
║  Stampede Protection  │  99.6% improvement vs unprotected         ║
╚═══════════════════════════════════════════════════════════════════╝
```

### Scenario Performance

| Scenario | Clients | RPS | Avg Latency |
|----------|---------|-----|-------------|
| Gradual Ramp-up | 10-100 | 3,498.8 | 0.3ms |
| Sustained High Load | 200 | 16,829.3 | 5.2ms |
| Burst Load | 500 | 291.5 | 681.8ms* |

*Burst scenario limited by client-side connection pooling

### Cache Strategy Performance

| Data Type | Strategy | TTL | Response Time |
|-----------|----------|-----|---------------|
| BTC Price | RealTime | 30s | <1ms (L1 hit) |
| Fear & Greed | ShortTerm | 5min | <1ms (L1 hit) |
| Market Stats | MediumTerm | 1hr | 2-5ms (L2 hit) |
| Technical Indicators | LongTerm | 3hr | 2-5ms (L2 hit) |

---

## Getting Started

### Prerequisites

- **Rust** 1.70+ ([Install Rust](https://rustup.rs/))
- **PostgreSQL** database
- **Redis** server (required for cache + streams)
- **Node.js** 18+ (for frontend build)

### Local Development

```bash
# Clone repository
git clone https://github.com/thichuong/Web-server-Report.git
cd Web-server-Report

# Copy environment template
cp .env.example .env

# Edit .env with your configuration
nano .env

# Build and run (development)
cargo run

# Or with watch mode
cargo install cargo-watch
cargo watch -x run

# Build frontend assets
npm install
npm run build
```

Server starts at `http://localhost:8000`

### Environment Variables

```env
# Database (Required)
DATABASE_URL=postgresql://user:password@localhost:5432/database

# Redis (Required)
REDIS_URL=redis://localhost:6379

# Security (Required)
AUTO_UPDATE_SECRET_KEY=your_secret_key_here

# Server
HOST=0.0.0.0
PORT=8000

# API Keys (Optional - for websocket service)
TAAPI_SECRET=your_taapi_jwt_token
CMC_API_KEY=your_coinmarketcap_key
FINNHUB_API_KEY=your_finnhub_key

# Development
DEBUG=1
RUST_LOG=info
```

### Docker Development

```bash
# Build with optimized Dockerfile
docker build -f Dockerfile.railway -t crypto-dashboard .

# Run container
docker run -p 8000:8000 \
  -e DATABASE_URL="postgresql://..." \
  -e REDIS_URL="redis://..." \
  -e AUTO_UPDATE_SECRET_KEY="..." \
  crypto-dashboard
```

---

## API Reference

### Core Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/` | Homepage with latest report |
| `GET` | `/crypto_report` | Latest report (Declarative Shadow DOM) |
| `GET` | `/crypto_report/:id` | Specific report by ID (DSD) |
| `GET` | `/crypto_report_iframe` | Latest report (legacy iframe) |
| `GET` | `/crypto_report_iframe/:id` | Specific report by ID (iframe) |
| `GET` | `/crypto_reports_list` | Paginated report list |
| `GET` | `/pdf-template/:id` | PDF-optimized report view |

### API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/api/crypto/dashboard-summary` | Cached dashboard data (JSON) |
| `GET` | `/api/crypto/dashboard-summary/refresh` | Force refresh dashboard |
| `GET` | `/api/crypto_reports/:id/shadow_dom` | Shadow DOM content fragment |

### SEO & Discovery

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/sitemap.xml` | Dynamic XML sitemap |
| `GET` | `/rss.xml` | RSS 2.0 feed (20 latest reports) |
| `GET` | `/rss` | Alternative RSS endpoint |
| `GET` | `/robots.txt` | Crawler directives |

### System & Monitoring

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/health` | Health check + cache metrics |
| `GET` | `/cache-stats` | Detailed L1/L2 statistics |
| `POST` | `/clear-cache` | Clear all cache tiers |

---

## Project Structure

```
Web-server-Report/
├── src/
│   ├── main.rs                          # Server initialization
│   ├── performance.rs                   # Performance monitoring
│   ├── routes/                          # Axum route handlers
│   │   ├── mod.rs                       # Router composition
│   │   ├── homepage.rs                  # GET /
│   │   ├── crypto_reports.rs            # Report endpoints
│   │   ├── api.rs                       # API endpoints
│   │   ├── system.rs                    # Health & monitoring
│   │   ├── seo.rs                       # Sitemap endpoint
│   │   ├── rss_feed.rs                  # RSS feed endpoint
│   │   └── static_files.rs              # Asset serving
│   │
│   └── service_islands/                 # Service Islands Architecture
│       ├── mod.rs                       # Islands coordination
│       │
│       ├── layer1_infrastructure/       # Foundation layer
│       │   ├── cache_system_island/     # L1+L2 cache + stampede protection
│       │   │   ├── cache_manager.rs     # Unified cache interface
│       │   │   ├── l1_cache.rs          # Moka in-memory cache
│       │   │   └── l2_cache.rs          # Redis + DashMap coalescing
│       │   ├── app_state_island/        # DB pool, Redis, app state
│       │   ├── shared_components_island/# Template registry, utilities
│       │   └── chart_modules_island/    # JS bundling service
│       │
│       ├── layer3_communication/        # Data layer
│       │   ├── data_communication/      # PostgreSQL operations
│       │   │   └── crypto_data_service.rs
│       │   ├── dashboard_communication/ # Dashboard data routing
│       │   └── redis_stream_reader/     # Redis Streams consumer
│       │
│       ├── layer4_observability/        # Monitoring layer
│       │   └── health_system/           # Health checks, metrics
│       │       ├── health_checker.rs
│       │       ├── performance_monitor.rs
│       │       ├── connectivity_tester.rs
│       │       └── ssl_tester.rs
│       │
│       └── layer5_business_logic/       # Business layer
│           ├── dashboard/               # Dashboard processing
│           │   ├── handlers.rs
│           │   ├── report_manager.rs
│           │   └── template_renderer.rs
│           ├── crypto_reports/          # Report rendering
│           │   ├── handlers.rs          # Route handlers
│           │   ├── report_creator.rs    # Report generation
│           │   ├── template_orchestrator.rs
│           │   └── rendering/           # Rendering engines
│           │       ├── shadow_dom_renderer.rs  # DSD rendering
│           │       ├── iframe_renderer.rs      # Legacy iframe
│           │       ├── geo_metadata.rs         # SEO/AI optimization
│           │       └── breadcrumbs.rs          # Navigation
│           └── shared/                  # Shared utilities
│               ├── compression.rs       # Gzip compression
│               ├── response_builder.rs  # HTTP responses
│               ├── sitemap_creator.rs   # Dynamic sitemap
│               ├── rss_creator.rs       # RSS 2.0 generator
│               └── security.rs          # Token generation
│
├── dashboards/                          # HTML templates
│   ├── home.html                        # Homepage
│   └── crypto_dashboard/
│       ├── routes/reports/
│       │   ├── view_dsd.html            # DSD report view
│       │   ├── view.html                # Legacy iframe view
│       │   └── list.html                # Reports list
│       └── assets/                      # Dashboard assets
│
├── shared_assets/                       # Static assets
│   ├── css/                             # Stylesheets
│   └── js/chart_modules/                # Chart components
│
├── shared_components/                   # Reusable components
│   ├── view_shadow_dom.html             # DSD template
│   └── view_iframe.html                 # Iframe template
│
├── docs/                                # Documentation
│   ├── SERVICE_ISLANDS_ARCHITECTURE.md
│   ├── DECLARATIVE_SHADOW_DOM.md
│   ├── CACHE_ARCHITECTURE_ANALYSIS.md
│   └── RAILWAY_DEPLOYMENT_GUIDE.md
│
├── Cargo.toml                           # Rust dependencies
├── Dockerfile.railway                   # Optimized Docker build
├── railway.json                         # Railway configuration
├── build.js                             # esbuild configuration
└── .env.example                         # Environment template
```

---

## Deployment

### Railway (Recommended)

```bash
# Install Railway CLI
npm install -g @railway/cli

# Login and link project
railway login
railway link

# Set environment variables
railway env set DATABASE_URL="postgresql://..."
railway env set REDIS_URL="redis://..."
railway env set AUTO_UPDATE_SECRET_KEY="..."

# Deploy
railway up
```

**Railway Configuration** (`railway.json`):
```json
{
  "build": {
    "builder": "DOCKERFILE",
    "dockerfilePath": "Dockerfile.railway"
  },
  "deploy": {
    "startCommand": "./web-server-report",
    "healthcheckPath": "/health",
    "healthcheckTimeout": 120
  }
}
```

### Docker Production

```bash
# Multi-stage build (optimized ~200MB image)
docker build -f Dockerfile.railway -t crypto-dashboard:latest .

# Run with environment file
docker run -d \
  --name crypto-dashboard \
  --env-file .env \
  -p 8000:8000 \
  crypto-dashboard:latest
```

### Verification Checklist

After deployment:
- [ ] Health check: `curl https://your-domain/health`
- [ ] Homepage loads: `curl https://your-domain/`
- [ ] Cache stats: `curl https://your-domain/cache-stats`
- [ ] RSS feed: `curl https://your-domain/rss.xml`
- [ ] Sitemap: `curl https://your-domain/sitemap.xml`

---

## Documentation

Detailed documentation available in `docs/`:

| Document | Description |
|----------|-------------|
| [SERVICE_ISLANDS_ARCHITECTURE.md](docs/SERVICE_ISLANDS_ARCHITECTURE.md) | 5-layer architecture details |
| [DECLARATIVE_SHADOW_DOM.md](docs/DECLARATIVE_SHADOW_DOM.md) | DSD implementation guide |
| [CACHE_ARCHITECTURE_ANALYSIS.md](docs/CACHE_ARCHITECTURE_ANALYSIS.md) | Cache system deep-dive |
| [RAILWAY_DEPLOYMENT_GUIDE.md](docs/RAILWAY_DEPLOYMENT_GUIDE.md) | Deployment instructions |
| [GENERIC_CACHE_ARCHITECTURE.md](docs/GENERIC_CACHE_ARCHITECTURE.md) | Layer separation cache design |
| [BUILD_SYSTEM.md](docs/BUILD_SYSTEM.md) | Frontend build pipeline |

---

## Contributing

### Development Workflow

```bash
# Code quality checks
cargo fmt          # Format code
cargo clippy       # Lint code
cargo test         # Run tests

# Performance testing
./scripts/simple_rps_test.sh        # Quick RPS test
./scripts/stress_test.sh            # Load testing
cargo run --example http_load_benchmark
```

### Code Style

- Use `cargo fmt` for consistent formatting
- Follow Rust naming conventions
- Add documentation for public functions
- Include error handling with `tracing` logs

### Architecture Guidelines

1. **Layer Dependencies**: Higher layers depend on lower layers only
2. **No Skip Levels**: Layer 5 → Layer 3 → Layer 1 (no shortcuts)
3. **Cache-First**: Always use cache manager for external data
4. **Concurrency**: Use `tokio::join!`, `DashMap`, `AtomicUsize`
5. **Performance**: Benchmark after significant changes

---

## License

This project is licensed under the **Apache License 2.0** - see the [LICENSE](LICENSE) file for details.

---

## Related Projects

| Project | Description |
|---------|-------------|
| [Web-server-Report-websocket](https://github.com/thichuong/Web-server-Report-websocket) | Websocket service handling external APIs |
| [Crypto-Dashboard-and-AI-ReportGenerator](https://github.com/thichuong/Crypto-Dashboard-and-AI-ReportGenerator) | Admin UI & AI report generation |

---

<p align="center">
  <b>Built with Rust for maximum performance</b><br>
  <sub>16,829+ RPS | 5.2ms latency | 95%+ cache hit rate</sub>
</p>
