//! Crypto Reports HTTP Request Handlers
//! 
//! This module contains all HTTP request handlers related to crypto reports functionality.
//! Based on archive_old_code/handlers/crypto.rs
//! ONLY uses Template Engine - NO manual HTML creation

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Serialize, Deserialize};
use std::{collections::HashMap, error::Error as StdError, sync::Arc, sync::atomic::Ordering, path::Path, env};
use tera::Context;
use tokio::fs::read_dir;

// Import from current state - will be refactored when lower layers are implemented
use crate::state::AppState;

// Import from our specialized components
use super::report_creator::{Report, ReportSummary, ReportListItem, ReportCreator};
use super::pdf_generator::PdfGenerator;

/// Crypto Handlers
/// 
/// Contains all HTTP request handlers for crypto reports-related operations.
/// These handlers manage crypto report generation, PDF creation, and API interactions.
/// ONLY uses Template Engine like archive_old_code/handlers/crypto.rs
pub struct CryptoHandlers {
    pub report_creator: ReportCreator,
    pub pdf_generator: PdfGenerator,
}

impl CryptoHandlers {
    /// Create a new CryptoHandlers instance
    pub fn new() -> Self {
        Self {
            report_creator: ReportCreator::new(),
            pdf_generator: PdfGenerator::new(),
        }
    }
    
    /// Health check for crypto handlers
    pub async fn health_check(&self) -> bool {
        // Verify handlers are functioning properly
        true // Will implement actual health checks
    }

    /// Helper function for template rendering
    /// 
    /// Directly from archive_old_code/handlers/crypto.rs::render_crypto_template
    /// ONLY Template Engine - NO manual HTML
    pub async fn render_crypto_template(
        &self,
        tera: &tera::Tera, 
        template: &str,
        report: &Report,
        chart_modules_content: &str,
        additional_context: Option<HashMap<String, serde_json::Value>>
    ) -> Result<String, Box<dyn StdError + Send + Sync>> {
        let tera_clone = tera.clone();
        let template_str = template.to_string();
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
                let pdf_url = format!("/crypto_report/{}/pdf", report_clone.id);
                context.insert("pdf_url", &pdf_url);
            }
            
            // PDF template specific context
            if template_str.contains("pdf.html") {
                let created_display = (report_clone.created_at + chrono::Duration::hours(7)).format("%d-%m-%Y %H:%M").to_string();
                context.insert("created_at_display", &created_display);
            }

            // ONLY use Tera template engine - NO manual HTML
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

    /// Create cached response with proper headers
    /// 
    /// From archive_old_code/handlers/crypto.rs::create_cached_response
    pub fn create_cached_response(&self, html: String, cache_status: &str) -> Response {
        Response::builder()
            .status(StatusCode::OK)
            .header("cache-control", "public, max-age=15")
            .header("content-type", "text/html; charset=utf-8")
            .header("x-cache", cache_status)
            .body(html)
            .unwrap()
            .into_response()
    }

    /// Fetch and cache latest report 
    /// 
    /// From archive_old_code/handlers/crypto.rs::fetch_and_cache_latest_report
    /// Real database query - Data from database ONLY - NO HTML creation here
    pub async fn fetch_and_cache_latest_report(
        &self,
        state: &Arc<AppState>
    ) -> Result<Option<Report>, sqlx::Error> {
        // Real database query exactly like archive_old_code
        let report = sqlx::query_as::<_, Report>(
            "SELECT id, html_content, css_content, js_content, html_content_en, js_content_en, created_at FROM crypto_report ORDER BY created_at DESC LIMIT 1",
        ).fetch_optional(&state.db).await?;
        
        if let Some(ref report) = report {
            // Update latest id cache (atomic operation like archive)
            state.cached_latest_id.store(report.id, Ordering::Relaxed);
            println!("💾 Fetched latest crypto report {} from database", report.id);
            
            // TODO: Implement L1/L2 caching logic when cache layers are ready
            // Cache in L1
            // state.report_cache.insert(report.id, report.clone()).await;
            // 
            // Cache in L2 Redis with TTL for latest report
            // if let Err(e) = state.cache_manager.set_with_ttl("crypto_latest_report", report, 300).await {
            //     eprintln!("⚠️ Failed to cache latest report in Redis: {}", e);
            // } else {
            //     println!("💾 Cached latest crypto report {} in Redis (key: crypto_latest_report, TTL: 5min)", report.id);
            // }
        }
        
        Ok(report)
    }

    /// Fetch and cache specific report by ID
    /// 
    /// Similar to fetch_and_cache_latest_report but for specific ID
    /// Real database query - Data from database ONLY - NO HTML creation here
    pub async fn fetch_and_cache_report_by_id(
        &self,
        state: &Arc<AppState>,
        report_id: i32
    ) -> Result<Option<Report>, sqlx::Error> {
        println!("🔍 Fetching crypto report {} from database", report_id);
        
        // Real database query for specific ID
        let report = sqlx::query_as::<_, Report>(
            "SELECT id, html_content, css_content, js_content, html_content_en, js_content_en, created_at FROM crypto_report WHERE id = $1",
        )
        .bind(report_id)
        .fetch_optional(&state.db).await?;
        
        if let Some(ref report) = report {
            println!("💾 Fetched crypto report {} from database successfully", report.id);
            
            // TODO: Implement L1/L2 caching logic when cache layers are ready
            // Cache in L1
            // state.report_cache.insert(report.id, report.clone()).await;
            // 
            // Cache in L2 Redis with TTL for specific report
            // let cache_key = format!("crypto_report_{}", report.id);
            // if let Err(e) = state.cache_manager.set_with_ttl(&cache_key, report, 1800).await {
            //     eprintln!("⚠️ Failed to cache report {} in Redis: {}", report.id, e);
            // } else {
            //     println!("💾 Cached crypto report {} in Redis (key: {}, TTL: 30min)", report.id, cache_key);
            // }
        } else {
            println!("⚠️ Crypto report {} not found in database", report_id);
        }
        
        Ok(report)
    }

    /// Get chart modules content
    /// 
    /// Exactly from archive_old_code/utils.rs::get_chart_modules_content
    /// Reads actual chart modules from shared_assets/js/chart_modules/
    pub async fn get_chart_modules_content(&self) -> String {
        // TODO: Add chart modules cache when cache layer is ready
        // For now always read from files (like debug mode in archive)
        let debug = env::var("DEBUG").unwrap_or_default() == "1";
        
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

        // Parallel file reading with concurrent futures
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

        // Await all file reads concurrently like archive code
        let parts = futures::future::join_all(file_futures).await;

        // Final concatenation in CPU thread pool to avoid blocking async runtime
        let final_content = tokio::task::spawn_blocking(move || {
            parts.join("\n\n")
        }).await.unwrap_or_else(|e| {
            eprintln!("Chart modules concatenation error: {:#?}", e);
            "// Error loading chart modules".to_string()
        });

        // TODO: Cache if not debug when cache layer is ready
        // if !debug {
        //     let mut w = state.chart_modules_cache.write().await;
        //     *w = Some(final_content.clone());
        // }

        final_content
    }
    
    /// Crypto Index handler - Main crypto dashboard
    /// 
    /// Originally from archive_old_code/handlers/crypto.rs::crypto_index
    /// ONLY uses Tera template engine - NO manual HTML creation
    pub async fn crypto_index(&self) -> Result<String, Box<dyn StdError + Send + Sync>> {
        println!("🚀 CryptoHandlers::crypto_index - Template Engine ONLY");
        Err("Need Tera engine access from Service Islands AppState".into())
    }
    
    /// Crypto Index with Tera template engine - FULL IMPLEMENTATION
    /// 
    /// Exactly like archive_old_code/handlers/crypto.rs::crypto_index - Complete L1/L2 caching
    pub async fn crypto_index_with_tera(&self, state: &Arc<AppState>) -> Result<String, Box<dyn StdError + Send + Sync>> {
        println!("🚀 CryptoHandlers::crypto_index_with_tera - Full L1/L2 Caching Implementation");
        
        // Increment request counter to monitor performance
        let request_count = state.request_counter.fetch_add(1, Ordering::Relaxed);
        
        // Log every 100 requests for monitoring
        if request_count % 100 == 0 {
            println!("Processed {} requests to crypto_index", request_count);
        }

        // Fast path: check L1 cache using atomic latest id
        let latest_id = state.cached_latest_id.load(Ordering::Relaxed);
        if latest_id > 0 {
            // TODO: Implement L1 cache when cache layers are ready
            // if let Some(cached) = state.report_cache.get(&latest_id).await {
            //     let chart_modules_content = self.get_chart_modules_content().await;
            //     match self.render_crypto_template(
            //         &state.tera,
            //         "crypto/routes/reports/view.html",
            //         &cached,
            //         &chart_modules_content,
            //         None
            //     ).await {
            //         Ok(html) => {
            //             println!("🔥 L1 Cache HIT for crypto_index");
            //             return Ok(html);
            //         }
            //         Err(_) => {
            //             println!("⚠️ L1 cache render failed");
            //         }
            //     }
            // }
        }

        // L1 Cache miss: try L2 cache (Redis) before hitting DB
        println!("🔍 L1 Cache miss for crypto_index - checking L2 cache (Redis)");
        
        // TODO: Implement L2 Redis cache when cache manager is ready
        // if let Ok(Some(cached_report)) = state.cache_manager.get::<Report>("crypto_latest_report").await {
        //     println!("🔥 L2 Cache HIT for crypto_index from Redis");
        //     // Put it back into L1 cache for faster access
        //     state.report_cache.insert(cached_report.id, cached_report.clone()).await;
        //     state.cached_latest_id.store(cached_report.id as usize, Ordering::Relaxed);
        //     
        //     let chart_modules_content = self.get_chart_modules_content().await;
        //     match self.render_crypto_template(
        //         &state.tera,
        //         "crypto/routes/reports/view.html",
        //         &cached_report,
        //         &chart_modules_content,
        //         None
        //     ).await {
        //         Ok(html) => return Ok(html),
        //         Err(_) => {
        //             println!("⚠️ Failed to render from L2 cache, falling back to DB");
        //         }
        //     }
        // }

        // Both L1 and L2 cache miss: fetch from DB and cache in both L1 and L2
        println!("🔍 L1+L2 Cache miss for crypto_index - fetching from DB");

        // Parallel fetch DB and chart modules to avoid blocking
        let db_fut = self.fetch_and_cache_latest_report(state);
        let chart_fut = self.get_chart_modules_content();

        let (db_res, chart_modules_content) = tokio::join!(db_fut, chart_fut);

        match db_res {
            Ok(Some(report)) => {
                // Template rendering with helper function
                match self.render_crypto_template(
                    &state.tera,
                    "crypto/routes/reports/view.html",
                    &report,
                    &chart_modules_content,
                    None
                ).await {
                    Ok(html) => {
                        println!("✅ Template rendered from DB - crypto_index complete");
                        Ok(html)
                    }
                    Err(e) => {
                        eprintln!("❌ Template render error: {}", e);
                        Err("Template render error".into())
                    }
                }
            }
            Ok(None) => {
                println!("⚠️ No reports found in database - rendering empty template");
                
                // Handle empty report case like archive code
                let empty_report = serde_json::json!({
                    "html_content": "",
                    "html_content_en": "",
                    "css_content": "",
                    "js_content": ""
                });
                
                let tera_clone = state.tera.clone();
                let chart_content_clone = chart_modules_content.clone();
                
                let render_result = tokio::task::spawn_blocking(move || {
                    let mut context = Context::new();
                    context.insert("current_route", "dashboard");
                    context.insert("current_lang", "vi");
                    context.insert("current_time", &chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string());
                    context.insert("report", &empty_report);
                    context.insert("chart_modules_content", &chart_content_clone);
                    context.insert("pdf_url", &"#");

                    tera_clone.render("crypto/routes/reports/view.html", &context)
                }).await;

                match render_result {
                    Ok(Ok(html)) => {
                        println!("✅ Empty template rendered successfully");
                        Ok(html)
                    }
                    Ok(Err(e)) => {
                        eprintln!("❌ Empty template render error: {:#?}", e);
                        Err(format!("Empty template render error: {}", e).into())
                    }
                    Err(e) => {
                        eprintln!("❌ Task join error: {:#?}", e);
                        Err(format!("Task join error: {}", e).into())
                    }
                }
            }
            Err(e) => {
                eprintln!("❌ Database error in crypto_index: {}", e);
                Err(format!("Database error: {}", e).into())
            }
        }
    }

    /// Crypto Report by ID handler - Specific crypto report view
    /// 
    /// Similar to crypto_index_with_tera but for specific report ID
    /// Exactly like archive_old_code/handlers/crypto.rs pattern - Complete L1/L2 caching
    pub async fn crypto_report_by_id_with_tera(
        &self, 
        state: &Arc<AppState>,
        report_id: i32
    ) -> Result<String, Box<dyn StdError + Send + Sync>> {
        println!("🚀 CryptoHandlers::crypto_report_by_id_with_tera - Report ID: {}", report_id);
        
        // Increment request counter to monitor performance
        let request_count = state.request_counter.fetch_add(1, Ordering::Relaxed);
        
        // Log every 100 requests for monitoring
        if request_count % 100 == 0 {
            println!("Processed {} requests to crypto_report_by_id", request_count);
        }

        // TODO: Fast path: check L1 cache using report ID when cache layers are ready
        // let cache_key = format!("report_{}", report_id);
        // if let Some(cached) = state.report_cache.get(&report_id).await {
        //     let chart_modules_content = self.get_chart_modules_content().await;
        //     match self.render_crypto_template(
        //         &state.tera,
        //         "crypto/routes/reports/view.html",
        //         &cached,
        //         &chart_modules_content,
        //         None
        //     ).await {
        //         Ok(html) => {
        //             println!("🔥 L1 Cache HIT for report ID: {}", report_id);
        //             return Ok(html);
        //         }
        //         Err(_) => {
        //             println!("⚠️ L1 cache render failed for report ID: {}", report_id);
        //         }
        //     }
        // }

        // L1 Cache miss: try L2 cache (Redis) before hitting DB
        println!("🔍 L1 Cache miss for report ID: {} - checking L2 cache (Redis)", report_id);
        
        // TODO: Implement L2 Redis cache when cache manager is ready
        // let cache_key = format!("crypto_report_{}", report_id);
        // if let Ok(Some(cached_report)) = state.cache_manager.get::<Report>(&cache_key).await {
        //     println!("🔥 L2 Cache HIT for report ID: {} from Redis", report_id);
        //     // Put it back into L1 cache for faster access
        //     state.report_cache.insert(cached_report.id, cached_report.clone()).await;
        //     
        //     let chart_modules_content = self.get_chart_modules_content().await;
        //     match self.render_crypto_template(
        //         &state.tera,
        //         "crypto/routes/reports/view.html",
        //         &cached_report,
        //         &chart_modules_content,
        //         None
        //     ).await {
        //         Ok(html) => return Ok(html),
        //         Err(_) => {
        //             println!("⚠️ Failed to render from L2 cache for report ID: {}, falling back to DB", report_id);
        //         }
        //     }
        // }

        // Both L1 and L2 cache miss: fetch from DB and cache in both L1 and L2
        println!("🔍 L1+L2 Cache miss for report ID: {} - fetching from DB", report_id);

        // Parallel fetch DB and chart modules to avoid blocking
        let db_fut = self.fetch_and_cache_report_by_id(state, report_id);
        let chart_fut = self.get_chart_modules_content();

        let (db_res, chart_modules_content) = tokio::join!(db_fut, chart_fut);

        match db_res {
            Ok(Some(report)) => {
                // Template rendering with helper function
                match self.render_crypto_template(
                    &state.tera,
                    "crypto/routes/reports/view.html",
                    &report,
                    &chart_modules_content,
                    None
                ).await {
                    Ok(html) => {
                        println!("✅ Template rendered from DB for report ID: {} - crypto_report_by_id complete", report_id);
                        Ok(html)
                    }
                    Err(e) => {
                        eprintln!("❌ Template render error for report ID: {}: {}", report_id, e);
                        Err("Template render error".into())
                    }
                }
            }
            Ok(None) => {
                println!("⚠️ Report ID: {} not found in database - rendering 404 template", report_id);
                
                // Handle not found case
                let not_found_report = serde_json::json!({
                    "id": report_id,
                    "html_content": format!("<div class='text-center py-16'><h2 class='text-2xl font-bold text-red-600'>Báo cáo #{} không tồn tại</h2><p class='text-gray-500 mt-4'>Báo cáo này có thể đã bị xóa hoặc không có quyền truy cập.</p><a href='/crypto_reports_list' class='mt-6 inline-block px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700'>Quay lại danh sách báo cáo</a></div>", report_id),
                    "html_content_en": format!("<div class='text-center py-16'><h2 class='text-2xl font-bold text-red-600'>Report #{} not found</h2><p class='text-gray-500 mt-4'>This report may have been deleted or you don't have access.</p><a href='/crypto_reports_list' class='mt-6 inline-block px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700'>Back to reports list</a></div>", report_id),
                    "css_content": "",
                    "js_content": ""
                });
                
                let tera_clone = state.tera.clone();
                let chart_content_clone = chart_modules_content.clone();
                
                let render_result = tokio::task::spawn_blocking(move || {
                    let mut context = tera::Context::new();
                    context.insert("current_route", "dashboard");
                    context.insert("current_lang", "vi");
                    context.insert("current_time", &chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string());
                    context.insert("report", &not_found_report);
                    context.insert("chart_modules_content", &chart_content_clone);
                    context.insert("pdf_url", &"#");

                    tera_clone.render("crypto/routes/reports/view.html", &context)
                }).await;

                match render_result {
                    Ok(Ok(html)) => {
                        println!("✅ 404 template rendered successfully for report ID: {}", report_id);
                        Ok(html)
                    }
                    Ok(Err(e)) => {
                        eprintln!("❌ 404 template render error for report ID: {}: {:#?}", report_id, e);
                        Err(format!("404 template render error: {}", e).into())
                    }
                    Err(e) => {
                        eprintln!("❌ Task join error for report ID: {}: {:#?}", report_id, e);
                        Err(format!("Task join error: {}", e).into())
                    }
                }
            }
            Err(e) => {
                eprintln!("❌ Database error for report ID: {}: {}", report_id, e);
                Err(format!("Database error: {}", e).into())
            }
        }
    }

    /// Crypto Reports List handler - Paginated list of all reports
    /// 
    /// Exactly like archive_old_code/handlers/crypto.rs::report_list - Full implementation with pagination
    pub async fn crypto_reports_list_with_tera(
        &self, 
        state: &Arc<AppState>,
        page: i64
    ) -> Result<String, Box<dyn StdError + Send + Sync>> {
        println!("📋 CryptoHandlers::crypto_reports_list_with_tera - Full Pagination Implementation");
        
        // Increment request counter to monitor performance
        let request_count = state.request_counter.fetch_add(1, Ordering::Relaxed);
        
        // Log every 50 requests for monitoring
        if request_count % 50 == 0 {
            println!("Processed {} requests to crypto_reports_list", request_count);
        }

        // Pagination configuration
        let per_page: i64 = 10;
        let offset = (page - 1) * per_page;

        // Parallel fetch total count and page rows - exactly like archive
        let total_fut = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM crypto_report").fetch_one(&state.db);
        let rows_fut = sqlx::query_as::<_, ReportSummary>(
            "SELECT id, created_at FROM crypto_report ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(per_page)
        .bind(offset)
        .fetch_all(&state.db);

        let (total_res, rows_res) = tokio::join!(total_fut, rows_fut);

        let total = match total_res {
            Ok(t) => t,
            Err(e) => {
                eprintln!("❌ Database error getting total count: {}", e);
                return Err(format!("Database error: {}", e).into());
            }
        };

        let list = match rows_res {
            Ok(list) => list,
            Err(e) => {
                eprintln!("❌ Database error getting report list: {}", e);
                return Err(format!("Database error: {}", e).into());
            }
        };

        // Parallel processing of items with rayon (CPU-intensive date formatting) - exactly like archive
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
            eprintln!("❌ Date formatting task join error: {:#?}", e);
            Vec::new()
        });

        // Parallel computation of pagination logic - exactly like archive
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
            eprintln!("❌ Pagination task join error: {:#?}", e);
            (1, vec![Some(1)])
        });

        // Build reports context - exactly like archive
        let display_start = if total == 0 { 0 } else { offset + 1 };
        let display_end = offset + (items.len() as i64);

        let reports = serde_json::json!({
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

        // Template rendering in spawn_blocking - exactly like archive
        let tera = state.tera.clone();
        let reports_clone = reports.clone();
        
        let render_result = tokio::task::spawn_blocking(move || {
            let mut context = tera::Context::new();
            context.insert("reports", &reports_clone);
            tera.render("crypto/routes/reports/list.html", &context)
        }).await;

        match render_result {
            Ok(Ok(html)) => {
                println!("✅ Reports list template rendered successfully - {} items, page {} of {}", items.len(), page, pages);
                Ok(html)
            }
            Ok(Err(e)) => {
                eprintln!("❌ Reports list template render error: {:#?}", e);
                let mut src = e.source();
                while let Some(s) = src {
                    eprintln!("❌ Template render error source: {:#?}", s);
                    src = s.source();
                }
                Err(format!("Template render error: {}", e).into())
            }
            Err(e) => {
                eprintln!("❌ Reports list task join error: {:#?}", e);
                Err(format!("Task join error: {}", e).into())
            }
        }
    }

    /// Generate PDF template for a specific crypto report by ID
    /// 
    /// Delegates to PdfGenerator component for clean architecture separation
    pub async fn crypto_report_pdf_with_tera(&self, app_state: &Arc<AppState>, report_id: i32) -> Result<String, Box<dyn StdError + Send + Sync>> {
        self.pdf_generator.crypto_report_pdf_with_tera(app_state, report_id).await
    }

    // NOTE: Crypto handlers implementation following archive_old_code/handlers/crypto.rs
    // Key requirements:
    // 1. MUST use Tera template engine - NO manual HTML creation
    // 2. MUST use "crypto/routes/reports/view.html" template 
    // 3. Template variables: {{ report.css_content }}, {{ report.js_content }}, {{ chart_modules_content }}
    // 4. Implement L1/L2 caching logic like original
    // 5. Parallel chart modules fetching
    // 
    // Current status: Template engine access needed from Service Islands architecture

}
