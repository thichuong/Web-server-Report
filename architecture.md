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

## Key Design Principles
1. **Idiomatic Rust & Safety**: Strict adherence to Rust 2021 idioms. Usage of `.unwrap()` is explicitly prohibited throughout the codebase; instead, explicit error handling with `Result`, `Option`, and the `?` operator is strictly enforced.
2. **Performance First**: Extensive usage of caching layers, pre-rendering during startup, optimization profiles (`codegen-units = 1`, `lto = "fat"` for release), and parallel processing architectures.
3. **Decoupled WebSocket Service**: Real-time WebSocket connections and external API polling are externalized to distinct services. The main web server interacts with them asynchronously via Redis streams, improving stability and separating concerns.
4. **Graceful Shutdown**: Structured shutdown sequences close resources in reverse dependency order, ensuring no data loss and clean connection termination for PostgreSQL and Redis.
