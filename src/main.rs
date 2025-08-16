use axum::{extract::{Path, Query, State, ws::WebSocketUpgrade}, http::StatusCode, response::{Html, IntoResponse, Response}, routing::get, Json, Router};
use tower_http::services::ServeDir;
use dotenvy::dotenv;
use serde::Serialize;
use sqlx::FromRow;
use sqlx::PgPool;
use std::{env, net::SocketAddr, sync::Arc};
use serde_json::json;
use tokio::sync::RwLock;
use tokio::fs;
use tera::{Tera, Context};
use std::error::Error as StdError;
use rayon::prelude::*; // Thêm rayon cho CPU parallelism
use std::sync::atomic::{AtomicUsize, Ordering};
use dashmap::DashMap; // Thread-safe HashMap thay thế RwLock<HashMap>

mod data_service;
mod websocket_service;

use data_service::DataService;

struct AppState {
    db: PgPool,
    auto_update_secret: Option<String>,
    // Thread-safe cache cho chart modules
    chart_modules_cache: RwLock<Option<String>>,
    // DashMap thay thế RwLock<HashMap> để tránh lock contention
    cached_reports: DashMap<i32, Report>,
    // Atomic cho latest report ID
    cached_latest_id: AtomicUsize, // Sử dụng AtomicUsize thay vì RwLock
    // Tera template engine - thread-safe
    tera: Tera,
    // WebSocket service for real-time updates
    websocket_service: Arc<crate::websocket_service::WebSocketService>,
    // Thread pool cho CPU-intensive tasks
    cpu_pool: rayon::ThreadPool,
    // Request counter cho monitoring
    request_counter: AtomicUsize,
}

#[derive(FromRow, Serialize, Debug, Clone)]
struct Report {
    id: i32,
    html_content: String,
    css_content: Option<String>,
    js_content: Option<String>,
    html_content_en: Option<String>,
    js_content_en: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
    let auto_update_secret = env::var("AUTO_UPDATE_SECRET_KEY").ok();
    let taapi_secret = env::var("TAAPI_SECRET").expect("TAAPI_SECRET must be set in .env");
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

    // Tối ưu connection pool cho đa luồng
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(32) // Tăng từ default 10 lên 32 cho 16 cores
        .min_connections(8)  // Duy trì ít nhất 8 connections
        .max_lifetime(std::time::Duration::from_secs(30 * 60)) // 30 phút
        .idle_timeout(std::time::Duration::from_secs(10 * 60)) // 10 phút idle
        .acquire_timeout(std::time::Duration::from_secs(30)) // Timeout nếu không lấy được connection
        .connect(&database_url).await?;

    // Initialize data service
    let data_service = DataService::new(taapi_secret);
    
    // Initialize WebSocket service
    let websocket_service = Arc::new(crate::websocket_service::WebSocketService::new(&redis_url, data_service)?);
    
    // Start background data updates
    websocket_service.start_background_updates().await;

    // Initialize Tera template engine with new architecture
    let mut tera = Tera::default();
    
    // Load shared components (global)
    tera.add_template_file("shared_components/theme_toggle.html", Some("shared/components/theme_toggle.html")).expect("Failed to load shared theme_toggle.html");
    tera.add_template_file("shared_components/language_toggle.html", Some("shared/components/language_toggle.html")).expect("Failed to load shared language_toggle.html");
    
    // Load route-specific templates for crypto_dashboard
    tera.add_template_file("dashboards/crypto_dashboard/routes/reports/view.html", Some("crypto/routes/reports/view.html")).expect("Failed to load crypto reports view template");
    tera.add_template_file("dashboards/crypto_dashboard/routes/reports/pdf.html", Some("crypto/routes/reports/pdf.html")).expect("Failed to load crypto reports pdf template");
    tera.add_template_file("dashboards/crypto_dashboard/routes/reports/list.html", Some("crypto/routes/reports/list.html")).expect("Failed to load crypto reports list template");
    
    // Load legacy templates for backwards compatibility (keeping for fallback)
    // Add legacy components as well for backwards compatibility
    tera.add_template_file("shared_components/theme_toggle.html", Some("crypto/components/theme_toggle.html")).expect("Failed to load legacy crypto theme_toggle.html");
    tera.add_template_file("shared_components/language_toggle.html", Some("crypto/components/language_toggle.html")).expect("Failed to load legacy crypto language_toggle.html");
    
    tera.autoescape_on(vec![]); // Disable auto-escaping for safe content

    // Khởi tạo thread pool với số lõi tối ưu
    let num_cpus = num_cpus::get();
    let cpu_pool = rayon::ThreadPoolBuilder::new()
        .num_threads(num_cpus)
        .thread_name(|index| format!("cpu-worker-{}", index))
        .build()
        .expect("Failed to create CPU thread pool");

    let state = AppState { 
        db: pool, 
        auto_update_secret, 
        chart_modules_cache: RwLock::new(None),
        cached_reports: DashMap::new(), // Thread-safe HashMap
        cached_latest_id: AtomicUsize::new(0), // Atomic counter
        tera,
        websocket_service,
        cpu_pool,
        request_counter: AtomicUsize::new(0),
    };
    let shared_state = Arc::new(state);

    // Prime the latest-report cache once at startup to reduce first-request latency
    // (best-effort; failure won't stop the server)
    {
        let s = Arc::clone(&shared_state);
        let pool_ref = s.db.clone();
        tokio::spawn(async move {
            if let Ok(Some(report)) = sqlx::query_as::<_, Report>(
                "SELECT id, html_content, css_content, js_content, html_content_en, js_content_en, created_at FROM report ORDER BY created_at DESC LIMIT 1",
            )
            .fetch_optional(&pool_ref)
            .await
            {
                // Insert vào DashMap (thread-safe)
                s.cached_reports.insert(report.id, report.clone());
                // Cập nhật latest id với atomic
                s.cached_latest_id.store(report.id as usize, Ordering::Relaxed);
            }
        });
    }

    let app = Router::new()
    // Serve crypto_dashboard assets
    .nest_service("/crypto_dashboard/shared", ServeDir::new("dashboards/crypto_dashboard/shared"))
    .nest_service("/crypto_dashboard/routes", ServeDir::new("dashboards/crypto_dashboard/routes"))
    .nest_service("/crypto_dashboard/assets", ServeDir::new("dashboards/crypto_dashboard/assets"))
    .nest_service("/crypto_dashboard/pages", ServeDir::new("dashboards/crypto_dashboard/pages"))
    
    // Serve stock_dashboard assets
    .nest_service("/stock_dashboard/shared", ServeDir::new("dashboards/stock_dashboard/shared"))
    .nest_service("/stock_dashboard/routes", ServeDir::new("dashboards/stock_dashboard/routes"))
    .nest_service("/stock_dashboard/assets", ServeDir::new("dashboards/stock_dashboard/assets"))
    .nest_service("/stock_dashboard/pages", ServeDir::new("dashboards/stock_dashboard/pages"))
    
    // Serve shared components and assets
    .nest_service("/shared_components", ServeDir::new("shared_components"))
    .nest_service("/shared_assets", ServeDir::new("shared_assets"))
    
    // Legacy compatibility routes
    .nest_service("/assets", ServeDir::new("dashboards/crypto_dashboard/assets"))
    .nest_service("/static", ServeDir::new("dashboards/crypto_dashboard/assets"))
        .route("/health", get(health))
        .route("/metrics", get(performance_metrics)) // Performance monitoring endpoint
        .route("/admin/cache/clear", get(clear_cache)) // Cache management endpoint
        .route("/admin/cache/stats", get(cache_stats)) // Cache statistics endpoint
        .route("/", get(homepage))
        .route("/crypto_report", get(crypto_index))
        .route("/crypto_report/:id", get(crypto_view_report))
        .route("/pdf-template/:id", get(pdf_template))
        .route("/crypto_reports_list", get(report_list))
        .route("/upload", get(upload_page))
        .route("/auto-update-system-:secret", get(auto_update))
        // New WebSocket and API routes
        .route("/ws", get(websocket_handler))
        .route("/api/crypto/dashboard-summary", get(dashboard_summary_api))
        .route("/api/crypto/dashboard-summary/refresh", get(force_refresh_dashboard))
        .route("/shared_assets/js/chart_modules.js", get(serve_chart_modules))
    .with_state(shared_state);

    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port: u16 = env::var("PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(8000);
    let addr: SocketAddr = format!("{}:{}", host, port).parse().unwrap();

    println!("🚀 Starting high-performance Rust server");
    println!("📍 Address: http://{}", addr);
    println!("🖥️  Available CPUs: {}", num_cpus::get());
    println!("🗂️  Database pool: max_connections=32, min_connections=8");
    println!("🏃 Rayon thread pool: {} worker threads", num_cpus::get());
    println!("💾 Cache: DashMap (lock-free), Atomic counters");
    
    // Sử dụng axum::Server::bind cho compatibility với axum 0.6
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

async fn health(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let request_count = state.request_counter.load(Ordering::Relaxed);
    let cache_size = state.cached_reports.len();
    let latest_id = state.cached_latest_id.load(Ordering::Relaxed);
    
    Json(serde_json::json!({
        "status": "healthy", 
        "message": "Crypto Dashboard Rust server is running",
        "metrics": {
            "total_requests": request_count,
            "cache_size": cache_size,
            "latest_report_id": latest_id,
            "available_cpus": num_cpus::get(),
            "thread_pool_active": true
        }
    }))
}

// Performance monitoring endpoint
async fn performance_metrics(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let request_count = state.request_counter.load(Ordering::Relaxed);
    let cache_size = state.cached_reports.len();
    let latest_id = state.cached_latest_id.load(Ordering::Relaxed);
    let num_cpus = num_cpus::get();
    
    // Get system memory info (basic)
    let memory_info = {
        #[cfg(target_os = "linux")]
        {
            std::fs::read_to_string("/proc/meminfo")
                .ok()
                .and_then(|content| {
                    let mut total = 0u64;
                    let mut available = 0u64;
                    for line in content.lines() {
                        if line.starts_with("MemTotal:") {
                            total = line.split_whitespace().nth(1)?.parse().ok()?;
                        } else if line.starts_with("MemAvailable:") {
                            available = line.split_whitespace().nth(1)?.parse().ok()?;
                        }
                    }
                    Some((total, available))
                })
        }
        #[cfg(not(target_os = "linux"))]
        {
            None
        }
    };
    
    let mut metrics = json!({
        "performance": {
            "total_requests_processed": request_count,
            "cache_metrics": {
                "reports_cached": cache_size,
                "latest_report_id": latest_id
            },
            "system_resources": {
                "cpu_cores": num_cpus,
                "rayon_thread_pool_active": true
            },
            "uptime_seconds": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        }
    });
    
    if let Some((total_mem, available_mem)) = memory_info {
        metrics["performance"]["system_resources"]["memory_total_kb"] = json!(total_mem);
        metrics["performance"]["system_resources"]["memory_available_kb"] = json!(available_mem);
        metrics["performance"]["system_resources"]["memory_used_percent"] = 
            json!(((total_mem - available_mem) as f64 / total_mem as f64) * 100.0);
    }
    
    Json(metrics)
}

// Cache management endpoints
async fn clear_cache(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    // Clear all caches
    let reports_cleared = state.cached_reports.len();
    state.cached_reports.clear();
    state.cached_latest_id.store(0, Ordering::Relaxed);
    
    // Clear chart modules cache
    {
        let mut w = state.chart_modules_cache.write().await;
        *w = None;
    }
    
    Json(json!({
        "status": "success",
        "message": "All caches cleared",
        "details": {
            "reports_cleared": reports_cleared,
            "chart_modules_cache_cleared": true,
            "latest_id_reset": true
        }
    }))
}

async fn cache_stats(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let reports_cached = state.cached_reports.len();
    let latest_id = state.cached_latest_id.load(Ordering::Relaxed);
    let chart_modules_cached = state.chart_modules_cache.read().await.is_some();
    
    // Get cache hit statistics from requests processed
    let total_requests = state.request_counter.load(Ordering::Relaxed);
    
    Json(json!({
        "cache_statistics": {
            "reports_cache": {
                "total_cached_reports": reports_cached,
                "latest_report_id": latest_id,
                "cache_hit_ratio_estimate": if reports_cached > 0 { "High (DashMap efficient)" } else { "No cache" }
            },
            "chart_modules_cache": {
                "cached": chart_modules_cached,
                "status": if chart_modules_cached { "Active" } else { "Empty" }
            },
            "performance": {
                "total_requests_processed": total_requests,
                "cache_type": "DashMap + Atomic counters (lock-free)"
            }
        }
    }))
}

async fn crypto_index(State(state): State<Arc<AppState>>) -> Response {
    // Increment request counter để monitor
    let request_count = state.request_counter.fetch_add(1, Ordering::Relaxed);
    
    // Log mỗi 100 requests để monitor performance
    if request_count % 100 == 0 {
        println!("Processed {} requests to crypto_index", request_count);
    }

    // Fast path: kiểm tra cache với atomic operation
    let latest_id = state.cached_latest_id.load(Ordering::Relaxed) as i32;
    if latest_id > 0 {
        if let Some(cached_report) = state.cached_reports.get(&latest_id) {
            let cached = cached_report.clone();
            drop(cached_report); // Release reference sớm
            
            // Parallel fetch chart modules để tránh blocking
            let chart_modules_content = get_chart_modules_content(&state).await;
            
            // Sử dụng spawn_blocking cho template rendering nặng
            let tera = state.tera.clone();
            
            let render_result = {
                let cached_clone = cached.clone();
                let chart_content_clone = chart_modules_content.clone();
                
                // Spawn blocking task cho template rendering
                tokio::task::spawn_blocking(move || {
                    let mut context = Context::new();
                    context.insert("current_route", "dashboard");
                    context.insert("current_lang", "vi");
                    context.insert("current_time", &chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string());
                    context.insert("report", &cached_clone);
                    context.insert("chart_modules_content", &chart_content_clone);
                    let pdf_url = format!("/pdf-template/{}", cached_clone.id);
                    context.insert("pdf_url", &pdf_url);

                    tera.render("crypto/routes/reports/view.html", &context)
                }).await
            };

            match render_result {
                Ok(Ok(html)) => {
                    return Response::builder()
                        .status(StatusCode::OK)
                        .header("cache-control", "public, max-age=15")
                        .header("content-type", "text/html; charset=utf-8")
                        .header("x-cache", "hit") // Cache hit indicator
                        .body(html)
                        .unwrap()
                        .into_response();
                }
                Ok(Err(e)) => {
                    eprintln!("Template render error: {:#?}", e);
                    let mut src = e.source();
                    while let Some(s) = src {
                        eprintln!("Template render error source: {:#?}", s);
                        src = s.source();
                    }
                    return (StatusCode::INTERNAL_SERVER_ERROR, "Template render error").into_response();
                }
                Err(e) => {
                    eprintln!("Task join error: {:#?}", e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, "Internal error").into_response();
                }
            }
        }
    }

    // Cache miss: fetch DB và chart modules song song
    let db_fut = sqlx::query_as::<_, Report>(
        "SELECT id, html_content, css_content, js_content, html_content_en, js_content_en, created_at FROM report ORDER BY created_at DESC LIMIT 1",
    ).fetch_optional(&state.db);
    let chart_fut = get_chart_modules_content(&state);

    let (db_res, chart_modules_content) = tokio::join!(db_fut, chart_fut);

    match db_res {
        Ok(Some(report)) => {
            // Update cache với thread-safe operations
            state.cached_reports.insert(report.id, report.clone());
            state.cached_latest_id.store(report.id as usize, Ordering::Relaxed);
            
            // Template rendering trong CPU pool
            let tera = state.tera.clone();
            let report_clone = report.clone();
            let chart_content_clone = chart_modules_content.clone();
            
            let render_result = tokio::task::spawn_blocking(move || {
                let mut context = Context::new();
                context.insert("current_route", "dashboard");
                context.insert("current_lang", "vi");
                context.insert("current_time", &chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string());
                context.insert("report", &report_clone);
                context.insert("chart_modules_content", &chart_content_clone);
                let pdf_url = format!("/pdf-template/{}", report_clone.id);
                context.insert("pdf_url", &pdf_url);

                tera.render("crypto/routes/reports/view.html", &context)
            }).await;

            match render_result {
                Ok(Ok(html)) => Response::builder()
                    .status(StatusCode::OK)
                    .header("cache-control", "public, max-age=15")
                    .header("content-type", "text/html; charset=utf-8")
                    .header("x-cache", "miss") // Cache miss indicator
                    .body(html)
                    .unwrap()
                    .into_response(),
                Ok(Err(e)) => {
                    eprintln!("Template render error: {:#?}", e);
                    let mut src = e.source();
                    while let Some(s) = src {
                        eprintln!("Template render error source: {:#?}", s);
                        src = s.source();
                    }
                    (StatusCode::INTERNAL_SERVER_ERROR, "Template render error").into_response()
                }
                Err(e) => {
                    eprintln!("Task join error: {:#?}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, "Internal error").into_response()
                }
            }
        }
        Ok(None) => {
            let empty_report = serde_json::json!({
                "html_content": "",
                "html_content_en": "",
                "css_content": "",
                "js_content": ""
            });
            
            let tera = state.tera.clone();
            let chart_content_clone = chart_modules_content.clone();
            
            let render_result = tokio::task::spawn_blocking(move || {
                let mut context = Context::new();
                context.insert("current_route", "dashboard");
                context.insert("current_lang", "vi");
                context.insert("current_time", &chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string());
                context.insert("report", &empty_report);
                context.insert("chart_modules_content", &chart_content_clone);
                context.insert("pdf_url", &"#");

                tera.render("crypto/routes/reports/view.html", &context)
            }).await;

            match render_result {
                Ok(Ok(html)) => Response::builder()
                    .status(StatusCode::OK)
                    .header("cache-control", "public, max-age=15")
                    .header("content-type", "text/html; charset=utf-8")
                    .header("x-cache", "empty")
                    .body(html)
                    .unwrap()
                    .into_response(),
                Ok(Err(e)) => {
                    eprintln!("Template render error: {:#?}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, "Template render error").into_response()
                }
                Err(e) => {
                    eprintln!("Task join error: {:#?}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, "Internal error").into_response()
                }
            }
        }
        Err(e) => {
            eprintln!("DB error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response()
        }
    }
}

async fn homepage(State(state): State<Arc<AppState>>) -> Response {
    // Increment request counter
    state.request_counter.fetch_add(1, Ordering::Relaxed);
    
    // Async file reading để tránh block thread
    match fs::read_to_string("dashboards/home.html").await {
        Ok(s) => Html(s).into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "Home page not found").into_response(),
    }
}

async fn crypto_view_report(Path(id): Path<i32>, State(state): State<Arc<AppState>>) -> Response {
    // Increment request counter
    state.request_counter.fetch_add(1, Ordering::Relaxed);
    
    // Fast path: check cache với DashMap
    if let Some(cached_report) = state.cached_reports.get(&id) {
        let cached = cached_report.clone();
        drop(cached_report); // Release reference sớm
        
        // Parallel fetch chart modules
        let chart_modules_content = get_chart_modules_content(&state).await;
        
        // Template rendering trong spawn_blocking
        let tera = state.tera.clone();
        let cached_clone = cached.clone();
        let chart_content_clone = chart_modules_content.clone();
        
        let render_result = tokio::task::spawn_blocking(move || {
            let mut context = Context::new();
            context.insert("report", &cached_clone);
            context.insert("chart_modules_content", &chart_content_clone);
            let pdf_url = format!("/pdf-template/{}", cached_clone.id);
            context.insert("pdf_url", &pdf_url);

            tera.render("crypto/routes/reports/view.html", &context)
        }).await;

        match render_result {
            Ok(Ok(html)) => return Response::builder()
                .status(StatusCode::OK)
                .header("cache-control", "public, max-age=15")
                .header("content-type", "text/html; charset=utf-8")
                .header("x-cache", "hit")
                .body(html)
                .unwrap()
                .into_response(),
            Ok(Err(e)) => {
                eprintln!("Template render error: {:#?}", e);
                let mut src = e.source();
                while let Some(s) = src {
                    eprintln!("Template render error source: {:#?}", s);
                    src = s.source();
                }
                return (StatusCode::INTERNAL_SERVER_ERROR, "Template render error").into_response();
            }
            Err(e) => {
                eprintln!("Task join error: {:#?}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, "Internal error").into_response();
            }
        }
    }

    // Cache miss: fetch DB và chart modules concurrently
    let db_fut = sqlx::query_as::<_, Report>(
        "SELECT id, html_content, css_content, js_content, html_content_en, js_content_en, created_at FROM report WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db);

    let chart_fut = get_chart_modules_content(&state);

    let (db_res, chart_modules_content) = tokio::join!(db_fut, chart_fut);

    match db_res {
        Ok(Some(report)) => {
            // Insert vào DashMap
            state.cached_reports.insert(report.id, report.clone());
            
            // Update latest id nếu report này mới hơn
            let current_latest = state.cached_latest_id.load(Ordering::Relaxed) as i32;
            if current_latest == 0 || report.id > current_latest {
                state.cached_latest_id.store(report.id as usize, Ordering::Relaxed);
            }

            // Template rendering trong spawn_blocking
            let tera = state.tera.clone();
            let report_clone = report.clone();
            let chart_content_clone = chart_modules_content.clone();
            
            let render_result = tokio::task::spawn_blocking(move || {
                let mut context = Context::new();
                context.insert("report", &report_clone);
                context.insert("chart_modules_content", &chart_content_clone);
                let pdf_url = format!("/pdf-template/{}", report_clone.id);
                context.insert("pdf_url", &pdf_url);

                tera.render("crypto/routes/reports/view.html", &context)
            }).await;

            match render_result {
                Ok(Ok(html)) => Response::builder()
                        .status(StatusCode::OK)
                        .header("cache-control", "public, max-age=15")
                        .header("content-type", "text/html; charset=utf-8")
                        .header("x-cache", "miss")
                        .body(html)
                        .unwrap()
                        .into_response(),
                Ok(Err(e)) => {
                    eprintln!("Template render error: {:#?}", e);
                    let mut src = e.source();
                    while let Some(s) = src {
                        eprintln!("Template render error source: {:#?}", s);
                        src = s.source();
                    }
                    (StatusCode::INTERNAL_SERVER_ERROR, "Template render error").into_response()
                }
                Err(e) => {
                    eprintln!("Task join error: {:#?}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, "Internal error").into_response()
                }
            }
        }
        Ok(None) => (StatusCode::NOT_FOUND, "Report not found").into_response(),
        Err(e) => {
            eprintln!("DB error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response()
        }
    }
}

async fn pdf_template(Path(id): Path<i32>, State(state): State<Arc<AppState>>) -> Response {
    // Increment request counter
    state.request_counter.fetch_add(1, Ordering::Relaxed);
    
    // Fast path: return cached report if present
    if let Some(cached_report) = state.cached_reports.get(&id) {
        let cached = cached_report.clone();
        drop(cached_report);
        
        let chart_modules_content = get_chart_modules_content(&state).await;

        // Template rendering trong spawn_blocking
        let tera = state.tera.clone();
        let cached_clone = cached.clone();
        let chart_content_clone = chart_modules_content.clone();
        
        let render_result = tokio::task::spawn_blocking(move || {
            let mut context = Context::new();
            context.insert("report", &cached_clone);
            context.insert("chart_modules_content", &chart_content_clone);

            // formatted created date in UTC+7 for display
            let created_display = (cached_clone.created_at + chrono::Duration::hours(7)).format("%d-%m-%Y %H:%M").to_string();
            context.insert("created_at_display", &created_display);

            tera.render("crypto/routes/reports/pdf.html", &context)
        }).await;

        match render_result {
            Ok(Ok(html)) => return Html(html).into_response(),
            Ok(Err(e)) => {
                eprintln!("Template render error: {:#?}", e);
                let mut src = e.source();
                while let Some(s) = src {
                    eprintln!("Template render error source: {:#?}", s);
                    src = s.source();
                }
                return (StatusCode::INTERNAL_SERVER_ERROR, "Template render error").into_response();
            }
            Err(e) => {
                eprintln!("Task join error: {:#?}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, "Internal error").into_response();
            }
        }
    }

    // Cache miss: fetch DB và chart modules concurrently
    let db_fut = sqlx::query_as::<_, Report>(
        "SELECT id, html_content, css_content, js_content, html_content_en, js_content_en, created_at FROM report WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db);

    let chart_fut = get_chart_modules_content(&state);

    let (db_res, chart_modules_content) = tokio::join!(db_fut, chart_fut);

    match db_res {
        Ok(Some(report)) => {
            // insert vào cache
            state.cached_reports.insert(report.id, report.clone());

            // optionally update latest id if this is newer
            let current_latest = state.cached_latest_id.load(Ordering::Relaxed) as i32;
            if current_latest == 0 || report.id > current_latest {
                state.cached_latest_id.store(report.id as usize, Ordering::Relaxed);
            }

            // Template rendering trong spawn_blocking
            let tera = state.tera.clone();
            let report_clone = report.clone();
            let chart_content_clone = chart_modules_content.clone();
            
            let render_result = tokio::task::spawn_blocking(move || {
                let mut context = Context::new();
                context.insert("report", &report_clone);
                context.insert("chart_modules_content", &chart_content_clone);

                // formatted created date in UTC+7 for display
                let created_display = (report_clone.created_at + chrono::Duration::hours(7)).format("%d-%m-%Y %H:%M").to_string();
                context.insert("created_at_display", &created_display);

                tera.render("crypto/routes/reports/pdf.html", &context)
            }).await;

            match render_result {
                Ok(Ok(html)) => Html(html).into_response(),
                Ok(Err(e)) => {
                    eprintln!("Template render error: {:#?}", e);
                    let mut src = e.source();
                    while let Some(s) = src {
                        eprintln!("Template render error source: {:#?}", s);
                        src = s.source();
                    }
                    (StatusCode::INTERNAL_SERVER_ERROR, "Template render error").into_response()
                }
                Err(e) => {
                    eprintln!("Task join error: {:#?}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, "Internal error").into_response()
                }
            }
        }
        Ok(None) => (StatusCode::NOT_FOUND, "Report not found").into_response(),
        Err(e) => {
            eprintln!("DB error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response()
        }
    }
}

#[derive(FromRow, Serialize)]
struct ReportSummary {
    id: i32,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize)]
struct ReportListItem {
    id: i32,
    created_date: String,
    created_time: String,
}

async fn report_list(Query(params): Query<std::collections::HashMap<String, String>>, State(state): State<Arc<AppState>>) -> Response {
    // Increment request counter
    state.request_counter.fetch_add(1, Ordering::Relaxed);
    
    // Pagination params
    let page: i64 = params.get("page").and_then(|p| p.parse().ok()).unwrap_or(1);
    let per_page: i64 = 10;
    let offset = (page - 1) * per_page;

    // Parallel fetch total count và page rows
    let total_fut = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM report").fetch_one(&state.db);
    let rows_fut = sqlx::query_as::<_, ReportSummary>(
        "SELECT id, created_at FROM report ORDER BY created_at DESC LIMIT $1 OFFSET $2",
    )
    .bind(per_page as i64)
    .bind(offset as i64)
    .fetch_all(&state.db);

    let (total_res, rows_res) = tokio::join!(total_fut, rows_fut);

    let total = match total_res {
        Ok(t) => t,
        Err(e) => {
            eprintln!("DB error: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
        }
    };

    let list = match rows_res {
        Ok(list) => list,
        Err(e) => {
            eprintln!("DB error: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
        }
    };

    // Parallel processing của items với rayon (CPU-intensive date formatting)
    let items: Vec<ReportListItem> = tokio::task::spawn_blocking(move || {
        use rayon::prelude::*;
        
        list.par_iter()
            .map(|r| {
                let dt = r.created_at + chrono::Duration::hours(7);
                let created_date = dt.format("%d/%m/%Y").to_string();
                let created_time = format!("{} UTC+7", dt.format("%H:%M:%S"));
                ReportListItem { 
                    id: r.id, 
                    created_date, 
                    created_time 
                }
            })
            .collect()
    }).await.unwrap_or_else(|e| {
        eprintln!("Task join error: {:#?}", e);
        Vec::new()
    });

    // Parallel computation của pagination logic
    let (pages, page_numbers) = tokio::task::spawn_blocking(move || {
        let pages = if total == 0 { 1 } else { ((total as f64) / (per_page as f64)).ceil() as i64 };
        
        // Build simple page numbers similar to Flask pagination.iter_pages
        let mut page_numbers: Vec<Option<i64>> = Vec::new();
        if pages <= 10 {
            for p in 1..=pages { 
                page_numbers.push(Some(p)); 
            }
        } else {
            // always show first 1-2, last 1-2, and current +/-2 with ellipses
            let mut added = std::collections::HashSet::new();
            let push = |vec: &mut Vec<Option<i64>>, v: i64, added: &mut std::collections::HashSet<i64>| {
                if !added.contains(&v) && v > 0 && v <= pages {
                    vec.push(Some(v));
                    added.insert(v);
                }
            };
            
            push(&mut page_numbers, 1, &mut added);
            push(&mut page_numbers, 2, &mut added);
            for v in (page-2)..=(page+2) { 
                if v > 2 && v < pages-1 { 
                    push(&mut page_numbers, v, &mut added); 
                } 
            }
            push(&mut page_numbers, pages-1, &mut added);
            push(&mut page_numbers, pages, &mut added);

            // sort and insert None where gaps >1
            let mut nums: Vec<i64> = page_numbers.iter().filter_map(|o| *o).collect();
            nums.sort();
            page_numbers.clear();
            let mut last: Option<i64> = None;
            for n in nums {
                if let Some(l) = last {
                    if n - l > 1 {
                        page_numbers.push(None);
                    }
                }
                page_numbers.push(Some(n));
                last = Some(n);
            }
        }
        
        (pages, page_numbers)
    }).await.unwrap_or_else(|e| {
        eprintln!("Pagination task join error: {:#?}", e);
        (1, vec![Some(1)])
    });

    // Build reports context
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

    // Template rendering trong spawn_blocking
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
            let mut src = e.source();
            while let Some(s) = src {
                eprintln!("Template render error source: {:#?}", s);
                src = s.source();
            }
            // Dump the reports JSON context to help diagnose template issues
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

async fn upload_page(State(state): State<Arc<AppState>>) -> Response {
    // Increment request counter
    state.request_counter.fetch_add(1, Ordering::Relaxed);
    
    // Async file reading để tránh block thread
    let file_read_result = tokio::task::spawn_blocking(|| {
        std::fs::read_to_string("crypto_dashboard/templates/upload.html")
    }).await;
    
    match file_read_result {
        Ok(Ok(s)) => Html(s).into_response(),
        Ok(Err(_)) => (StatusCode::NOT_FOUND, "Upload page not found").into_response(),
        Err(e) => {
            eprintln!("File read task error: {:#?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal error").into_response()
        }
    }
}

async fn auto_update(Path(secret): Path<String>, State(state): State<Arc<AppState>>) -> Response {
    // Increment request counter
    state.request_counter.fetch_add(1, Ordering::Relaxed);
    
    match &state.auto_update_secret {
        None => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "Auto update system not configured", "message": "Set AUTO_UPDATE_SECRET_KEY in .env"})),
        )
            .into_response(),
        Some(required) => {
            if &secret != required {
                eprintln!("Unauthorized access attempt with key: {}", secret);
                (StatusCode::FORBIDDEN, Json(serde_json::json!({"error": "Access denied", "message": "Invalid secret key"}))).into_response()
            } else {
                // Async file reading để tránh block thread
                let file_read_result = tokio::task::spawn_blocking(|| {
                    std::fs::read_to_string("crypto_dashboard/templates/auto_update.html")
                }).await;
                
                match file_read_result {
                    Ok(Ok(s)) => Html(s).into_response(),
                    Ok(Err(_)) => (StatusCode::NOT_FOUND, "Auto update page not found").into_response(),
                    Err(e) => {
                        eprintln!("File read task error: {:#?}", e);
                        (StatusCode::INTERNAL_SERVER_ERROR, "Internal error").into_response()
                    }
                }
            }
        }
    }
}

async fn get_chart_modules_content(state: &AppState) -> String {
    use tokio::fs::read_dir;
    use std::path::Path;

    // If not in debug mode, try cache first
    let debug = env::var("DEBUG").unwrap_or_default() == "1";
    if !debug {
        if let Some(cached) = state.chart_modules_cache.read().await.clone() {
            return cached;
        }
    }

    let source_dir = Path::new("shared_assets").join("js").join("chart_modules");
    let priority_order = vec!["gauge.js", "bar.js", "line.js", "doughnut.js"];

    let mut entries = match read_dir(&source_dir).await {
        Ok(rd) => rd,
        Err(_) => return "// No chart modules found".to_string(),
    };

    let mut all_files = Vec::new();
    while let Ok(Some(entry)) = entries.next_entry().await {
        if let Ok(ft) = entry.file_type().await {
            if ft.is_file() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.ends_with(".js") {
                        all_files.push(name.to_string());
                    }
                }
            }
        }
    }

    // Order files: priority first, then alphabetically
    let mut ordered = Vec::new();
    for p in &priority_order {
        if let Some(idx) = all_files.iter().position(|f| f == p) {
            ordered.push(all_files.remove(idx));
        }
    }
    all_files.sort();
    ordered.extend(all_files);

    // Parallel file reading với concurrent futures
    let file_futures: Vec<_> = ordered
        .iter()
        .map(|filename| {
            let path = source_dir.join(filename);
            let filename_clone = filename.clone();
            async move {
                match tokio::fs::read_to_string(&path).await {
                    Ok(content) => {
                        let wrapped = format!(
                            "// ==================== {name} ====================\ntry {{\n{code}\n}} catch (error) {{\n    console.error('Error loading chart module {name}:', error);\n}}\n// ==================== End {name} ====================",
                            name = filename_clone,
                            code = content
                        );
                        wrapped
                    }
                    Err(_) => {
                        format!("// Warning: {name} not found", name = filename_clone)
                    }
                }
            }
        })
        .collect();

    // Await all file reads concurrently
    let parts = futures::future::join_all(file_futures).await;

    // Final concatenation trong CPU thread pool để avoid blocking async runtime
    let final_content = tokio::task::spawn_blocking(move || {
        parts.join("\n\n")
    }).await.unwrap_or_else(|e| {
        eprintln!("Chart modules concatenation error: {:#?}", e);
        "// Error loading chart modules".to_string()
    });

    // Cache if not debug
    if !debug {
        let mut w = state.chart_modules_cache.write().await;
        *w = Some(final_content.clone());
    }

    final_content
}

async fn serve_chart_modules(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let content = get_chart_modules_content(&state).await;
    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/javascript")
        .header("cache-control", "public, max-age=3600")
        .body(content)
        .unwrap()
}

// WebSocket handler for real-time updates
async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| async move {
        state.websocket_service.handle_websocket(socket).await;
    })
}

// API endpoint to get cached dashboard summary
async fn dashboard_summary_api(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match state.websocket_service.get_cached_dashboard_data().await {
        Ok(Some(data)) => Json(data).into_response(),
        Ok(None) => {
            // No cached data, try to fetch fresh data
            match state.websocket_service.force_update_dashboard().await {
                Ok(data) => Json(data).into_response(),
                Err(e) => {
                    eprintln!("Failed to fetch dashboard data: {}", e);
                    (
                        StatusCode::SERVICE_UNAVAILABLE,
                        Json(json!({
                            "error": "Service temporarily unavailable",
                            "message": "Unable to fetch dashboard data"
                        }))
                    ).into_response()
                }
            }
        }
        Err(e) => {
            eprintln!("Redis error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Internal server error",
                    "message": "Database connection failed"
                }))
            ).into_response()
        }
    }
}

// API endpoint to force refresh dashboard data
async fn force_refresh_dashboard(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match state.websocket_service.force_update_dashboard().await {
        Ok(data) => Json(json!({
            "status": "success",
            "message": "Dashboard data refreshed",
            "data": data
        })).into_response(),
        Err(e) => {
            eprintln!("Failed to refresh dashboard data: {}", e);
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({
                    "error": "Service temporarily unavailable",
                    "message": format!("Unable to refresh dashboard data: {}", e)
                }))
            ).into_response()
        }
    }
}
