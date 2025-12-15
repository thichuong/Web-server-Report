# Web Server Report - High-Performance Crypto Dashboard

[![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange?logo=rust)](https://www.rust-lang.org/)
[![Axum](https://img.shields.io/badge/Axum-0.6-blue)](https://github.com/tokio-rs/axum)
[![Redis](https://img.shields.io/badge/Redis-Streams-red?logo=redis)](https://redis.io/)
[![Docker](https://img.shields.io/badge/Docker-Ready-2496ED?logo=docker)](https://www.docker.com/)
[![License](https://img.shields.io/badge/License-Apache%202.0-green)](LICENSE)

**Ultra-fast Rust web server** achieving **16,829+ RPS** with **5.2ms latency** for crypto investment reports. Built with **Service Islands Architecture**, multi-tier caching (RAM/Redis), **Zero-Allocation Pre-rendering**, and Declarative Shadow DOM.

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
| **Layer 5** | Business Logic | Dashboard processing, Pre-rendering (RAM Cache), Report rendering (DSD) |
| **Layer 4** | Observability | Health checks, Performance monitoring, SSL validation |
| **Layer 3** | Communication | Redis Streams consumer, PostgreSQL operations, Data routing |
| **Layer 2** | External Services | *Moved to [Websocket Service](https://github.com/thichuong/Web-server-Report-websocket)* |
| **Layer 1** | Infrastructure | Cache system (L1+L2), Template registry, App state |

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

Sophisticated multi-level caching with **Stampede Protection**:

1.  **Level 0 (App Cache)**: Pre-rendered static frames in RAM (Layer 5).
2.  **Level 1 (Moka)**: In-memory LRU cache for hot data (Layer 1).
3.  **Level 2 (Redis)**: Distributed persistent cache (Layer 1).

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
│                    Websocket Service (Port 3001)                    │
│  Layer 2: External APIs -> Redis Streams (market_data_stream)       │
└──────────────────────────────┬──────────────────────────────────────┘
                               │ XREAD
                               ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    Main Service (Port 8000) - This Repo             │
│                                                                     │
│  ┌───────────────────────────────────────────────────────────────┐  │
│  │  Layer 5: Business Logic & Pre-rendering                      │  │
│  │  • Homepage RAM Cache (Vec<u8>)                               │  │
│  │  • Report Frames RAM Cache (String)                           │  │
│  │  • Dashboard & Crypto Report Handlers                         │  │
│  └───────────────────────────────────────────────────────────────┘  │
│                              │                                       │
│                              ▼                                       │
│  ┌───────────────────────────────────────────────────────────────┐  │
│  │  Layer 4: Observability (Health, Monitor)                     │  │
│  └───────────────────────────────────────────────────────────────┘  │
│                              │                                       │
│                              ▼                                       │
│  ┌───────────────────────────────────────────────────────────────┐  │
│  │  Layer 3: Communication (Redis Streams, Postgres)             │  │
│  └───────────────────────────────────────────────────────────────┘  │
│                              │                                       │
│                              ▼                                       │
│  ┌───────────────────────────────────────────────────────────────┐  │
│  │  Layer 1: Infrastructure (Moka, Redis, Tera, DB Pool)         │  │
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
└─────────────────┘
      │
      ▼
┌──────────────────────────────────────────────┐
│  Layer 5: Business Logic                     │
│                                              │
│  [1] Check Pre-rendered RAM Cache?           │
│      ├── YES -> SERVE INSTANTLY (0 allocs)   │
│      └── NO  -> [2] Render/Pre-render        │
└──────────────────────────────────────────────┘
                        │
                        ▼ (Dynamic Data Needed)
┌─────────────────────────────────────────────────────────┐
│                    Layer 1: Data Cache                   │
│  L1 (Moka) ◄──── Cache Miss ────► L2 (Redis)            │
└─────────────────────────────────────────────────────────┘
```

---

## Performance Metrics

```
╔═══════════════════════════════════════════════════════════════════╗
║                    PERFORMANCE SUMMARY                             ║
╠═══════════════════════════════════════════════════════════════════╣
║  Peak Throughput      │  16,829.3 req/s (sustained)               ║
║  Average Latency      │  5.2ms (with full load)                   ║
║  Homepage Latency     │  <0.5ms (served from RAM)                 ║
║  Success Rate         │  100%                                     ║
║  Cache Hit Rate       │  95%+ overall (L1: 90%, L2: 75%)          ║
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

## Project Structure

```
Web-server-Report/
├── src/
│   ├── main.rs                          # Entry point
│   ├── routes/                          # Axum routes
│   └── service_islands/                 # Service Islands Architecture
│       ├── mod.rs                       # Registry & Initialization
│       ├── layer1_infrastructure/       # Cache, DB, Tera
│       ├── layer3_communication/        # Data flow, Redis Streams
│       ├── layer4_observability/        # Health checks
│       └── layer5_business_logic/       # Core Logic
│           ├── dashboard/               # Homepage & Dashboard
│           │   ├── handlers.rs          # Homepage Pre-rendering
│           │   └── ...
│           ├── crypto_reports/          # Report Generation
│           │   ├── template_orchestrator.rs # Frame Pre-rendering
│           │   ├── rendering/           # DSD / Iframe renderers
│           │   └── ...
│           └── shared/                  # Utilities
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
