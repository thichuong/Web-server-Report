# Web Server Architecture Report

## Overview
This project is a high-performance web server built with **Rust**, utilizing the **Axum** framework and **Tokio** asynchronous runtime. It employs a highly modular **Service Islands Architecture**, which organizes the system into distinct, decoupled layers for infrastructure, communication, observability, and business logic.

## Technology Stack
- **Web Framework**: Axum
- **Async Runtime**: Tokio
- **Database**: PostgreSQL (via SQLx)
- **Caching**: Specialized multi-tier caching strategy using `OnceCell` (L0 RAM), `moka` (L1 In-memory), and `redis` (L2 Distributed), orchestrated via the `multi-tier-cache` crate.
- **Concurrency**: Rayon (for CPU-intensive task parallelism), Dashmap, Threadpool

## Performance Summary
- **Peak Throughput**: 44,700+ RPS (Apache Benchmark)
- **Mean Latency**: 11ms (under heavy load)
- **Optimization**: Zero-Allocation Pre-rendering & Immediate Route-Level Cache Checks

## Core Services Architecture
The application is broken down into initialized distinct modules ("services") that process requests and maintain their own states. This ensures clear boundaries and dependency separation.

### Infrastructure & Communication (`src/stream.rs`, `src/dto/`, `src/routes/`)
The foundation of the application handling databases, caching layers, shared UI components, and HTTP routing.
- **State Management**: Manages database connection pools and core configuration (`AppState`).
- **RedisStreamReader**: Subscribes to Redis streams to receive real-time data from external services (`stream.rs`). Note: WebSocket streaming and connections have been decoupled into a separate dedicated microservice.

### Request Handling (`src/handlers/`)
Functions responsible for accepting HTTP requests, validating inputs via DTOs, and calling the appropriate service layer functions.

### Core Business Logic (`src/services/`)
The actual application features and services. These endpoints read from the cached data rather than directly invoking external APIs.
- **Dashboard Service**: Manages main dashboard rendering, pre-rendering homepage cache during initialization for maximum performance (`src/services/dashboard.rs`).
- **Crypto Reports Service**: Handles data fetching and templating for cryptocurrency market reports (`src/services/crypto_reports/`).
- **Shared Utilities**: Cross-domain utilities for SEO, response building, security, and gzip compression (`src/services/shared/`).

## Caching Architecture
The system employs a 4-layer caching strategy to ensure sub-millisecond responses for hot paths:

1. **L0: Immediate Route Cache (RAM)**:
   - Uses `OnceCell` or `ArcSwap` for pre-rendered content (e.g., Homepage).
   - Checked immediately in `src/routes/` to bypass all service orchestration.
2. **L1: In-Memory Cache (Moka)**:
   - LRU cache with 1000-entry capacity for dynamic reports and list pages.
   - Handles localized hot-data access with microsecond latency.
3. **L2: Distributed Cache (Redis)**:
   - Shared persistent cache for cross-instance data consistency.
   - Stores compressed GZIP payloads to minimize network IO.
4. **L3: Stream Cache**:
   - Cache-first pattern on Redis Streams to avoid redundant `XREAD` operations.

## Key Design Principles
1. **Idiomatic Rust & Safety**: Strict adherence to Rust 2021 idioms. Usage of `.unwrap()` is explicitly prohibited throughout the codebase; instead, explicit error handling with `Result`, `Option`, and the `?` operator is strictly enforced.
2. **Performance First**: Extensive usage of caching layers, immediate route-level cache checks, pre-rendering during startup, optimization profiles (`codegen-units = 1`, `lto = "fat"` for release), and parallel processing architectures.
3. **Layered Decoupling**: Adherence to the **Service Islands Architecture**. Each layer (Route, Handler, Service, Communication, Infrastructure) has a specific responsibility and strictly defined communication patterns.
4. **Decoupled WebSocket Service**: Real-time WebSocket connections and external API polling are externalized to distinct services. The main web server interacts with them asynchronously via Redis streams, improving stability and separating concerns.
5. **Graceful Shutdown**: Structured shutdown sequences close resources in reverse dependency order, ensuring no data loss and clean connection termination for PostgreSQL and Redis.
