use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use serde_json::json;
use std::{collections::HashMap, error::Error as StdError, sync::Arc, sync::atomic::Ordering};
use tera::Context;
use tokio::fs;

use crate::{
    models::{Report, ReportListItem, ReportSummary},
    state::AppState,
    utils,
};

// Helper functions for code reuse
async fn render_crypto_template(
    tera: &tera::Tera, 
    template: &str,
    report: &Report,
    chart_modules_content: &str,
    additional_context: Option<HashMap<String, serde_json::Value>>
) -> Result<String, Box<dyn StdError + Send + Sync>> {
    let tera_clone = tera.clone();
    let template_str = template.to_string(); // Clone to owned string
    let report_clone = report.clone();
    let chart_content_clone = chart_modules_content.to_string();
    let additional_clone = additional_context.clone();
    
    let render_result = tokio::task::spawn_blocking(move || {
        let mut context = Context::new();
        context.insert("report", &report_clone);
        context.insert("chart_modules_content", &chart_content_clone);
        
        // Add additional context for different templates
        if let Some(extra) = additional_clone {
            for (key, value) in extra {
                context.insert(&key, &value);
            }
        }
        
        // Common context for view templates
        if template_str.contains("view.html") {
            context.insert("current_route", "dashboard");
            context.insert("current_lang", "vi");
            context.insert("current_time", &chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string());
            let pdf_url = format!("/pdf-template/{}", report_clone.id);
            context.insert("pdf_url", &pdf_url);
        }
        
        // PDF template specific context
        if template_str.contains("pdf.html") {
            let created_display = (report_clone.created_at + chrono::Duration::hours(7)).format("%d-%m-%Y %H:%M").to_string();
            context.insert("created_at_display", &created_display);
        }

        tera_clone.render(&template_str, &context)
    }).await;
    
    match render_result {
        Ok(Ok(html)) => Ok(html),
        Ok(Err(e)) => {
            eprintln!("Template render error: {:#?}", e);
            let mut src = e.source();
            while let Some(s) = src {
                eprintln!("Template render error source: {:#?}", s);
                src = s.source();
            }
            Err(format!("Template render error: {}", e).into())
        }
        Err(e) => {
            eprintln!("Task join error: {:#?}", e);
            Err(format!("Task join error: {}", e).into())
        }
    }
}

fn create_cached_response(html: String, cache_status: &str) -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header("cache-control", "public, max-age=15")
        .header("content-type", "text/html; charset=utf-8")
        .header("x-cache", cache_status)
        .body(html)
        .unwrap()
        .into_response()
}

async fn fetch_and_cache_report_by_id(
    state: &Arc<AppState>,
    id: i32
) -> Result<Option<Report>, sqlx::Error> {
    let report = sqlx::query_as::<_, Report>(
        "SELECT id, html_content, css_content, js_content, html_content_en, js_content_en, created_at FROM crypto_report WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?;
    
    if let Some(ref report) = report {
        // Cache in L1
        state.report_cache.insert(report.id, report.clone()).await;
        
        // Update latest id if newer
        let current_latest = state.cached_latest_id.load(Ordering::Relaxed) as i32;
        if current_latest == 0 || report.id > current_latest {
            state.cached_latest_id.store(report.id as usize, Ordering::Relaxed);
        }
        
        // Cache in L2 Redis
        if let Err(e) = state.cache_manager.set(&format!("crypto_report:{}", report.id), report).await {
            eprintln!("⚠️ Failed to cache report in Redis: {}", e);
        } else {
            println!("💾 Cached crypto report {} in Redis (key: crypto_report:{})", report.id, report.id);
        }
    }
    
    Ok(report)
}

async fn fetch_and_cache_latest_report(
    state: &Arc<AppState>
) -> Result<Option<Report>, sqlx::Error> {
    let report = sqlx::query_as::<_, Report>(
        "SELECT id, html_content, css_content, js_content, html_content_en, js_content_en, created_at FROM crypto_report ORDER BY created_at DESC LIMIT 1",
    ).fetch_optional(&state.db).await?;
    
    if let Some(ref report) = report {
        // Cache in L1
        state.report_cache.insert(report.id, report.clone()).await;
        state.cached_latest_id.store(report.id as usize, Ordering::Relaxed);
        
        // Cache in L2 Redis with TTL for latest report
        if let Err(e) = state.cache_manager.set_with_ttl("crypto_latest_report", report, 300).await {
            eprintln!("⚠️ Failed to cache latest report in Redis: {}", e);
        } else {
            println!("💾 Cached latest crypto report {} in Redis (key: crypto_latest_report, TTL: 5min)", report.id);
        }
    }
    
    Ok(report)
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

    // Fast path: check L1 cache (report_cache) using atomic latest id
    let latest_id = state.cached_latest_id.load(Ordering::Relaxed) as i32;
    if latest_id > 0 {
        if let Some(cached) = state.report_cache.get(&latest_id).await {
            // Parallel fetch chart modules to avoid blocking
            let chart_modules_content = utils::get_chart_modules_content(&state.chart_modules_cache).await;

            // Render using helper function
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
    }

    // L1 Cache miss: try L2 cache (Redis) before hitting DB
    println!("🔍 L1 Cache miss for crypto_index - checking L2 cache (Redis)");
    
    // Try to get the latest report from Redis cache using a fixed key
    if let Ok(Some(cached_report)) = state.cache_manager.get::<Report>("crypto_latest_report").await {
        println!("🔥 L2 Cache HIT for crypto_index from Redis");
        // Put it back into L1 cache for faster access
        state.report_cache.insert(cached_report.id, cached_report.clone()).await;
        state.cached_latest_id.store(cached_report.id as usize, Ordering::Relaxed);
        
        // Render from cached data
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
                // If rendering fails, continue to DB fetch
                println!("⚠️ Failed to render from L2 cache, falling back to DB");
            }
        }
    }

    // Both L1 and L2 cache miss: fetch from DB and cache in both L1 and L2
    println!("🔍 L1+L2 Cache miss for crypto_index - fetching from DB");

    // Cache miss: fetch DB và chart modules song song
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
                Ok(Ok(html)) => create_cached_response(html, "empty"),
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
    
    // Fast path: check L1 cache (report_cache)
    if let Some(cached) = state.report_cache.get(&id).await {
        // Parallel fetch chart modules
        let chart_modules_content = utils::get_chart_modules_content(&state.chart_modules_cache).await;

        // Template rendering using helper function
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

    // Cache miss: fetch DB và chart modules concurrently
    let db_fut = fetch_and_cache_report_by_id(&state, id);
    let chart_fut = utils::get_chart_modules_content(&state.chart_modules_cache);

    let (db_res, chart_modules_content) = tokio::join!(db_fut, chart_fut);

    match db_res {
        Ok(Some(report)) => {
            // Template rendering using helper function
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

pub async fn pdf_template(Path(id): Path<i32>, State(state): State<Arc<AppState>>) -> Response {
    // Increment request counter
    state.request_counter.fetch_add(1, Ordering::Relaxed);
    
    // Fast path: check L1 cache (report_cache)
    if let Some(cached) = state.report_cache.get(&id).await {
        let chart_modules_content = utils::get_chart_modules_content(&state.chart_modules_cache).await;

        // Template rendering using helper function
        match render_crypto_template(
            &state.tera,
            "crypto/routes/reports/pdf.html",
            &cached,
            &chart_modules_content,
            None
        ).await {
            Ok(html) => return Html(html).into_response(),
            Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Template render error").into_response(),
        }
    }

    // Cache miss: fetch DB và chart modules concurrently
    let db_fut = fetch_and_cache_report_by_id(&state, id);
    let chart_fut = utils::get_chart_modules_content(&state.chart_modules_cache);

    let (db_res, chart_modules_content) = tokio::join!(db_fut, chart_fut);

    match db_res {
        Ok(Some(report)) => {
            // Template rendering using helper function
            match render_crypto_template(
                &state.tera,
                "crypto/routes/reports/pdf.html",
                &report,
                &chart_modules_content,
                None
            ).await {
                Ok(html) => Html(html).into_response(),
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

pub async fn report_list(Query(params): Query<HashMap<String, String>>, State(state): State<Arc<AppState>>) -> Response {
    // Increment request counter
    state.request_counter.fetch_add(1, Ordering::Relaxed);
    
    // Pagination params
    let page: i64 = params.get("page").and_then(|p| p.parse().ok()).unwrap_or(1);
    let per_page: i64 = 10;
    let offset = (page - 1) * per_page;

    // Parallel fetch total count và page rows
    let total_fut = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM crypto_report").fetch_one(&state.db);
    let rows_fut = sqlx::query_as::<_, ReportSummary>(
        "SELECT id, created_at FROM crypto_report ORDER BY created_at DESC LIMIT $1 OFFSET $2",
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
    axum::response::Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/javascript")
        .header("cache-control", "public, max-age=3600")
        .body(content)
        .unwrap()
}
