# Web Server Architecture Report

## Overview
This project is a high-performance web server built with **Rust**, utilizing the **Axum** framework and **Tokio** asynchronous runtime. It employs a highly modular **Service Islands Architecture**, which organizes the system into distinct, decoupled layers for infrastructure, communication, observability, and business logic.

## Technology Stack
- **Web Framework**: Axum
- **Async Runtime**: Tokio
- **Database**: PostgreSQL (via SQLx)
- **Caching**: Multi-tier caching strategy using `moka` (L1 In-memory) and `redis` (L2 Distributed Cache), orchestrated via the `multi-tier-cache` crate.
- **Templating**: Tera
- **Concurrency**: Rayon (for CPU-intensive task parallelism), Dashmap, Threadpool

## Service Islands Architecture
The core design pattern is the **Service Islands Architecture**. The application is broken down into initialized "Islands" that process requests and maintain their own states. This ensures clear boundaries and dependency injection logic.

### Layer 1: Infrastructure Islands
The foundation of the application handling databases, caching layers, and shared UI components.
- **AppStateIsland**: Manages database connection pools and core configuration. Includes a legacy state bridge for backward compatibility.
- **SharedComponentsIsland**: Pre-loads and caches shared UI elements and Tera templates (e.g., chart modules) for optimal initial load time.
- **CacheSystemIsland**: Provides a unified interface to the multi-tier caching backend.

### Layer 3: Communication Islands
Handles inter-service and async communication, particularly for streaming data.
- **RedisStreamReader**: Subscribes to Redis streams to receive real-time data from external services. Note: WebSocket streaming and connections have been decoupled into a separate dedicated microservice.

### Layer 4: Observability Islands
Monitoring and system health.
- **HealthSystemIsland**: Aggregates health checks across all Service Islands (database connections, Redis connectivity, internal states) to provide a unified `health_check` endpoint.

### Layer 5: Business Logic Islands
The actual application features and handlers. These islands read from the cached data rather than directly invoking external APIs.
- **DashboardIsland**: Manages main dashboard rendering, pre-rendering homepage cache during initialization for maximum performance.
- **CryptoReportsIsland**: Handles data fetching and templating for cryptocurrency market reports.

## Key Design Principles
1. **Idiomatic Rust & Safety**: Strict adherence to Rust 2021 idioms. Usage of `.unwrap()` is explicitly prohibited throughout the codebase; instead, explicit error handling with `Result`, `Option`, and the `?` operator is strictly enforced.
2. **Performance First**: Extensive usage of caching layers, pre-rendering during startup, optimization profiles (`codegen-units = 1`, `lto = "fat"` for release), and parallel processing architectures.
3. **Decoupled WebSocket Service**: Real-time WebSocket connections and external API polling are externalized to distinct services. The main web server interacts with them asynchronously via Redis streams, improving stability and separating concerns.
4. **Graceful Shutdown**: Structured shutdown sequences close resources in reverse dependency order, ensuring no data loss and clean connection termination for PostgreSQL and Redis.
