# ⚙️ CONFIGURATION & DEPENDENCIES SPECIFICATION

## 📊 Overview
This document specifies the complete configuration, dependency management, environment variables, and deployment patterns for the Rust crypto dashboard application. Includes both development and production configuration patterns with migration-ready modular organization.

## 📦 Cargo.toml & Dependencies

### 1. Package Configuration
```toml
[package]
name = "web-server-report"
version = "0.1.0"
edition = "2021"
default-run = "web-server-report"
license = "Apache-2.0"
authors = ["thichuong"]

[[bin]]
name = "web-server-report"
path = "src/main.rs"
```

### 2. Build Profiles

#### Development Profile (Optimized)
```toml
[profile.dev]
opt-level = 1              # Some optimization for better runtime performance
debug = true               # Keep debug info for development
lto = false               # No LTO for faster compile time
codegen-units = 16        # Parallel compilation
panic = "unwind"          # Allow debugging panics
overflow-checks = true    # Keep safety checks in dev
```

#### Release Profile (Production)
```toml
[profile.release]
opt-level = 3              # Maximum optimization
lto = "fat"               # Full Link Time Optimization
codegen-units = 1         # Single codegen unit for better optimization  
panic = "abort"           # Abort on panic (smaller binary, faster)
strip = true             # Strip debug symbols
overflow-checks = false   # Disable overflow checks in release
```

**Performance Impact**:
- **Development**: Fast compile times (~15-30s), decent runtime performance
- **Production**: Optimal runtime performance, binary size reduction (~50MB)

### 3. Core Web Framework Dependencies

#### Axum Web Framework
```toml
axum = { version = "0.6", features = ["ws"] }
```

**Features**:
- **WebSocket Support**: `ws` feature for real-time updates
- **Route Handlers**: Type-safe extractors (Path, Query, State)
- **Middleware**: Tower middleware integration
- **Static Files**: ServeDir for asset serving

#### Tokio Async Runtime
```toml
tokio = { version = "1.28", features = ["full", "sync"] }
```

**Features**:
- **Full Runtime**: Multi-threaded scheduler, I/O drivers
- **Synchronization**: Channels, mutexes, atomic operations
- **Spawn Blocking**: CPU-intensive task handling

#### HTTP Client
```toml
reqwest = { version = "0.11", features = ["json"] }
```

**Usage**:
- **External APIs**: CoinGecko, TAAPI, Fear & Greed Index
- **JSON Parsing**: Automatic serialization/deserialization
- **SSL Support**: HTTPS connections to crypto APIs

### 4. Database & Persistence

#### SQLx Database Driver
```toml
sqlx = { version = "0.8", features = ["runtime-tokio-native-tls", "postgres", "macros", "chrono"] }
```

**Features**:
- **PostgreSQL**: Native PostgreSQL protocol support
- **Async Runtime**: Tokio integration with connection pooling
- **Compile-time Checks**: SQL query validation at compile time
- **Date/Time**: Chrono integration for timestamps

**Connection Configuration**:
```rust
// Database pool initialization
let pool = PgPoolOptions::new()
    .max_connections(10)
    .connect(&database_url).await?;
```

#### Redis Caching
```toml
redis = { version = "0.32", features = ["tokio-comp", "connection-manager"] }
bb8-redis = "0.15"     # Redis connection pool for L2 cache
```

**Multi-tier Cache Setup**:
```rust
// L1: In-memory cache (Moka)
moka = { version = "0.12", features = ["future"] }

// L2: Redis cache with connection pool
let redis_client = redis::Client::open(redis_url)?;
let redis_pool = bb8_redis::RedisConnectionManager::new(redis_client);
```

### 5. Serialization & Data Handling

#### Serde Ecosystem
```toml
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = { version = "0.4", features = ["serde"] }
```

**Usage Patterns**:
```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DashboardSummary {
    pub market_cap: f64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

// JSON API responses
Json(dashboard_summary).into_response()
```

### 6. Template Engine

#### Tera Templates
```toml
tera = "1.19"
```

**Configuration**:
```rust
let mut tera = Tera::new("dashboards/**/*.html")?;
tera.autoescape_on(vec![]); // Disable auto-escaping for safe content

// Template registration with logical names
tera.add_template_file(
    "dashboards/crypto_dashboard/routes/reports/view.html",
    Some("crypto/routes/reports/view.html")
)?;
```

### 7. Performance Optimization Dependencies

#### Concurrency & Parallelism
```toml
rayon = "1.8"          # Data parallelism for CPU-intensive tasks
dashmap = "5.5"        # Thread-safe HashMap
num_cpus = "1.16"      # Detect CPU cores count
```

#### Fast Collections & Algorithms
```toml
ahash = "0.8"         # Faster hash function than default
smallvec = "1.11"     # Stack-allocated vectors for small collections
parking_lot = "0.12"  # Fast synchronization primitives (RwLock, Mutex)
```

#### Static Initialization
```toml
lazy_static = "1.4"   # Static initialization patterns
```

**Performance Optimizations**:
```rust
// Optimized HTTP client (global, reused)
use lazy_static::lazy_static;
use reqwest::Client;

lazy_static! {
    pub static ref OPTIMIZED_HTTP_CLIENT: Client = Client::builder()
        .timeout(Duration::from_secs(30))
        .pool_max_idle_per_host(10)
        .pool_idle_timeout(Duration::from_secs(90))
        .build()
        .expect("Failed to create optimized HTTP client");
}
```

#### Thread Pool Management
```toml
threadpool = "1.8"     # Dedicated thread pool for CPU tasks
```

### 8. Error Handling & Debugging

#### Error Management
```toml
anyhow = "1.0"         # Error handling with context
```

**Usage Pattern**:
```rust
use anyhow::{Result, Context};

async fn fetch_dashboard_summary(&self) -> Result<DashboardSummary> {
    let response = self.client.get(API_URL)
        .send().await
        .context("Failed to fetch dashboard data")?;
    
    Ok(response.json().await?)
}
```

#### Dependency Pinning (Railway Compatibility)
```toml
base64ct = "=1.6.0"    # Pin to avoid edition2024 requirement
futures = "0.3"        # Async utilities for concurrent operations
```

**Rationale**: Fixed version to prevent `edition2024` compilation errors on Railway platform.

## 🌍 Environment Variables

### 1. Required Environment Variables

#### Database Configuration
```bash
# PostgreSQL connection string
DATABASE_URL=postgresql://username:password@host:port/database_name

# Examples:
# Local development: postgresql://localhost:5432/crypto_reports
# Railway/production: postgresql://user:pass@host.region.provider:5432/railway
```

#### External API Keys
```bash
# TAAPI technical analysis API key
TAAPI_SECRET=your_taapi_secret_key_here

# Used for: RSI indicators, crypto technical analysis
# Get from: https://taapi.io/
```

#### Redis Cache (Optional)
```bash
# Redis connection URL (defaults to localhost)
REDIS_URL=redis://localhost:6379

# Examples:
# Local development: redis://localhost:6379
# Railway with auth: redis://default:password@host:port
```

### 2. Optional Configuration Variables

#### Server Configuration
```bash
# Server binding configuration
HOST=0.0.0.0           # Bind to all interfaces (production)
PORT=8000              # Server port (Railway auto-sets this)

# Development vs Production
HOST=127.0.0.1         # Development: localhost only
HOST=0.0.0.0           # Production: all interfaces
```

#### Logging & Debug
```bash
# Rust logging level
RUST_LOG=info                    # Production
RUST_LOG=debug                   # Development
RUST_LOG=web_server_report=debug # Application-specific

# Backtrace on panic
RUST_BACKTRACE=1        # Enable stack traces
RUST_BACKTRACE=full     # Full stack traces (debug)
```

#### Performance Tuning
```bash
# Tokio thread pool configuration
TOKIO_THREAD_STACK_SIZE=4194304  # 4MB stack size

# Cache configuration
CACHE_L1_CAPACITY=2000           # In-memory cache capacity
CACHE_L1_TTL_SECONDS=300         # L1 cache TTL (5 minutes)
CACHE_L2_TTL_SECONDS=3600        # L2 cache TTL (1 hour)
```

### 3. Environment Loading & Validation

#### Development (.env file)
```rust
use dotenvy::dotenv;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenv().ok(); // Load .env file in development
    
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env");
    let taapi_secret = env::var("TAAPI_SECRET")
        .expect("TAAPI_SECRET must be set in .env");
    let redis_url = env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://localhost:6379".to_string());
    
    // ... application initialization
}
```

#### Production Environment Detection
```rust
fn get_environment() -> Environment {
    match env::var("RAILWAY_ENVIRONMENT").as_deref() {
        Ok("production") => Environment::Production,
        _ => match env::var("DATABASE_URL").as_deref() {
            Ok(url) if url.contains("localhost") => Environment::Development,
            Ok(_) => Environment::Production,
            Err(_) => Environment::Development,
        }
    }
}
```

## 🚀 Deployment Configuration

### 1. Docker Configuration

#### Multi-stage Dockerfile
```dockerfile
FROM rust:1.82-slim as builder
WORKDIR /app

# Copy and cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm src/main.rs

# Build application
COPY src ./src
COPY dashboards ./dashboards
COPY shared_components ./shared_components
COPY shared_assets ./shared_assets
RUN touch src/main.rs && cargo build --release

# Runtime stage
FROM ubuntu:22.04
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder --chmod=755 /app/target/release/web-server-report ./web-server-report
COPY --from=builder /app/dashboards ./dashboards
COPY --from=builder /app/shared_components ./shared_components
COPY --from=builder /app/shared_assets ./shared_assets

# Environment variables with defaults
ENV RUST_LOG=info \
    RUST_BACKTRACE=1 \
    DATABASE_URL="" \
    TAAPI_SECRET="" \
    REDIS_URL="redis://localhost:6379" \
    HOST="0.0.0.0" \
    PORT="8000"

EXPOSE 8000
CMD ["./web-server-report"]
```

### 2. Railway Deployment

#### railway.json Configuration
```json
{
  "build": {
    "builder": "DOCKERFILE",
    "dockerfilePath": "Dockerfile"
  },
  "deploy": {
    "healthcheckPath": "/health",
    "healthcheckTimeout": 300,
    "sleepApplication": false,
    "restartPolicyType": "ON_FAILURE",
    "restartPolicyMaxRetries": 10
  }
}
```

#### Nixpacks Alternative
```toml
# nixpacks.toml
[variables]
RUST_LOG = "info"
TOKIO_THREAD_STACK_SIZE = "4194304"

[phases.build]
cmd = "cargo build --release --locked"

[phases.install]
cmd = "cp target/release/web-server-report /app/"

[start]
cmd = "./web-server-report"
```

### 3. Health Check Configuration

#### Application Health Check
```rust
// GET /health endpoint
pub async fn health(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let start_time = Instant::now();
    
    // Test database connectivity
    let db_healthy = sqlx::query("SELECT 1")
        .fetch_one(&state.db).await.is_ok();
    
    // Test Redis connectivity  
    let redis_healthy = state.cache_manager.health_check().await.overall_healthy;
    
    // Test external API connectivity
    let api_healthy = test_ssl_connectivity().await;
    
    let response_time = start_time.elapsed().as_millis() as u64;
    
    Json(json!({
        "status": if db_healthy && redis_healthy { "healthy" } else { "unhealthy" },
        "services": {
            "database": db_healthy,
            "redis": redis_healthy,
            "external_apis": api_healthy
        },
        "response_time_ms": response_time,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}
```

#### Docker Health Check
```dockerfile
# Health check (not supported in distroless)
# Railway handles external health checks via HTTP
HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=3 \
  CMD curl -f http://localhost:8000/health || exit 1
```

## 🎛️ Application State Configuration

### 1. AppState Initialization
```rust
impl AppState {
    pub async fn new(
        database_url: &str, 
        redis_url: &str, 
        taapi_secret: String
    ) -> Result<Self, anyhow::Error> {
        // Database pool with optimized settings
        let db = PgPoolOptions::new()
            .max_connections(10)
            .acquire_timeout(Duration::from_secs(30))
            .connect(database_url).await?;
        
        // Unified cache manager (L1 + L2)
        let cache_manager = Arc::new(
            CacheManager::new(redis_url).await?
        );
        
        // Data service with API clients
        let data_service = DataService::with_cache_manager(
            taapi_secret,
            cache_manager.clone()
        );
        
        // WebSocket service
        let websocket_service = Arc::new(
            WebSocketService::new(data_service.clone()).await
        );
        
        // Template engine with all required templates
        let mut tera = Tera::new("dashboards/**/*.html")?;
        tera.autoescape_on(vec![]);
        
        // Register templates with logical names
        register_templates(&mut tera)?;
        
        Ok(Self {
            db,
            cache_manager,
            data_service,
            report_cache: MultiLevelCache::new(1000, Duration::from_secs(3600)),
            chart_modules_cache: RwLock::new(None),
            cached_latest_id: AtomicUsize::new(0),
            tera,
            websocket_service,
            metrics: Arc::new(PerformanceMetrics::default()),
            request_counter: AtomicUsize::new(0),
            start_time: Instant::now(),
        })
    }
    
    // Cache priming at startup
    pub async fn prime_cache(&self) {
        println!("🔥 Priming cache at startup...");
        
        // Prime latest report
        if let Ok(Some(_)) = self.fetch_latest_report().await {
            println!("✅ Latest report cached");
        }
        
        // Prime chart modules
        let _ = utils::get_chart_modules_content(&self.chart_modules_cache).await;
        println!("✅ Chart modules cached");
        
        // Prime dashboard summary
        if let Ok(_) = self.data_service.fetch_dashboard_summary().await {
            println!("✅ Dashboard summary cached");
        }
    }
}
```

### 2. Template Registration
```rust
fn register_templates(tera: &mut Tera) -> Result<(), anyhow::Error> {
    // Shared components
    tera.add_template_file(
        "shared_components/theme_toggle.html",
        Some("crypto/components/theme_toggle.html")
    )?;
    tera.add_template_file(
        "shared_components/language_toggle.html", 
        Some("crypto/components/language_toggle.html")
    )?;
    
    // Crypto dashboard templates
    tera.add_template_file(
        "dashboards/crypto_dashboard/routes/reports/view.html",
        Some("crypto/routes/reports/view.html")
    )?;
    tera.add_template_file(
        "dashboards/crypto_dashboard/routes/reports/pdf.html",
        Some("crypto/routes/reports/pdf.html")
    )?;
    tera.add_template_file(
        "dashboards/crypto_dashboard/routes/reports/list.html",
        Some("crypto/routes/reports/list.html")
    )?;
    
    Ok(())
}
```

## 🔧 Development Tools & Scripts

### 1. Development Scripts

#### Local Development
```bash
#!/bin/bash
# scripts/dev.sh

# Load environment
source .env

# Check dependencies
echo "🔍 Checking dependencies..."
which cargo || { echo "❌ Rust not installed"; exit 1; }
which psql || echo "⚠️  PostgreSQL client not found"
which redis-cli || echo "⚠️  Redis client not found"

# Start services (if needed)
echo "🚀 Starting development server..."
export RUST_LOG=debug
export RUST_BACKTRACE=1

# Hot reload with cargo-watch
if command -v cargo-watch &> /dev/null; then
    cargo watch -x "run --bin web-server-report"
else
    cargo run --bin web-server-report
fi
```

#### Database Testing
```bash
#!/bin/bash
# scripts/test_db_connection.sh

echo "🔍 Testing database connectivity..."
source .env

# Extract connection details
HOST=$(echo $DATABASE_URL | grep -oP '(?<=@)[^:/]+(?=:)')
PORT=$(echo $DATABASE_URL | grep -oP '(?<=:)[0-9]+(?=/)')

echo "Host: $HOST, Port: $PORT"

# Test TCP connection
if timeout 10 nc -z $HOST $PORT; then
    echo "✅ TCP connection successful"
    
    # Test SQL query
    if echo "SELECT version();" | psql $DATABASE_URL; then
        echo "✅ Database query successful"
    else
        echo "❌ Database query failed"
    fi
else
    echo "❌ Cannot reach database"
fi
```

### 2. Build Optimization Scripts

#### Performance Testing
```bash
#!/bin/bash
# scripts/performance_benchmark.sh

echo "🏃 Running performance benchmarks..."

# Build optimized version
cargo build --release

# Run load test
if command -v wrk &> /dev/null; then
    echo "📊 Load testing with wrk..."
    ./target/release/web-server-report &
    SERVER_PID=$!
    
    sleep 5  # Wait for server startup
    
    # Test endpoints
    wrk -t12 -c400 -d30s --timeout 30s http://localhost:8000/health
    wrk -t12 -c400 -d30s --timeout 30s http://localhost:8000/crypto_report
    wrk -t12 -c400 -d30s --timeout 30s http://localhost:8000/api/crypto/dashboard-summary
    
    kill $SERVER_PID
else
    echo "⚠️  wrk not available, install for load testing"
fi
```

## 🎯 Migration-Ready Configuration

### 1. Feature-Based Configuration Structure
**Target Organization**:
```
configs/
├── database/
│   ├── connection.rs     # Database configuration
│   ├── migrations.rs     # Migration management
│   └── pools.rs         # Connection pooling
├── cache/
│   ├── redis.rs         # Redis configuration
│   ├── memory.rs        # In-memory cache config
│   └── strategies.rs    # Cache strategies
├── external_apis/
│   ├── crypto_apis.rs   # CoinGecko, TAAPI config
│   ├── rate_limits.rs   # Rate limiting config
│   └── circuit_breaker.rs # Circuit breaker config
├── templates/
│   ├── registration.rs  # Template registration
│   └── contexts.rs      # Template contexts
└── deployment/
    ├── docker.rs        # Docker configuration
    ├── railway.rs       # Railway-specific config
    └── environment.rs   # Environment detection
```

### 2. Configuration Traits
```rust
pub trait DatabaseConfig {
    fn connection_url(&self) -> &str;
    fn max_connections(&self) -> u32;
    fn timeout(&self) -> Duration;
}

pub trait CacheConfig {
    fn l1_capacity(&self) -> usize;
    fn l1_ttl(&self) -> Duration;
    fn l2_ttl(&self) -> Duration;
    fn redis_url(&self) -> Option<&str>;
}

pub trait ApiConfig {
    fn timeout(&self) -> Duration;
    fn rate_limit(&self) -> Option<RateLimit>;
    fn circuit_breaker(&self) -> Option<CircuitBreakerConfig>;
}
```

### 3. Environment-Specific Configurations
```rust
#[derive(Debug, Clone)]
pub enum Environment {
    Development,
    Testing,
    Production,
}

pub struct AppConfig {
    pub environment: Environment,
    pub database: DatabaseConfiguration,
    pub cache: CacheConfiguration,
    pub apis: ExternalApiConfiguration,
    pub server: ServerConfiguration,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let environment = Environment::detect()?;
        
        Ok(Self {
            environment: environment.clone(),
            database: DatabaseConfiguration::for_env(&environment)?,
            cache: CacheConfiguration::for_env(&environment)?,
            apis: ExternalApiConfiguration::for_env(&environment)?,
            server: ServerConfiguration::for_env(&environment)?,
        })
    }
}
```

---

**📝 Generated**: August 20, 2025  
**🔄 Version**: 1.0  
**📦 Dependencies**: 20+ crates with optimized build profiles  
**🎯 Migration Target**: `configs/` module with trait-based configuration management
