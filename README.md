# Web Server Report - High-Performance Crypto Dashboard

[![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange?logo=rust)](https://www.rust-lang.org/)
[![Axum](https://img.shields.io/badge/Axum-0.6-blue)](https://github.com/tokio-rs/axum)
[![Redis](https://img.shields.io/badge/Redis-Streams-red?logo=redis)](https://redis.io/)
[![Docker](https://img.shields.io/badge/Docker-Ready-2496ED?logo=docker)](https://www.docker.com/)
[![License](https://img.shields.io/badge/License-Apache%202.0-green)](LICENSE)

**Ultra-fast Rust web server** achieving **44,700+ RPS** with **11ms latency** under extreme load. Built with **Service Islands Architecture**, 4-tier caching (RAM/Redis), **Zero-Allocation Pre-rendering**, and Declarative Shadow DOM.

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

### Core Services Architecture

The application is structured into domain-driven services for maintainability and scalability:

| Module | Name | Responsibility |
|-------|------|----------------|
| **Services** | Business Logic | Core application features like `dashboard`, `crypto_reports`, and `shared` logic. |
| **Handlers** | Request Processing | Axum HTTP handlers for processing inbound requests. |
| **Routes** | Routing & APIs | Registering routes and middleware (e.g., `api_routes.rs`). |
| **DTOs** | Data Transfer | Type-safe JSON request and response payloads. |
| **Infrastructure** | Foundational | Real-time streams (`stream.rs`) and shared assets. |

### Zero-Allocation Pre-rendering

Layer 5 implements intelligent pre-rendering for high-traffic routes to minimize CPU usage:
- **Homepage**: Pre-compressed GZIP content is rendered during startup and stored in RAM (`Vec<u8>`). Requests are served instantly with zero allocations.
- **Report Frames**: The static "shell" of crypto reports is pre-rendered. Dynamic data (IDs, charts, tokens) is injected via optimized string replacement, bypassing expensive template re-rendering.

### Declarative Shadow DOM (DSD)

Modern rendering approach replacing legacy iframe-based architecture:

- **30-40% faster** page load vs iframe
- **SEO-friendly** - Content indexable by search engines
- **Style isolation** - Native Shadow DOM encapsulation
- **No postMessage** - Direct JavaScript access
- Browser support: Chrome 90+, Edge 91+, Safari 16.4+, Firefox 123+

### Multi-tier Cache System

Sophisticated 4-level caching with **Stampede Protection**:

1.  **Level 0 (Route Cache)**: Pre-rendered static frames in RAM (`OnceCell`) checked immediately at the route level.
2.  **Level 1 (Moka)**: In-memory LRU cache (1000 entries) for dynamic reports and list pages.
3.  **Level 2 (Redis)**: Distributed persistent cache storing compressed GZIP payloads.
4.  **Level 3 (Stream Cache)**: Optimized Redis Stream reading with local caching.

### Real-time Data Pipeline

Redis Streams integration for microservices communication:

- **Consumer role**: Main service reads from `market_data_stream`
- **Publisher**: Separate Websocket Service handles external APIs
- **Throughput**: 10,000+ entries/second
- **Latency**: <2ms read operations

---

## Architecture Overview

### Microservices Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Websocket Service (Port 3001)                   │
│  External APIs -> Redis Streams (market_data_stream)               │
└──────────────────────────────┬──────────────────────────────────────┘
                               │ XREAD
                               ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    Main Service (Port 8000) - This Repo             │
│                                                                     │
│  ┌───────────────────────────────────────────────────────────────┐  │
│  │  Services (src/services/)                                     │  │
│  │  • crypto_reports: Dashboard & Renderings                     │  │
│  │  • dashboard: Pre-rendering Homepage RAM Cache                │  │
│  │  • shared: Utilities, compression, seo                        │  │
│  └───────────────────────────────────────────────────────────────┘  │
│                              │                                      │
│                              ▼                                      │
│  ┌───────────────────────────────────────────────────────────────┐  │
│  │  Handlers & Routes (src/handlers/, src/routes/)               │  │
│  │  • Request logic, API routing                                 │  │
│  └───────────────────────────────────────────────────────────────┘  │
│                              │                                      │
│                              ▼                                      │
│  ┌───────────────────────────────────────────────────────────────┐  │
│  │  Infrastructure (src/stream.rs, src/dto/)                     │  │
│  │  • Data flowing, types, DB Pool, Redis Connections            │  │
│  └───────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────┘
```

### Request Flow with Pre-rendering

```
Client Request
      │
      ▼
┌─────────────────┐
│  Axum Router    │
│  (src/routes/)  │
└─────────────────┘
      │
      ▼
┌──────────────────────────────────────────────┐
│  Services Logic (src/services/)              │
│                                              │
│  [1] Check Pre-rendered RAM Cache?           │
│      ├── YES -> SERVE INSTANTLY (0 allocs)   │
│      └── NO  -> [2] Render/Pre-render        │
└──────────────────────────────────────────────┘
                        │
                        ▼ (Dynamic Data Needed)
┌─────────────────────────────────────────────────────────┐
│                    Data Cache                           │
│  L1 (Moka) ◄──── Cache Miss ────► L2 (Redis)            │
└─────────────────────────────────────────────────────────┘
```

---

## Performance Metrics

```
╔═══════════════════════════════════════════════════════════════════╗
║                    PERFORMANCE SUMMARY                             ║
╠═══════════════════════════════════════════════════════════════════╣
║  Peak Throughput      │  44,714.2 req/s (sustained)               ║
║  Average Latency      │  11.1ms (under heavy concurrency)         ║
║  Homepage Latency     │  <0.2ms (served from RAM)                 ║
║  Success Rate         │  100%                                     ║
║  Cache Hit Rate       │  98%+ overall (L0/L1 optimized)           ║
╚═══════════════════════════════════════════════════════════════════╝
```

---

## Getting Started

### Prerequisites

- **Rust** 1.70+ ([Install Rust](https://rustup.rs/))
- **PostgreSQL** database
- **Redis** server (required)
- **Node.js** 18+ (for frontend)

### Local Development

```bash
# Clone repository
git clone https://github.com/thichuong/Web-server-Report.git
cd Web-server-Report

# Setup Environment
cp .env.example .env
nano .env

# Build and Run
cargo run

# Build Frontend (optional if using pre-built assets)
npm install && npm run build
```

Server starts at `http://localhost:8000`.

### Environment Variables

```env
DATABASE_URL=postgresql://user:password@localhost:5432/database
REDIS_URL=redis://localhost:6379
AUTO_UPDATE_SECRET_KEY=your_secret
WEBSOCKET_SERVICE_URL=ws://localhost:8081
```

---

---

## Testing

Run the test suite to ensure system stability:

### Unit Tests
Run standard unit tests (DTOs, Handlers, Utilities):
```bash
cargo test
```

### Integration Tests
Run integration tests (Requires running Database and Redis):
```bash
cargo test -- --ignored
```

### Code Quality
Run linter and formatter:
```bash
cargo check
cargo clippy
```

### Benchmarking Performance
You can test the Requests Per Second (RPS) of the server using the provided script (requires `ab` - Apache Benchmark):

1. **Ensure the server is running**: `cargo run --release`
2. **Run the benchmark**:
   ```bash
   ./test_rps.sh http://localhost:8000/ 500 50000
   ```
   ```bash
   ./test_rps.sh http://127.0.0.1:8000/crypto_report 500 50000
   ```
   *Parameters: [URL] [Concurrency] [Total Requests]*

If `ab` is not installed, install it via:
- **Ubuntu/Debian**: `sudo apt install apache2-utils`
- **Fedora/CentOS**: `sudo dnf install httpd-tools`

## Project Structure

```
Web-server-Report/
├── src/
│   ├── main.rs                          # Entry point
│   ├── routes/                          # Axum routes (e.g. api_routes.rs)
│   ├── handlers/                        # API request handlers
│   ├── services/                        # Core Domain Logic
│   │   ├── crypto_reports/              # Report Generation & Logic
│   │   │   ├── rendering/               # DSD / HTML renderers
│   │   │   └── ...
│   │   ├── dashboard/                   # Homepage & Dashboard features
│   │   └── shared/                      # Utilities, Compression, SEO
│   ├── dto/                             # Data Transfer Objects
│   └── stream.rs                        # Redis Streams connectivity
├── dashboards/                          # HTML Templates
├── shared_assets/                       # Static Files (CSS/JS)
├── docs/                                # Detailed Documentation
├── Dockerfile.railway                   # Production build
└── Cargo.toml                           # Dependencies
```

---

## Deployment

### Railway / Docker

```bash
docker build -f Dockerfile.railway -t crypto-dashboard .
docker run -p 8000:8000 --env-file .env crypto-dashboard
```

Health check: `curl http://localhost:8000/health`

---

## Documentation

See `docs/` for detailed guides:

- [SERVICE_ISLANDS_ARCHITECTURE.md](docs/SERVICE_ISLANDS_ARCHITECTURE.md)
- [DECLARATIVE_SHADOW_DOM.md](docs/DECLARATIVE_SHADOW_DOM.md)
- [CACHE_ARCHITECTURE_ANALYSIS.md](docs/CACHE_ARCHITECTURE_ANALYSIS.md)

---

## License

Apache License 2.0 - [LICENSE](LICENSE)

<p align="center">
  <b>Built with Rust for maximum performance</b><br>
  <sub>16,829+ RPS | 5.2ms latency | Zero-Allocation Pre-rendering</sub>
</p>
