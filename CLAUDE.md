# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

High-performance Rust web server for a crypto investment dashboard achieving 16,829+ RPS with 5.2ms latency. Built using **Service Islands Architecture** (5-layer separation of concerns), multi-tier caching with stampede protection, and Redis Streams for real-time data pipeline.

**Tech Stack:**
- Backend: Rust + Axum web framework
- Database: PostgreSQL with connection pooling (32 max connections)
- Caching: L1 (moka in-memory) + L2 (Redis) with Cache Stampede Protection
- Real-time: Redis Streams + WebSocket broadcasting
- Frontend: Vanilla JS + Chart.js + esbuild bundler
- APIs: Binance (primary), CoinGecko, CoinMarketCap (fallback), TAAPI.io, Finnhub (US stocks)

## Development Commands

### Rust Backend

```bash
# Development build and run
cargo run

# Production build
cargo build --release
./target/release/web-server-report

# Watch mode (requires cargo-watch)
cargo install cargo-watch
cargo watch -x run

# Run tests
cargo test

# Code quality
cargo clippy
cargo fmt

# Run specific examples
cargo run --example test_coinmarketcap_fallback
cargo run --example test_finnhub_integration
cargo run --example http_load_benchmark
```

### Frontend Assets

```bash
# Install dependencies
npm install

# Build JavaScript bundles (development)
npm run build

# Production build (minified)
npm run build:prod

# Watch mode for development
npm run build:watch

# Clean dist directory
npm run clean
```

### Testing & Benchmarks

```bash
# Integration tests
./examples/test-websocket.sh
./examples/test-memory-cleanup.sh

# Run Rust benchmarks
cargo run --example http_load_benchmark
cargo run --example performance_benchmark
```

## Service Islands Architecture (5 Layers)

The codebase uses a strict **5-layer Service Islands Architecture** with clear dependency flow from top to bottom:

```
Layer 5: Business Logic → Layer 3
Layer 4: Observability (standalone)
Layer 3: Communication → Layer 2
Layer 2: External Services → Layer 1
Layer 1: Infrastructure (foundation)
```

### Layer Responsibilities

**Layer 1 - Infrastructure** (`src/service_islands/layer1_infrastructure/`)
- `cache_system_island/`: L1 (moka) + L2 (Redis) caching + Redis Streams (XADD/XREAD)
- `app_state_island/`: Application state management
- `shared_components_island/`: Template registry, utilities
- `chart_modules_island/`: JavaScript bundling service

**Layer 2 - External Services** (`src/service_islands/layer2_external_services/`)
- `external_apis_island/`: API integration with Binance, CoinGecko, CoinMarketCap, Finnhub
  - `api_aggregator/`: Multi-source data fetching with fallback logic
  - `circuit_breaker.rs`: Fault tolerance for API failures
  - Cache-first strategy: Always cache API responses before returning

**Layer 3 - Communication** (`src/service_islands/layer3_communication/`)
- `websocket_service.rs`: Real-time WebSocket broadcasting
- `data_communication.rs`: PostgreSQL database operations
- `dashboard_communication.rs`: Dashboard data routing
- `layer2_adapters/market_data_adapter.rs`: Publishes market data to Redis Streams after caching

**Layer 4 - Observability** (`src/service_islands/layer4_observability/`)
- `health_system_island/`: Health checks, metrics, monitoring

**Layer 5 - Business Logic** (`src/service_islands/layer5_business_logic/`)
- `dashboard_island.rs`: Dashboard market data processing
- `crypto_reports_island.rs`: Report management and template orchestration
- **STRICT RULE**: Layer 5 ONLY accesses Layer 3, never directly to Layer 2

### Architecture Principles

1. **Top-Down Dependencies**: Higher layers depend on lower layers only
2. **No Skip Levels**: Layer 5 → Layer 3 → Layer 2 → Layer 1 (no shortcuts)
3. **Generic Layer 1**: Pure infrastructure, no business logic knowledge
4. **Business Layer 2**: API-specific implementations using generic Layer 1 cache
5. **Service Islands Coordination**: `src/service_islands/mod.rs` initializes all islands in correct order

## Cache System Architecture

### Multi-Tier Cache with Stampede Protection

The system implements a sophisticated 2-tier cache with request coalescing to prevent cache stampede:

**L1 Cache (moka):**
- In-memory cache: 2000 entries, 5min TTL
- Sub-millisecond response time (<1ms)
- Built-in stampede protection using `get_with()` method
- 90% hit rate

**L2 Cache (Redis):**
- Distributed cache: 1hr default TTL
- 2-5ms response time
- DashMap+Mutex request coalescing for stampede protection
- 75% hit rate
- Automatic L1 promotion on L2 hits

**Cache Stampede Protection:**
- Prevents multiple concurrent requests from hitting the same expired cache key
- DashMap tracks pending requests with Mutex locks
- Only ONE request fetches data; others wait and read from cache
- 99.6% performance improvement: 534ms → 5.2ms latency in high-concurrency

**Generic Cache Strategies** (defined in Layer 1):
```rust
CacheStrategy::ShortTerm   // 5 min - Fast-changing data (BTC price)
CacheStrategy::MediumTerm  // 1 hour - Market stats
CacheStrategy::LongTerm    // 3 hours - Technical indicators (RSI, MACD)
CacheStrategy::RealTime    // 30s - Market sentiment (Fear & Greed)
CacheStrategy::Custom(Duration) // Custom TTL
```

### Key Cache Files

- `layer1_infrastructure/cache_system_island/cache_manager.rs`: Unified cache interface
- `layer1_infrastructure/cache_system_island/l1_cache.rs`: Moka implementation
- `layer1_infrastructure/cache_system_island/l2_cache.rs`: Redis + DashMap coalescing
- `layer2_external_services/external_apis_island/api_aggregator/`: Business logic cache wrappers

## Redis Streams Integration

**Purpose**: Real-time data pipeline for external consumers (Python AI service, analytics, monitoring)

**Architecture Flow**:
```
Layer 2 External APIs → Layer 3 Market Data Adapter → Layer 1 Cache + Redis Streams → External Consumers
```

**Key Operations** (in `cache_manager.rs`):
- `publish_to_stream()`: XADD with automatic trimming (1000 entries max)
- `read_stream_latest()`: Retrieve N latest entries (newest first)
- `read_stream()`: Blocking/non-blocking stream consumption with XREAD

**Performance**:
- Publish latency: <1ms (non-blocking)
- Throughput: 10,000+ entries/second
- Consumer lag: Sub-second for Python consumers
- Auto-trimming: Maintains last 1000 entries (~200KB)

**Publishing Location**: `layer3_communication/layer2_adapters/market_data_adapter.rs`
- Automatically publishes to `market_data_stream` after Layer 2 API fetch and Layer 1 cache storage
- Non-critical: Stream failures don't affect core functionality

## Critical Development Patterns

### 1. Adding New API Endpoints

When adding endpoints that fetch external data:
```rust
// ✅ CORRECT: Use Layer 3 adapter (market_data_adapter.rs)
// This ensures cache-first + stream publishing
let data = market_data_adapter.fetch_normalized_data().await?;

// ❌ WRONG: Direct Layer 2 access from Layer 5
let data = external_apis.fetch_btc_price().await?;
```

### 2. Cache Integration Pattern

For new API calls in Layer 2:
```rust
// Always use cache_api_data helper with appropriate strategy
async fn fetch_new_data(cache_manager: &CacheManager) -> Result<Value> {
    cache_api_data(
        "cache_key",
        CacheStrategy::MediumTerm,  // Choose appropriate strategy
        || async {
            // API call logic here
        }
    ).await
}
```

### 3. Concurrency Guidelines

- Use `tokio::join!` for concurrent operations
- Use `DashMap` for thread-safe shared state
- Use `AtomicUsize` for lock-free counters
- Use `spawn_blocking` for CPU-intensive tasks (template rendering)

### 4. Service Islands Initialization Order

When modifying `src/service_islands/mod.rs`, maintain strict order:
1. Layer 1 (Infrastructure) - no dependencies
2. Layer 2 (External Services) - depends on Layer 1 cache
3. Layer 4 (Observability) - standalone
4. Layer 3 (Communication) - depends on Layer 2 + Layer 1 cache
5. Layer 5 (Business Logic) - depends ONLY on Layer 3

## Environment Configuration

Required environment variables (`.env`):
```env
DATABASE_URL=postgresql://user:pass@host/db
AUTO_UPDATE_SECRET_KEY=your_secret_key
HOST=0.0.0.0
PORT=8000
TAAPI_SECRET=your_taapi_jwt_token
REDIS_URL=redis://localhost:6379

# Optional - enables fallback support
CMC_API_KEY=your_coinmarketcap_api_key
FINNHUB_API_KEY=your_finnhub_api_key

# Development mode (enables debug logging)
DEBUG=1
```

## Key API Endpoints

### Core Routes
- `GET /` - Homepage with latest report
- `GET /crypto_report` - Latest crypto report
- `GET /crypto_report/:id` - Specific report by ID
- `GET /pdf-template/:id` - PDF-optimized report view

### Monitoring
- `GET /health` - Health check + cache metrics + Redis Streams status
- `GET /cache-stats` - Detailed L1/L2 cache statistics
- `POST /clear-cache` - Clear all cache tiers (L1+L2)

### Real-time
- `GET /ws` - WebSocket connection for live updates
- `GET /api/crypto/dashboard-summary` - Cached dashboard data (JSON) + Stream publish
- `GET /api/crypto/dashboard-summary/refresh` - Force refresh + Stream publish

## Important Files

### Entry Points
- `src/main.rs`: Server initialization + Service Islands setup
- `src/routes/mod.rs`: Axum router configuration

### Core Infrastructure
- `src/service_islands/mod.rs`: Service Islands registry and initialization
- `src/performance.rs`: Performance metrics and monitoring

### Templates
- `dashboards/home.html`: Homepage template
- `dashboards/crypto_dashboard/`: Dashboard templates

### Frontend Build
- `build.js`: esbuild configuration for JavaScript bundling
- `shared_components/market-indicators/`: Modular chart components

## Troubleshooting

### Cache Issues
- Check cache hit rates in server logs with `DEBUG=1`
- Clear cache: `curl -X POST http://localhost:8000/clear-cache`
- Monitor: `curl http://localhost:8000/cache-stats`
- L1 cache auto-expires after 5 min, L2 after 1 hour

### Database Connection
```bash
# Verify PostgreSQL connection
psql $DATABASE_URL -c "SELECT version();"
```

### Redis Streams Debugging
```bash
# Check stream info
redis-cli XINFO STREAM market_data_stream

# Read latest entry
redis-cli XREVRANGE market_data_stream + - COUNT 1

# Monitor stream length
redis-cli XLEN market_data_stream
```

### Performance Testing
- Validate 16,829+ RPS performance after major changes
- Check cache stampede protection is working (see logs for request coalescing)
- Monitor memory usage: `ps aux | grep web-server-report`

## Documentation References

Key architecture documents in `docs/`:
- `GENERIC_CACHE_ARCHITECTURE.md`: Layer separation cache design
- `BUILD_SYSTEM.md`: Frontend build pipeline details
- `FINNHUB_INTEGRATION.md`: US stock market indices integration
- `COINMARKETCAP_SETUP.md`: Fallback API configuration

## Performance Targets

When making changes, maintain these benchmarks:
- **RPS**: 16,829+ requests per second sustained
- **Latency**: 5.2ms average response time
- **Cache Hit Rate**: L1 90%, L2 75%, Overall 95%
- **Success Rate**: 100% (zero failures under load)
- **Stampede Protection**: Single API call per cache key expiration
