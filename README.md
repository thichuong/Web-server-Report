# Web Server Report - High-Performance Crypto Dashboard

[![Rust](https://img.shields.io/badge/Rust-2024%20Edition-orange?logo=rust)](https://www.rust-lang.org/)
[![Axum](https://img.shields.io/badge/Axum-0.8-blue)](https://github.com/tokio-rs/axum)
[![Tokio](https://img.shields.io/badge/Tokio-1.52-black?logo=tokio)](https://tokio.rs/)
[![Multi-tier-cache](https://img.shields.io/badge/Multi--Tier--Cache-0.6.7-purple)](https://crates.io/crates/multi-tier-cache)
[![Redis](https://img.shields.io/badge/Redis-Streams-red?logo=redis)](https://redis.io/)
[![SQLx](https://img.shields.io/badge/SQLx-0.9-blue?logo=postgresql)](https://github.com/launchbadge/sqlx)
[![CI/CD](https://img.shields.io/badge/CI%2FCD-GitHub%20Actions-2088FF?logo=github-actions)](.github/workflows/rust-cd.yml)
[![Target](https://img.shields.io/badge/Target-x86__64--unknown--linux--musl-blue?logo=linux)](https://musl.libc.org/)
[![License](https://img.shields.io/badge/License-Apache%202.0-green)](LICENSE)

An enterprise-grade, ultra-high-throughput **Rust** web server achieving **44,700+ RPS** with **11ms latency** under 500 concurrent connections. Built with **Service Islands Architecture**, 4-tier caching (RAM / Moka / Redis / Streams), **Zero-Allocation Pre-rendering**, and **Declarative Shadow DOM (DSD)**.

> **Microservices Topology**: This is the **Main Web Presentation & Consumption Service**. External exchange API polling and live WebSocket broadcasts are handled by the decoupled [Web-server-Report-websocket](https://github.com/thichuong/Web-server-Report-websocket) microservice via **Redis Streams** (`market_data_stream`).

---

## Table of Contents

- [Key Features](#-key-features)
- [Architecture Overview](#-architecture-overview)
- [Performance Metrics](#-performance-metrics)
- [Route & API Reference](#-route--api-reference)
- [Getting Started](#-getting-started)
- [Testing & Quality Assurance](#-testing--quality-assurance)
- [Benchmarking Performance](#-benchmarking-performance)
- [Project Structure](#-project-structure)
- [Deployment](#-deployment)
- [Documentation](#-documentation)
- [License](#-license)

---

## 🚀 Key Features

### 1. Service Islands Architecture
The codebase is structured into strictly bounded, decoupled layers:
- **Routing Layer (`src/routes/`)**: Axum routes with immediate L0/L1 cache bypass.
- **Handler Layer (`src/services/`)**: Transport abstraction using `RenderedContent` and DTO mapping.
- **Data Communication Layer (`src/services/data_communication/`)**: Abstracted database access (`SQLx`) and cached queries.
- **Real-time Ingestion Layer (`src/stream.rs`)**: Asynchronous Redis Streams reader (`market_data_stream`).
- **Templating & Presentation Layer (`src/services/crypto_reports/rendering/`)**: DSD rendering, GEO metadata, breadcrumbs, and Tera templates.

### 2. Zero-Allocation Pre-rendering (L0 Cache)
- **Homepage (`/`)**: Pre-rendered and Gzip-compressed directly into RAM (`Vec<u8>`) during application boot (`init_homepage_cache`). Requests are served with **< 0.2ms latency** and zero dynamic allocations.
- **Report Frames**: Static HTML/DSD frames are cached in memory; dynamic parameters (tokens, language flags) are injected via optimized string replacement.

### 3. Declarative Shadow DOM (DSD)
Modern server-side component encapsulation replacing legacy `<iframe>` approaches:
- **30-40% Faster** page load times vs iframe-based isolation.
- **SEO & AI Bot Friendly**: Fully crawlable by Googlebot, Bingbot, and LLM crawlers.
- **Native Style & Script Isolation**: Zero CSS leakage to the parent application.
- **Multi-Language Support**: Seamless instant switching between Vietnamese (`vi`) and English (`en`).
- **Cryptographic Security**: Blake3 token verification (`sb_<token>`) with constant-time comparison to prevent timing attacks.

### 4. Multi-Tier Cache with Stampede Protection
Orchestrated via the [`multi-tier-cache`](https://crates.io/crates/multi-tier-cache) crate:
1. **Level 0 (Route RAM Cache)**: Pre-rendered static pages in memory.
2. **Level 1 (Moka In-Memory Cache)**: High-concurrency LRU cache (1,000 entries, 30m TTL, 2m TTI).
3. **Level 2 (Redis Distributed Cache)**: Persistent multi-node cache storing pre-compressed GZIP blobs.
4. **Level 3 (Stream Cache)**: Cache-first pattern on Redis Streams to minimize `XREAD` overhead.
- **Stampede Protection**: Internal request coalescing prevents redundant database or stream reads during concurrent spikes.

### 5. Market Analytics & SEO Suite
- **Market Analytics Dashboard (`/market_analytics`, `/analytics`)**: Spot vs. Futures Volume comparisons and Open Interest (OI) analysis.
- **Dynamic XML Sitemap (`/sitemap.xml`)**: Automated sitemap generation with 1-hour caching.
- **RSS 2.0 Feed (`/rss.xml`, `/rss`)**: Automated syndication feed for aggregators and search engines.
- **Structured GEO & Schema.org Metadata**: Complete JSON-LD schemas (`Article`, `FinancialProduct`, `BreadcrumbList`).

### 6. Idiomatic Rust & Strict Error Handling
- **Edition 2024 / 2021 Idioms**: Zero unsafe blocks in application code.
- **Strictly No Unwrap**: `.unwrap()` is prohibited across the codebase (enforced via `clippy::unwrap_used`). All errors are propagated cleanly using `Result`, `Option`, and the `?` operator.

---

## 🏗 Architecture Overview

### Microservices Communication Flow

```mermaid
flowchart TD
    subgraph External["External Data Sources"]
        Binance["Binance APIs"]
        CoinGecko["CoinGecko / CoinMarketCap"]
        FNG["Fear & Greed Index"]
    end

    subgraph WebSocket_Service["Websocket Microservice (Port 8081)"]
        Ingestion["API Polling Engine"]
        Producer["Redis Stream Producer"]
        WSServer["WebSocket Server (ws://localhost:8081)"]
    end

    subgraph Redis_Cluster["Redis Infrastructure"]
        Stream["Redis Stream: market_data_stream"]
        L2_Cache["Redis L2 Distributed Cache"]
    end

    subgraph Main_Service["Main Web Server (Port 8000) - This Repo"]
        StreamReader["src/stream.rs: RedisStreamReader"]
        Router["src/routes/ - Axum Router"]
        L0_L1["L0 RAM & L1 Moka Cache"]
        Services["Service Islands Layer 5"]
        DB[(PostgreSQL)]
    end

    External --> Ingestion
    Ingestion --> Producer
    Producer -->|XADD| Stream
    Ingestion --> WSServer

    Stream -->|XREAD Cache-First| StreamReader
    StreamReader --> Services
    Services <--> L0_L1
    Services <--> L2_Cache
    Services <--> DB
    Router --> Services

    Client[Web Browsers / Crawlers] <-->|HTTP / DSD| Router
    WSServer -->|Live Tickers| Client
```

### Request Resolution Pipeline

```
Incoming Request
      │
      ▼
┌─────────────────────────┐
│  Axum Route Handler     │
│  (src/routes/)          │
└───────────┬─────────────┘
            │
            ├─► [1] Immediate L0/L1 Cache Check (Gzip Buffer)?
            │        └── YES ──► Serve directly (Zero allocation, < 0.2ms)
            │
            └─► [2] Cache MISS ──► Layer 5 Service Orchestration
                     │
                     ├─► Check L2 Redis Cache (Compressed Bytes)
                     │
                     ├─► Check L3 Redis Stream (`RedisStreamReader`)
                     │
                     ├─► Query PostgreSQL (`CryptoDataService` / SQLx)
                     │
                     ▼
             Render Tera / DSD Template
                     │
                     ▼
             Flate2 Gzip Compression
                     │
                     ▼
             Populate L1 + L2 Cache & Return Response
```

---

## 📊 Performance Metrics

```
╔══════════════════════════════════════════════════════════════════════════╗
║                          BENCHMARK SUMMARY                               ║
╠══════════════════════════════════════════════════════════════════════════╣
║  Peak Sustained RPS     │  44,714.2 requests/sec (Apache Benchmark)      ║
║  Concurrency Tested     │  500 concurrent connections                    ║
║  Mean Latency           │  11.1 ms (under heavy concurrent load)         ║
║  Homepage Latency       │  < 0.2 ms (served directly from RAM L0)        ║
║  Success Rate           │  100.00% (50,000 / 50,000 requests)            ║
║  Cache Hit Rate         │  98%+ overall across L0/L1/L2 tiers            ║
║  Memory Footprint       │  ~25MB baseline RSS                            ║
╚══════════════════════════════════════════════════════════════════════════╝
```

---

## 📡 Route & API Reference

### Web UI Routes

| Method | Endpoint | Description | Cache Policy |
|---|---|---|---|
| `GET` | `/` | Pre-rendered Homepage (Market summary & quick links) | L0 RAM (`max-age=300`) |
| `GET` | `/crypto_report` | Latest Crypto Report with Declarative Shadow DOM | L0/L1/L2 (`max-age=300`) |
| `GET` | `/crypto_report/{id}` | Specific Crypto Report by numerical ID | L1/L2 (`max-age=300`) |
| `GET` | `/crypto_reports_list` | Paginated Report History (`?page=1`) | L1/L2 (`max-age=60`) |
| `GET` | `/market_analytics` | Spot vs Futures Volume & Open Interest Analytics | L1/L2 (`max-age=300`) |
| `GET` | `/analytics` | Alias for `/market_analytics` | L1/L2 (`max-age=300`) |

### REST API Endpoints

| Method | Endpoint | Description |
|---|---|---|
| `GET` | `/api/dashboard/data` | Real-time market data JSON read from Redis Stream |
| `GET` | `/api/crypto/dashboard-summary` | Summary JSON payload for dashboards |
| `GET` | `/api/crypto_reports/{id}/shadow_dom` | Returns raw Shadow DOM HTML fragment (`?token=...&lang=...`) |
| `GET` | `/api/health` | API subsystem health status |
| `GET` | `/api/websocket/stats` | Status and redirect info for WebSocket microservice |

### SEO & Syndication Feeds

| Method | Endpoint | Content-Type | Description |
|---|---|---|---|
| `GET` | `/sitemap.xml` | `application/xml` | Dynamic XML sitemap with 1-hour cache |
| `GET` | `/rss.xml` / `/rss` | `application/rss+xml` | RSS 2.0 feed containing latest 20 reports |
| `GET` | `/robots.txt` | `text/plain` | Crawler indexation rules |

### System & Admin Endpoints

| Method | Endpoint | Description |
|---|---|---|
| `GET` | `/health` | Core server and stream health status (`200 OK`) |
| `GET` | `/metrics` | Server performance and multi-tier cache metrics |
| `GET` | `/admin/cache/stats` | Detailed cache statistics (hits, misses, promotions, hit rate) |
| `GET` | `/admin/cache/clear` | Invalidate all keys across L1 and L2 cache |

---

## 🛠 Getting Started

### Prerequisites

- **Rust**: 1.70+ or Rust 2024 Edition ([Install Rust](https://rustup.rs/))
- **PostgreSQL**: 14+ database
- **Redis**: 6+ (Redis Streams enabled)
- **Node.js**: 18+ (for building frontend assets)

### Local Setup

```bash
# 1. Clone the repository
git clone https://github.com/thichuong/Web-server-Report.git
cd Web-server-Report

# 2. Configure Environment Variables
cp .env.example .env
nano .env

# 3. Build frontend assets (ESBuild)
npm install
npm run build

# 4. Run the development server
cargo run
```

The web server will start at `http://localhost:8000`.

### Environment Configuration (`.env`)

```env
# Database Configuration
DATABASE_URL=postgresql://user:password@localhost:5432/crypto_reports

# Redis Configuration (L2 Cache + Streams)
REDIS_URL=redis://127.0.0.1:6379

# Server Binding
HOST=0.0.0.0
PORT=8000
RUST_LOG=info

# Microservices Integration
WEBSOCKET_SERVICE_URL=ws://localhost:8081
AUTO_UPDATE_SECRET_KEY=your_secret_key_here
```

---

## 🧪 Testing & Quality Assurance

### Run Unit Tests
```bash
cargo test
```
*All 38 unit tests run in < 0.01s.*

### Run Integration Tests (Requires DB & Redis)
```bash
cargo test -- --ignored
```

### Strict Code Quality & Linting
Enforce strict zero-unwrap policy and Clippy pedantic rules:
```bash
cargo clippy --all-targets -- -D warnings -D clippy::unwrap_used
```

---

## ⚡ Benchmarking Performance

Run the included Apache Benchmark testing script:

```bash
# Benchmark Homepage (Pre-rendered RAM Cache)
./test_rps.sh http://localhost:8000/ 500 50000

# Benchmark Crypto Report (Declarative Shadow DOM)
./test_rps.sh http://localhost:8000/crypto_report 500 50000
```

*Parameters: `./test_rps.sh [URL] [Concurrency] [Total Requests]`*

---

## 📁 Project Structure

```
Web-server-Report/
├── .agents/                                # Agent workflows and coding rules
├── dashboards/                             # Tera HTML Templates
│   ├── crypto_dashboard/                   # Crypto report & analytics views
│   │   ├── assets/                         # Dashboard CSS/JS assets
│   │   └── routes/
│   │       ├── analytics/                  # Market analytics templates
│   │       └── reports/                    # View & list report templates
│   └── home.html                           # Pre-rendered homepage template
├── shared_assets/                          # Static assets (CSS, JS, Logos)
├── shared_components/                      # Modular UI components (DSD, Toggles)
├── src/                                    # Rust Source Code
│   ├── dto/                                # Data Transfer Objects & Responses
│   │   ├── common.rs                       # Shared DTOs (Health, Status)
│   │   └── responses/                      # Typed JSON responses
│   ├── routes/                             # Axum HTTP Routes (Layer 1)
│   │   ├── api.rs                          # REST APIs
│   │   ├── crypto_reports.rs               # DSD Report routes
│   │   ├── homepage.rs                     # Pre-rendered homepage route
│   │   ├── market_analytics.rs             # Spot vs Futures & OI route
│   │   ├── rss_feed.rs                     # RSS 2.0 feed route
│   │   ├── seo.rs                          # Sitemap.xml route
│   │   ├── static_files.rs                 # Static asset server
│   │   ├── system.rs                       # Health, metrics & admin cache
│   │   └── mod.rs                          # Router composition
│   ├── services/                           # Service Islands Domain Logic (Layer 5)
│   │   ├── crypto_reports/                 # Crypto report domain island
│   │   │   ├── rendering/                  # DSD renderer, GEO metadata, Breadcrumbs
│   │   │   ├── data_manager.rs             # Report data orchestration
│   │   │   ├── handlers.rs                 # Domain request handlers
│   │   │   ├── report_creator.rs           # Report builder
│   │   │   └── template_orchestrator.rs    # Tera context orchestration
│   │   ├── dashboard_data_service.rs       # Layer 3 data service for dashboard
│   │   ├── dashboard.rs                    # Dashboard & homepage service
│   │   ├── data_communication/             # Layer 3 Database & Cache communication
│   │   │   └── crypto_data_service.rs      # SQLx queries with multi-tier cache
│   │   └── shared/                         # Shared utilities
│   │       ├── cache_utils.rs              # Cache helpers & Gzip response builder
│   │       ├── compression.rs              # Flate2 compression utilities
│   │       ├── error.rs                    # Layer 5 custom error types
│   │       ├── response_builder.rs         # HTTP response builder
│   │       ├── rss_creator.rs              # RSS 2.0 XML generator
│   │       ├── security.rs                 # Blake3 cryptographic sandbox tokens
│   │       ├── sitemap_creator.rs          # Sitemap XML generator
│   │       └── websocket.rs                # WebSocket URL resolver
│   ├── error.rs                            # Top-level application error handler
│   ├── lib.rs                              # Library root
│   ├── main.rs                             # Application entry point & graceful shutdown
│   ├── state.rs                            # Global AppState (DB, Cache, Tera)
│   └── stream.rs                           # Redis Streams Reader (Layer 4)
├── tests/                                  # Integration test suite
├── build.js                                # Frontend ESBuild bundling pipeline
├── Cargo.toml                              # Rust dependencies & optimization profiles
├── package.json                            # Node.js build configuration
├── test_rps.sh                             # Apache Benchmark RPS testing script
├── architecture.md                         # Detailed Architectural Specification
└── README.md                               # Project overview and documentation
```

---

## 🚢 Deployment & CI/CD

### Automated GitHub Actions Workflow (`.github/workflows/rust-cd.yml`)

The repository uses automated continuous deployment triggered on pushes to `main`:

1. **Static Compilation with Musl**:
   - Cross-compiles a self-contained, statically linked binary targeting `x86_64-unknown-linux-musl`.
   - Embeds OpenSSL (`vendored`) with zero dynamic runtime dependency.
   ```bash
   cargo build --release --target x86_64-unknown-linux-musl
   ```
2. **Google Cloud IAP Deployment**:
   - Authenticates securely via Google Cloud Service Account (`GCP_SA_KEY`).
   - Uses Google Cloud Identity-Aware Proxy (IAP) SSH tunneling to safely transfer and deploy the release binary to the Google Cloud VM without opening public SSH ports.
   - Gracefully stops any running instance and restarts the new binary in the background (`setsid nohup ./target/release/web-server-report > app.log 2>&1 &`).

### Production Release Profile
In `Cargo.toml`, the release profile is tuned for maximum throughput and minimal binary footprint:
```toml
[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
panic = "abort"
strip = true
overflow-checks = false
```

---

## 📖 Documentation

For in-depth technical specifications, please consult:
- [architecture.md](architecture.md) — Comprehensive Service Islands Architecture, Multi-Tier Cache Topology, and Security Design.

---

## 📄 License

Licensed under the Apache License, Version 2.0 — see the [LICENSE](LICENSE) file for details.

<p align="center">
  <b>Built with Rust for Maximum Performance & Reliability</b><br>
  <sub>44,700+ RPS | 11ms Latency | Zero-Allocation Pre-rendering</sub>
</p>
