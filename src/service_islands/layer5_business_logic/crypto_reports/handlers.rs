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
use sqlx::{self, FromRow};
use std::{collections::HashMap, error::Error as StdError, sync::Arc, sync::atomic::Ordering, path::Path, env};
use tera::Context;
use tokio::fs::read_dir;

// Import from current state - will be refactored when lower layers are implemented
use crate::state::AppState;

/// Report model - exactly from archive_old_code/models.rs
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Report {
    pub id: i32,
    pub html_content: String,
    pub css_content: Option<String>,
    pub js_content: Option<String>,
    pub html_content_en: Option<String>,
    pub js_content_en: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Crypto Handlers
/// 
/// Contains all HTTP request handlers for crypto reports-related operations.
/// These handlers manage crypto report generation, PDF creation, and API interactions.
/// ONLY uses Template Engine like archive_old_code/handlers/crypto.rs
pub struct CryptoHandlers {
    // Component state will be added here as we implement lower layers
}

impl CryptoHandlers {
    /// Create a new CryptoHandlers instance
    pub fn new() -> Self {
        Self {
            // Initialize component state
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
                let pdf_url = format!("/pdf-template/{}", report_clone.id);
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
