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
use super::template_orchestrator::TemplateOrchestrator;

/// Crypto Handlers
/// 
/// Contains all HTTP request handlers for crypto reports-related operations.
/// These handlers manage crypto report generation, PDF creation, and API interactions.
/// ONLY uses Template Engine like archive_old_code/handlers/crypto.rs
pub struct CryptoHandlers {
    pub report_creator: ReportCreator,
    pub pdf_generator: PdfGenerator,
    pub template_orchestrator: TemplateOrchestrator,
}

impl CryptoHandlers {
    /// Create a new CryptoHandlers instance
    pub fn new() -> Self {
        let report_creator = ReportCreator::new();
        let template_orchestrator = TemplateOrchestrator::new(report_creator.clone());
        
        Self {
            report_creator,
            pdf_generator: PdfGenerator::new(),
            template_orchestrator,
        }
    }
    
    /// Health check for crypto handlers
    pub async fn health_check(&self) -> bool {
        // Verify handlers are functioning properly
        let report_creator_ok = self.report_creator.health_check().await;
        let pdf_generator_ok = self.pdf_generator.health_check().await;
        let template_orchestrator_ok = self.template_orchestrator.health_check().await;
        
        report_creator_ok && pdf_generator_ok && template_orchestrator_ok
    }

    /// Helper function for template rendering
    /// 
    /// DEPRECATED: Use template_orchestrator methods instead
    /// Kept for backward compatibility during transition
    pub async fn render_crypto_template(
        &self,
        tera: &tera::Tera, 
        template: &str,
        report: &Report,
        _chart_modules_content: &str, // Not used anymore, handled by orchestrator
        additional_context: Option<HashMap<String, serde_json::Value>>
    ) -> Result<String, Box<dyn StdError + Send + Sync>> {
        println!("🔄 CryptoHandlers::render_crypto_template - Delegating to TemplateOrchestrator");
        
        // Delegate to TemplateOrchestrator for proper separation of concerns
        match template {
            path if path.contains("view.html") => {
                self.template_orchestrator.render_crypto_report_view(
                    tera,
                    report,
                    additional_context
                ).await
            }
            path if path.contains("pdf.html") => {
                self.template_orchestrator.render_crypto_report_pdf(
                    tera,
                    report
                ).await
            }
            _ => {
                // Generic template rendering
                let context = self.template_orchestrator.prepare_crypto_report_context(
                    report,
                    template,
                    additional_context
                ).await?;
                
                self.template_orchestrator.render_template(
                    tera,
                    template,
                    context
                ).await
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

        // Parallel fetch DB and chart modules to avoid blocking - use ReportCreator for data logic
        let db_fut = self.report_creator.fetch_and_cache_latest_report(state);
        let chart_fut = self.report_creator.get_chart_modules_content();

        let (db_res, chart_modules_content) = tokio::join!(db_fut, chart_fut);

        match db_res {
            Ok(Some(report)) => {
                // Template rendering with TemplateOrchestrator
                match self.template_orchestrator.render_crypto_report_view(
                    &state.tera,
                    &report,
                    None
                ).await {
                    Ok(html) => {
                        println!("✅ Template rendered from DB via TemplateOrchestrator - crypto_index complete");
                        Ok(html)
                    }
                    Err(e) => {
                        eprintln!("❌ TemplateOrchestrator render error: {}", e);
                        Err("Template render error".into())
                    }
                }
            }
            Ok(None) => {
                println!("⚠️ No reports found in database - rendering empty template via TemplateOrchestrator");
                
                // Use TemplateOrchestrator for empty template
                match self.template_orchestrator.render_empty_template(&state.tera).await {
                    Ok(html) => {
                        println!("✅ Empty template rendered successfully via TemplateOrchestrator");
                        Ok(html)
                    }
                    Err(e) => {
                        eprintln!("❌ TemplateOrchestrator empty template render error: {}", e);
                        Err(format!("Empty template render error: {}", e).into())
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

        // Parallel fetch DB and chart modules to avoid blocking - use ReportCreator for data logic
        let db_fut = self.report_creator.fetch_and_cache_report_by_id(state, report_id);
        let chart_fut = self.report_creator.get_chart_modules_content();

        let (db_res, chart_modules_content) = tokio::join!(db_fut, chart_fut);

        match db_res {
            Ok(Some(report)) => {
                // Template rendering with TemplateOrchestrator
                match self.template_orchestrator.render_crypto_report_view(
                    &state.tera,
                    &report,
                    None
                ).await {
                    Ok(html) => {
                        println!("✅ Template rendered from DB via TemplateOrchestrator for report ID: {} - crypto_report_by_id complete", report_id);
                        Ok(html)
                    }
                    Err(e) => {
                        eprintln!("❌ TemplateOrchestrator render error for report ID: {}: {}", report_id, e);
                        Err("Template render error".into())
                    }
                }
            }
            Ok(None) => {
                println!("⚠️ Report ID: {} not found in database - rendering 404 template via TemplateOrchestrator", report_id);
                
                // Use TemplateOrchestrator for 404 template
                match self.template_orchestrator.render_not_found_template(&state.tera, report_id).await {
                    Ok(html) => {
                        println!("✅ 404 template rendered successfully via TemplateOrchestrator for report ID: {}", report_id);
                        Ok(html)
                    }
                    Err(e) => {
                        eprintln!("❌ TemplateOrchestrator 404 template render error for report ID: {}: {}", report_id, e);
                        Err(format!("404 template render error: {}", e).into())
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
