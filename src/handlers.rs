use axum::{
    extract::{Path, Query, State, ws::WebSocketUpgrade},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::{collections::HashMap, error::Error as StdError, sync::Arc, sync::atomic::Ordering, time::Instant};
use tera::Context;
use tokio::fs;

use crate::{
    models::{Report, ReportListItem, ReportSummary},
    state::AppState,
    utils,
};

pub async fn health(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let start_time = Instant::now();
    
    let request_count = state.request_counter.load(Ordering::Relaxed);
    let cache_size = state.cached_reports.len();
    let latest_id = state.cached_latest_id.load(Ordering::Relaxed);
    
    // Record performance metrics
    let response_time = start_time.elapsed().as_millis() as u64;
    state.metrics.record_request(response_time);
    
    Json(serde_json::json!({
        "status": "healthy", 
        "message": "Crypto Dashboard Rust server is running",
        "metrics": {
            "total_requests": request_count,
            "cache_size": cache_size,
            "latest_report_id": latest_id,
            "available_cpus": num_cpus::get(),
            "thread_pool_active": true,
            "avg_response_time_ms": state.metrics.avg_response_time(),
            "cache_hit_rate": state.report_cache.hit_rate()
        }
    }))
}

pub async fn performance_metrics(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let request_count = state.request_counter.load(Ordering::Relaxed);
    let cache_size = state.cached_reports.len();
    let latest_id = state.cached_latest_id.load(Ordering::Relaxed);
    let num_cpus = num_cpus::get();
    
    // Get advanced cache statistics
    let cache_stats = state.report_cache.stats().await;
    let performance_stats = state.metrics.clone();
    
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
            "avg_response_time_ms": performance_stats.avg_response_time(),
            "cache_metrics": {
                "reports_cached": cache_size,
                "latest_report_id": latest_id,
                "l1_cache": {
                    "entries": cache_stats.entries,
                    "hits": cache_stats.hits,
                    "misses": cache_stats.misses,
                    "hit_rate": cache_stats.hit_rate
                }
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

pub async fn clear_cache(State(state): State<Arc<AppState>>) -> impl IntoResponse {
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

pub async fn cache_stats(State(state): State<Arc<AppState>>) -> impl IntoResponse {
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

pub async fn homepage(State(state): State<Arc<AppState>>) -> Response {
    // Increment request counter
    state.request_counter.fetch_add(1, Ordering::Relaxed);
    
    // Async file reading để tránh block thread
    match fs::read_to_string("dashboards/home.html").await {
        Ok(s) => Html(s).into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "Home page not found").into_response(),
    }
}

pub async fn crypto_index(State(state): State<Arc<AppState>>) -> Response {
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
            let chart_modules_content = utils::get_chart_modules_content(&state.chart_modules_cache).await;
            
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
    let chart_fut = utils::get_chart_modules_content(&state.chart_modules_cache);

    let (db_res, chart_modules_content): (Result<Option<Report>, sqlx::Error>, String) = tokio::join!(db_fut, chart_fut);

    match db_res {
        Ok(Some(report)) => {
            // Update cache với thread-safe operations
            state.cached_reports.insert(report.id, report.clone());
            state.cached_latest_id.store(report.id as usize, Ordering::Relaxed);
            
            // Template rendering với spawn_blocking
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

pub async fn crypto_view_report(Path(id): Path<i32>, State(state): State<Arc<AppState>>) -> Response {
    // Increment request counter
    state.request_counter.fetch_add(1, Ordering::Relaxed);
    
    // Fast path: check cache với DashMap
    if let Some(cached_report) = state.cached_reports.get(&id) {
        let cached = cached_report.clone();
        drop(cached_report); // Release reference sớm
        
        // Parallel fetch chart modules
        let chart_modules_content = utils::get_chart_modules_content(&state.chart_modules_cache).await;
        
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

    let chart_fut = utils::get_chart_modules_content(&state.chart_modules_cache);

    let (db_res, chart_modules_content): (Result<Option<Report>, sqlx::Error>, String) = tokio::join!(db_fut, chart_fut);

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

pub async fn pdf_template(Path(id): Path<i32>, State(state): State<Arc<AppState>>) -> Response {
    // Increment request counter
    state.request_counter.fetch_add(1, Ordering::Relaxed);
    
    // Fast path: return cached report if present
    if let Some(cached_report) = state.cached_reports.get(&id) {
        let cached = cached_report.clone();
        drop(cached_report);
        
        let chart_modules_content = utils::get_chart_modules_content(&state.chart_modules_cache).await;

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

    let chart_fut = utils::get_chart_modules_content(&state.chart_modules_cache);

    let (db_res, chart_modules_content): (Result<Option<Report>, sqlx::Error>, String) = tokio::join!(db_fut, chart_fut);

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

pub async fn report_list(Query(params): Query<HashMap<String, String>>, State(state): State<Arc<AppState>>) -> Response {
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

pub async fn serve_chart_modules(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let content = utils::get_chart_modules_content(&state.chart_modules_cache).await;
    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/javascript")
        .header("cache-control", "public, max-age=3600")
        .body(content)
        .unwrap()
}

// WebSocket handler for real-time updates
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| async move {
        state.websocket_service.handle_websocket(socket).await;
    })
}

// API endpoint to get cached dashboard summary
pub async fn dashboard_summary_api(
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
pub async fn force_refresh_dashboard(
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
