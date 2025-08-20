# 🌐 API INTERFACES & HTTP ROUTES SPECIFICATION

## 📊 Overview
This document specifies the complete HTTP API interface system with route definitions, handler signatures, request/response formats, and middleware integration patterns using Axum web framework.

## 🚏 Route Architecture

### 1. Router Configuration
**Purpose**: Centralized route definition with static file serving and API endpoint organization

```rust
use axum::{routing::get, Router};
use tower_http::services::ServeDir;
use std::sync::Arc;
use crate::{handlers::*, state::AppState};

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        // Static file serving routes
        .nest_service("/crypto_dashboard/shared", ServeDir::new("dashboards/crypto_dashboard/shared"))
        .nest_service("/crypto_dashboard/routes", ServeDir::new("dashboards/crypto_dashboard/routes"))
        .nest_service("/crypto_dashboard/assets", ServeDir::new("dashboards/crypto_dashboard/assets"))
        .nest_service("/crypto_dashboard/pages", ServeDir::new("dashboards/crypto_dashboard/pages"))
        
        .nest_service("/stock_dashboard/shared", ServeDir::new("dashboards/stock_dashboard/shared"))
        .nest_service("/stock_dashboard/routes", ServeDir::new("dashboards/stock_dashboard/routes"))
        .nest_service("/stock_dashboard/assets", ServeDir::new("dashboards/stock_dashboard/assets"))
        .nest_service("/stock_dashboard/pages", ServeDir::new("dashboards/stock_dashboard/pages"))
        
        .nest_service("/shared_components", ServeDir::new("shared_components"))
        .nest_service("/shared_assets", ServeDir::new("shared_assets"))
        
        // Legacy compatibility routes
        .nest_service("/assets", ServeDir::new("dashboards/crypto_dashboard/assets"))
        .nest_service("/static", ServeDir::new("dashboards/crypto_dashboard/assets"))
        
        // Application routes (detailed below)
        // Health & Monitoring
        // Main Application
        // WebSocket & API
        
        .with_state(state)
}
```

## 🏥 Health & Monitoring Endpoints

### 1. Health Check Endpoint
```rust
// Route: GET /health
pub async fn health(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let start_time = Instant::now();
    
    let request_count = state.request_counter.load(Ordering::Relaxed);
    let report_cache_stats = state.report_cache.stats().await;
    let latest_id = state.cached_latest_id.load(Ordering::Relaxed);
    
    // Test SSL connectivity to external APIs
    let ssl_check = test_ssl_connectivity().await;
    
    // Get unified cache stats
    let cache_stats = state.cache_manager.stats().await;
    let cache_health = state.cache_manager.health_check().await;
    
    // Record performance metrics
    let response_time = start_time.elapsed().as_millis() as u64;
    state.metrics.record_request(response_time);
    
    Json(serde_json::json!({
        "status": "healthy", 
        "message": "Crypto Dashboard Rust server with Unified Cache Manager",
        "ssl_status": ssl_check,
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "metrics": {
            "total_requests": request_count,
            "cache_size": report_cache_stats.entries,
            "latest_report_id": latest_id,
            "available_cpus": num_cpus::get(),
            "thread_pool_active": true,
            "avg_response_time_ms": state.metrics.avg_response_time(),
            "cache_hit_rate": state.report_cache.hit_rate()
        },
        "cache_system": {
            "type": "unified_multi_tier",
            "l1_entries": cache_stats.l1_entry_count,
            "l1_hit_count": cache_stats.l1_hit_count,
            "l1_miss_count": cache_stats.l1_miss_count,
            "l1_hit_rate": cache_stats.l1_hit_rate,
            "l2_healthy": cache_health.l2_healthy,
            "overall_healthy": cache_health.overall_healthy
        }
    }))
}
```

**Response Format**:
```json
{
  "status": "healthy",
  "message": "Crypto Dashboard Rust server with Unified Cache Manager",
  "ssl_status": { "external_apis": "connected" },
  "timestamp": "2025-08-20T10:30:00Z",
  "metrics": {
    "total_requests": 15678,
    "cache_size": 145,
    "latest_report_id": 1456,
    "available_cpus": 8,
    "thread_pool_active": true,
    "avg_response_time_ms": 12.4,
    "cache_hit_rate": 89.7
  },
  "cache_system": {
    "type": "unified_multi_tier",
    "l1_entries": 145,
    "l1_hit_count": 2847,
    "l1_miss_count": 312,
    "l1_hit_rate": 90.1,
    "l2_healthy": true,
    "overall_healthy": true
  }
}
```

### 2. Performance Metrics Endpoint
```rust
// Route: GET /metrics
pub async fn performance_metrics(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let request_count = state.request_counter.load(Ordering::Relaxed);
    
    Json(serde_json::json!({
        "performance": {
            "total_requests": request_count,
            "avg_response_time_ms": state.metrics.avg_response_time(),
            "cache_size": state.report_cache.stats().await.entries,
            "cache_hit_rate": state.report_cache.hit_rate(),
        },
        "system": {
            "available_cpus": num_cpus::get(),
            "thread_pool_active": true,
        }
    }))
}
```

### 3. Cache Administration Endpoints
```rust
// Route: GET /admin/cache/stats
pub async fn cache_stats(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let cache_stats = state.cache_manager.stats().await;
    let cache_health = state.cache_manager.health_check().await;
    
    Json(serde_json::json!({
        "cache_system": "Unified Multi-Tier (L1: In-Memory + L2: Redis)",
        "l1_cache": {
            "type": "moka::future::Cache",
            "entry_count": cache_stats.l1_entry_count,
            "hit_count": cache_stats.l1_hit_count,
            "miss_count": cache_stats.l1_miss_count,
            "hit_rate_percent": cache_stats.l1_hit_rate,
            "max_capacity": 2000,
            "ttl_seconds": 300,
            "healthy": cache_health.l1_healthy
        },
        "l2_cache": {
            "type": "Redis",
            "ttl_seconds": 3600,
            "healthy": cache_health.l2_healthy,
            "status": if cache_health.l2_healthy { "connected" } else { "disconnected" }
        },
        "report_cache": {
            "entry_count": state.report_cache.stats().await.entries,
            "hit_rate_percent": state.report_cache.hit_rate(),
            "latest_report_id": state.cached_latest_id.load(Ordering::Relaxed)
        },
        "overall_health": cache_health.overall_healthy
    }))
}

// Route: GET /admin/cache/clear
pub async fn clear_cache(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    // Clear L1 cache
    state.cache_manager.clear_all().await.ok();
    
    // Clear report cache
    state.report_cache.clear_all().await;
    
    Json(serde_json::json!({
        "status": "success",
        "message": "All cache levels cleared",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}
```

## 📊 Main Application Routes

### 1. Homepage Route
```rust
// Route: GET /
pub async fn homepage(State(state): State<Arc<AppState>>) -> Response {
    // Increment request counter
    state.request_counter.fetch_add(1, Ordering::Relaxed);
    
    // Async file reading to avoid blocking thread
    match fs::read_to_string("dashboards/home.html").await {
        Ok(s) => Html(s).into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "Home page not found").into_response(),
    }
}
```

### 2. Crypto Report Routes

#### Latest Crypto Report
```rust
// Route: GET /crypto_report
pub async fn crypto_index(State(state): State<Arc<AppState>>) -> Response {
    // Increment request counter
    state.request_counter.fetch_add(1, Ordering::Relaxed);

    // L1 Cache check (atomic latest_id)
    let latest_id = state.cached_latest_id.load(Ordering::Relaxed) as i32;
    if let Some(cached) = state.report_cache.get(&latest_id).await {
        let chart_modules_content = utils::get_chart_modules_content(&state.chart_modules_cache).await;
        
        match render_crypto_template(
            &state.tera,
            "crypto/routes/reports/view.html",
            &cached,
            &chart_modules_content,
            None
        ).await {
            Ok(html) => return create_cached_response(html, "hit"),
            Err(_) => {
                // Template rendering failed, continue to DB fetch
                println!("⚠️ Failed to render from L1 cache, falling back to DB");
            }
        }
    }

    // L2 Cache check with fixed key "crypto_latest_report"
    if let Ok(Some(cached_report)) = state.cache_manager.get::<Report>("crypto_latest_report").await {
        // Promote to L1 + render
        state.report_cache.insert(cached_report.id, cached_report.clone()).await;
        
        let chart_modules_content = utils::get_chart_modules_content(&state.chart_modules_cache).await;
        
        match render_crypto_template(
            &state.tera,
            "crypto/routes/reports/view.html",
            &cached_report,
            &chart_modules_content,
            None
        ).await {
            Ok(html) => return create_cached_response(html, "l2-hit"),
            Err(_) => {
                println!("⚠️ Failed to render from L2 cache, falling back to DB");
            }
        }
    }

    // Both L1 and L2 cache miss: fetch from DB
    println!("🔍 L1+L2 Cache miss for crypto_index - fetching from DB");

    // DB fetch with concurrent operations
    let db_fut = fetch_and_cache_latest_report(&state);
    let chart_fut = utils::get_chart_modules_content(&state.chart_modules_cache);

    let (db_res, chart_modules_content) = tokio::join!(db_fut, chart_fut);

    match db_res {
        Ok(Some(report)) => {
            // Template rendering with helper function
            match render_crypto_template(
                &state.tera,
                "crypto/routes/reports/view.html",
                &report,
                &chart_modules_content,
                None
            ).await {
                Ok(html) => create_cached_response(html, "miss"),
                Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Template render error").into_response(),
            }
        }
        Ok(None) => {
            // No reports exist - render empty state
            let empty_report = serde_json::json!({
                "html_content": "",
                "html_content_en": "",
                "css_content": "",
                "js_content": ""
            });
            
            // Empty state template rendering logic...
            create_cached_response("Empty state HTML".to_string(), "empty")
        }
        Err(e) => {
            eprintln!("DB error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response()
        }
    }
}
```

#### Specific Crypto Report
```rust
// Route: GET /crypto_report/:id
pub async fn crypto_view_report(Path(id): Path<i32>, State(state): State<Arc<AppState>>) -> Response {
    // Increment request counter
    state.request_counter.fetch_add(1, Ordering::Relaxed);

    // L1 Cache check
    if let Some(cached) = state.report_cache.get(&id).await {
        let chart_modules_content = utils::get_chart_modules_content(&state.chart_modules_cache).await;
        
        match render_crypto_template(
            &state.tera,
            "crypto/routes/reports/view.html",
            &cached,
            &chart_modules_content,
            None
        ).await {
            Ok(html) => return create_cached_response(html, "hit"),
            Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Template render error").into_response(),
        }
    }

    // Cache miss: Use helper function
    match fetch_and_cache_report_by_id(&state, id).await {
        Ok(Some(report)) => {
            let chart_modules_content = utils::get_chart_modules_content(&state.chart_modules_cache).await;
            
            match render_crypto_template(
                &state.tera,
                "crypto/routes/reports/view.html",
                &report,
                &chart_modules_content,
                None
            ).await {
                Ok(html) => create_cached_response(html, "miss"),
                Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Template render error").into_response(),
            }
        }
        Ok(None) => (StatusCode::NOT_FOUND, "Report not found").into_response(),
        Err(e) => {
            eprintln!("DB error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response()
        }
    }
}
```

#### PDF Template Route
```rust
// Route: GET /pdf-template/:id
pub async fn pdf_template(Path(id): Path<i32>, State(state): State<Arc<AppState>>) -> Response {
    // Increment request counter
    state.request_counter.fetch_add(1, Ordering::Relaxed);
    
    // Fast path: check L1 cache (report_cache)
    if let Some(cached) = state.report_cache.get(&id).await {
        let chart_modules_content = utils::get_chart_modules_content(&state.chart_modules_cache).await;

        // Template rendering with PDF-specific context
        let tera = state.tera.clone();
        let cached_inner = cached.clone();
        let chart_content_clone = chart_modules_content.clone();
        
        let render_result = tokio::task::spawn_blocking(move || {
            let mut context = Context::new();
            context.insert("report", &cached_inner);
            context.insert("chart_modules_content", &chart_content_clone);

            // Formatted created date in UTC+7 timezone for display
            let created_display = (cached_inner.created_at + chrono::Duration::hours(7))
                .format("%d-%m-%Y %H:%M").to_string();
            context.insert("created_at_display", &created_display);

            tera.render("crypto/routes/reports/pdf.html", &context)
        }).await;

        match render_result {
            Ok(Ok(html)) => return Html(html).into_response(),
            Ok(Err(e)) => {
                eprintln!("Template render error: {:#?}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, "Template render error").into_response();
            }
            Err(e) => {
                eprintln!("Task join error: {:#?}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, "Internal error").into_response();
            }
        }
    }

    // Cache miss: fetch DB and render
    // Similar logic as crypto_view_report but with PDF template
    // ... implementation details ...
}
```

#### Report List with Pagination
```rust
// Route: GET /crypto_reports_list
pub async fn report_list(
    Query(params): Query<HashMap<String, String>>, 
    State(state): State<Arc<AppState>>
) -> Response {
    let page: i64 = params.get("page").and_then(|p| p.parse().ok()).unwrap_or(1);
    let per_page: i64 = 10;
    let offset = (page - 1) * per_page;

    // Database query with pagination
    let count_result = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM crypto_report")
        .fetch_one(&state.db).await;
    
    let items_result = sqlx::query_as::<_, ReportListItem>(
        "SELECT id, created_at FROM crypto_report ORDER BY created_at DESC LIMIT $1 OFFSET $2"
    )
    .bind(per_page)
    .bind(offset)
    .fetch_all(&state.db).await;

    match (count_result, items_result) {
        (Ok(total), Ok(items)) => {
            let pages = (total + per_page - 1) / per_page;
            
            // Pagination calculation
            let start_page = std::cmp::max(1, page - 2);
            let end_page = std::cmp::min(pages, start_page + 4);
            let page_numbers: Vec<i64> = (start_page..=end_page).collect();
            
            let display_start = if total == 0 { 0 } else { offset + 1 };
            let display_end = offset + (items.len() as i64);

            let reports = json!({
                "items": items,
                "total": total,
                "per_page": per_page,
                "page": page,
                "pages": pages,
                "has_prev": page > 1,
                "has_next": page < pages,
                "prev_num": if page > 1 { page - 1 } else { 1 },
                "next_num": if page < pages { page + 1 } else { pages },
                "page_numbers": page_numbers,
                "display_start": display_start,
                "display_end": display_end,
            });

            // Template rendering in spawn_blocking
            let tera = state.tera.clone();
            let reports_clone = reports.clone();
            
            let render_result = tokio::task::spawn_blocking(move || {
                let mut context = Context::new();
                context.insert("reports", &reports_clone);
                tera.render("crypto/routes/reports/list.html", &context)
            }).await;

            match render_result {
                Ok(Ok(html)) => Html(html).into_response(),
                Ok(Err(e)) => {
                    eprintln!("Template render error: {:#?}", e);
                    // Debug context output for troubleshooting
                    match serde_json::to_string_pretty(&reports) {
                        Ok(s) => eprintln!("reports context: {}", s),
                        Err(_) => eprintln!("Failed to serialize reports context for debugging"),
                    }
                    (StatusCode::INTERNAL_SERVER_ERROR, "Template render error").into_response()
                }
                Err(e) => {
                    eprintln!("Task join error: {:#?}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, "Internal error").into_response()
                }
            }
        }
        _ => (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response()
    }
}
```

## 🔌 WebSocket & API Routes

### 1. WebSocket Connection
```rust
// Route: GET /ws
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| async move {
        state.websocket_service.handle_websocket(socket).await;
    })
}
```

### 2. Dashboard API Endpoints

#### Primary Dashboard Summary
```rust
// Route: GET /api/crypto/dashboard-summary
pub async fn api_dashboard_summary(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let start_time = Instant::now();
    
    match state.data_service.fetch_dashboard_summary().await {
        Ok(summary) => {
            let response_time = start_time.elapsed().as_millis() as u64;
            state.metrics.record_request(response_time);
            state.request_counter.fetch_add(1, Ordering::Relaxed);

            // Return raw summary object (frontend expects top-level fields)
            Json(summary).into_response()
        },
        Err(e) => {
            eprintln!("❌ Dashboard summary API error: {}", e);

            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({
                    "error": "Failed to fetch dashboard data",
                    "details": e.to_string()
                }))
            ).into_response()
        }
    }
}
```

#### Cached Dashboard Summary (Legacy)
```rust
// Route: GET /api/crypto/dashboard-summary/cached
pub async fn dashboard_summary_api(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match state.websocket_service.get_dashboard_data_with_fallback().await {
        Ok(data) => Json(data).into_response(),
        Err(e) => {
            eprintln!("Failed to fetch dashboard data with fallback: {}", e);
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({
                    "error": "Service temporarily unavailable",
                    "message": format!("Unable to fetch dashboard data: {}", e),
                    "suggestion": "Dashboard data may be temporarily unavailable due to API rate limits. Please try again in a few minutes."
                }))
            ).into_response()
        }
    }
}
```

#### Force Dashboard Refresh
```rust
// Route: GET /api/crypto/dashboard-summary/refresh
pub async fn force_refresh_dashboard(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match state.websocket_service.force_update_dashboard().await {
        Ok(summary) => {
            println!("✅ Dashboard force refresh successful");
            Json(summary).into_response()
        },
        Err(e) => {
            eprintln!("❌ Dashboard force refresh failed: {}", e);
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({
                    "error": "Failed to refresh dashboard data",
                    "details": e.to_string(),
                    "retry_after": 60
                }))
            ).into_response()
        }
    }
}
```

#### Rate Limiting Status
```rust
// Route: GET /api/crypto/rate-limit-status
pub async fn api_rate_limit_status(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let rate_limit_status = state.data_service.get_rate_limit_status();
    
    Json(json!({
        "rate_limit_status": rate_limit_status,
        "timestamp": chrono::Utc::now(),
        "server_info": {
            "total_requests": state.request_counter.load(Ordering::Relaxed),
            "uptime_seconds": state.start_time.elapsed().as_secs()
        }
    })).into_response()
}
```

### 3. Asset Serving Route
```rust
// Route: GET /shared_assets/js/chart_modules.js
pub async fn serve_chart_modules(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let content = utils::get_chart_modules_content(&state.chart_modules_cache).await;
    axum::response::Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/javascript")
        .header("cache-control", "public, max-age=3600")
        .body(content)
        .unwrap()
}
```

## 📋 Handler Signature Patterns

### 1. Basic Extractors
```rust
// State extraction (most common)
pub async fn handler_name(State(state): State<Arc<AppState>>) -> impl IntoResponse

// Path parameter extraction
pub async fn handler_name(Path(id): Path<i32>, State(state): State<Arc<AppState>>) -> Response

// Query parameter extraction
pub async fn handler_name(
    Query(params): Query<HashMap<String, String>>, 
    State(state): State<Arc<AppState>>
) -> Response

// WebSocket upgrade
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse
```

### 2. Response Types
```rust
// JSON responses
-> impl IntoResponse  // For Json(data).into_response()

// HTML responses  
-> Response           // For Html(content).into_response()

// Custom responses with headers
-> Response           // For Response::builder().header().body().unwrap()
```

### 3. Error Handling Patterns
```rust
// Template rendering errors
match render_result {
    Ok(Ok(html)) => create_cached_response(html, "status"),
    Ok(Err(e)) => {
        eprintln!("Template render error: {:#?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Template render error").into_response()
    }
    Err(e) => {
        eprintln!("Task join error: {:#?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Internal error").into_response()
    }
}

// API errors with detailed responses
Err(e) => {
    eprintln!("❌ API error: {}", e);
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(json!({
            "error": "Service description",
            "details": e.to_string(),
            "suggestion": "User-friendly suggestion"
        }))
    ).into_response()
}

// Database errors
Err(e) => {
    eprintln!("DB error: {}", e);
    (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response()
}
```

## 🔧 Middleware & State Integration

### 1. AppState Dependencies
```rust
// Required AppState fields for handlers:
pub struct AppState {
    pub db: PgPool,                          // Database operations
    pub cache_manager: Arc<CacheManager>,    // Unified caching
    pub data_service: DataService,           // External API integration
    pub report_cache: MultiLevelCache<i32, Report>, // Report caching
    pub chart_modules_cache: RwLock<Option<String>>, // Chart JS caching
    pub cached_latest_id: AtomicUsize,       // Latest report ID
    pub tera: Tera,                          // Template engine
    pub websocket_service: Arc<WebSocketService>, // WebSocket integration
    pub metrics: Arc<PerformanceMetrics>,    // Performance tracking
    pub request_counter: AtomicUsize,        // Request counting
    pub start_time: Instant,                 // Server uptime tracking
}
```

### 2. Common Middleware Patterns
```rust
// Request counting (in handlers)
state.request_counter.fetch_add(1, Ordering::Relaxed);

// Performance metrics recording
let start_time = Instant::now();
// ... handler logic ...
let response_time = start_time.elapsed().as_millis() as u64;
state.metrics.record_request(response_time);

// Cache header injection
fn create_cached_response(html: String, cache_status: &str) -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "text/html; charset=utf-8")
        .header("cache-control", "public, max-age=300")
        .header("x-cache-status", cache_status)
        .body(html)
        .unwrap()
}
```

## 🎯 Migration Considerations

### Feature-Based Route Organization
When migrating to feature-based architecture:

```rust
// Current: Monolithic route definition
.route("/crypto_report", get(crypto_index))
.route("/crypto_report/:id", get(crypto_view_report))

// Target: Feature-specific route modules
// features/crypto_reports/routes.rs
pub fn crypto_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/crypto_report", get(crypto_index))
        .route("/crypto_report/:id", get(crypto_view_report))
        .route("/pdf-template/:id", get(pdf_template))
}

// features/health_system/routes.rs
pub fn health_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/health", get(health))
        .route("/metrics", get(performance_metrics))
}
```

### Handler Migration Strategy
1. **Group handlers by feature domain**
2. **Preserve exact signatures and behavior**
3. **Maintain error handling patterns**
4. **Keep response formats identical**

---

**📝 Generated**: August 20, 2025  
**🔄 Version**: 1.0  
**📊 Source Lines**: 52 lines route definition + 800+ lines handler implementations  
**🎯 Migration Target**: `features/*/routes.rs` + `features/*/handlers.rs`
