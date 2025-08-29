//! Crypto Data Service
//! 
//! Layer 3 data communication service for crypto reports.
//! Handles all database operations for crypto reports, isolating business logic
//! from infrastructure concerns.

use serde::{Serialize, Deserialize};
use sqlx::{FromRow};
use std::sync::Arc;
use rayon::prelude::*; // Add rayon for parallel processing

// Import from current state - will be refactored when lower layers are implemented
use crate::service_islands::layer1_infrastructure::AppState;

/// Report model for data layer - matches business logic model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ReportData {
    pub id: i32,
    pub html_content: String,
    pub css_content: Option<String>,
    pub js_content: Option<String>,
    pub html_content_en: Option<String>,
    pub js_content_en: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Report summary for data layer
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ReportSummaryData {
    pub id: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Crypto Data Service
/// 
/// Layer 3 service responsible for all crypto report database operations.
/// Acts as the communication layer between business logic and infrastructure.
#[derive(Clone)]
pub struct CryptoDataService {
    // Service state will be added here when needed
}

impl CryptoDataService {
    /// Create a new CryptoDataService
    pub fn new() -> Self {
        Self {
            // Initialize service
        }
    }

    /// Fetch latest crypto report from database with intelligent caching (L1+L2)
    /// 
    /// Pure data layer operation with dual cache integration - checks L1 cache first, then L2, then database
    pub async fn fetch_latest_report(
        &self,
        state: &Arc<AppState>,
    ) -> Result<Option<ReportData>, sqlx::Error> {
        let cache_key = "crypto_latest_report_data";
        
        // Try cache first if available (L1 then L2 fallback with promotion) - OPTIMIZED
        if let Some(ref cache_system) = state.cache_system {
            match cache_system.cache_manager.get(cache_key).await {
                Ok(Some(cached_data)) => {
                    match serde_json::from_value::<ReportData>(cached_data) {
                        Ok(cached_report) => {
                            println!("🔥 CryptoDataService: L1 Cache HIT for latest report {}", cached_report.id);
                            return Ok(Some(cached_report));
                        }
                        Err(e) => {
                            println!("⚠️ CryptoDataService: L1 Cache deserialization error: {}", e);
                        }
                    }
                }
                Ok(None) => {
                    println!("🔍 CryptoDataService: L1 Cache MISS for latest report");
                }
                Err(e) => {
                    println!("⚠️ CryptoDataService: L1 Cache access error: {}", e);
                }
            }
        }
        
        println!("🗄️ CryptoDataService: Fetching latest crypto report from database");
        
        let report = sqlx::query_as::<_, ReportData>(
            "SELECT id, html_content, css_content, js_content, html_content_en, js_content_en, created_at FROM crypto_report ORDER BY created_at DESC LIMIT 1",
        ).fetch_optional(&state.db).await?;
        
        if let Some(ref report) = report {
            println!("📊 CryptoDataService: Retrieved latest crypto report {} from database", report.id);
            
            // Cache the result for 10 minutes in both L1 and L2
            if let Some(ref cache_system) = state.cache_system {
                if let Ok(report_json) = serde_json::to_value(report) {
                    match cache_system.cache_manager.set_with_strategy(
                        cache_key, 
                        report_json,
                        crate::service_islands::layer1_infrastructure::cache_system_island::cache_manager::CacheStrategy::ShortTerm // 5 minutes
                    ).await {
                        Ok(_) => println!("💾 CryptoDataService: Cached latest report {} for 5 minutes", report.id),
                        Err(e) => println!("⚠️ CryptoDataService: L1 Cache set error: {}", e),
                    }
                }
            }
        } else {
            println!("📭 CryptoDataService: No crypto reports found in database");
        }
        
        Ok(report)
    }

    /// Fetch crypto report by ID from database with intelligent caching (L1+L2)
    /// 
    /// Pure data layer operation with dual cache integration - retrieves specific report by ID
    pub async fn fetch_report_by_id(
        &self,
        state: &Arc<AppState>,
        report_id: i32,
    ) -> Result<Option<ReportData>, sqlx::Error> {
        let cache_key = format!("crypto_report_data_{}", report_id);
        
        // Try cache first if available (L1 then L2 fallback with promotion) - OPTIMIZED
        if let Some(ref cache_system) = state.cache_system {
            match cache_system.cache_manager.get(&cache_key).await {
                Ok(Some(cached_data)) => {
                    match serde_json::from_value::<ReportData>(cached_data) {
                        Ok(cached_report) => {
                            println!("🔥 CryptoDataService: Cache HIT for report {}", report_id);
                            return Ok(Some(cached_report));
                        }
                        Err(e) => {
                            println!("⚠️ CryptoDataService: L1 Cache deserialization error for report {}: {}", report_id, e);
                        }
                    }
                }
                Ok(None) => {
                    println!("🔍 CryptoDataService: L1 Cache MISS for report {}", report_id);
                }
                Err(e) => {
                    println!("⚠️ CryptoDataService: L1 Cache access error for report {}: {}", report_id, e);
                }
            }
        }
        
        println!("🗄️ CryptoDataService: Fetching crypto report {} from database", report_id);
        
        let report = sqlx::query_as::<_, ReportData>(
            "SELECT id, html_content, css_content, js_content, html_content_en, js_content_en, created_at FROM crypto_report WHERE id = $1",
        )
        .bind(report_id)
        .fetch_optional(&state.db).await?;
        
        if let Some(ref report) = report {
            println!("📊 CryptoDataService: Retrieved crypto report {} from database", report.id);
            
            // Cache the result for 10 minutes in both L1 and L2
            if let Some(ref cache_system) = state.cache_system {
                if let Ok(report_json) = serde_json::to_value(report) {
                    match cache_system.cache_manager.set_with_strategy(
                        &cache_key, 
                        report_json,
                        crate::service_islands::layer1_infrastructure::cache_system_island::cache_manager::CacheStrategy::ShortTerm // 10 minutes
                    ).await {
                        Ok(_) => println!("💾 CryptoDataService: Cached report {} for 10 minutes", report.id),
                        Err(e) => println!("⚠️ CryptoDataService: L1 Cache set error for report {}: {}", report.id, e),
                    }
                }
            }
        } else {
            println!("📭 CryptoDataService: Crypto report {} not found in database", report_id);
        }
        
        Ok(report)
    }

    /// Get rendered report HTML (cached)
    #[allow(dead_code)]
    pub async fn get_rendered_report_html(&self, state: &Arc<AppState>, report_id: &str) -> Result<Option<String>, anyhow::Error> {
        if let Some(ref cache_system) = state.cache_system {
            let cache_key = format!("rendered_html_report_{}", report_id);
            // Chỉ cần lấy dưới dạng String, không cần deserialize
            if let Ok(Some(cached_value)) = cache_system.cache_manager.get(&cache_key).await {
                if let Some(html_string) = cached_value.as_str() {
                    let report_type = if report_id == "-1" { "latest report" } else { &format!("report #{}", report_id) };
                    println!("🔥 Layer 3: Cache HIT cho HTML đã render của {}", report_type);
                    return Ok(Some(html_string.to_string()));
                }
            }
        }
        Ok(None)
    }

    /// Lấy nội dung compressed data của một report từ cache.
    pub async fn get_rendered_report_compressed(&self, state: &Arc<AppState>, report_id: i32) -> Result<Option<Vec<u8>>, anyhow::Error> {
        if let Some(ref cache_system) = state.cache_system {
            let cache_key = format!("compressed_report_{}", report_id);
            if let Ok(Some(cached_value)) = cache_system.cache_manager.get(&cache_key).await {
                if let Ok(compressed_bytes) = serde_json::from_value::<Vec<u8>>(cached_value) {
                    let report_type = if report_id == -1 { "latest report" } else { &format!("report #{}", report_id) };
                    println!("🔥 Layer 3: Cache HIT cho compressed data của {}", report_type);
                    return Ok(Some(compressed_bytes));
                }
            }
        }
        Ok(None)
    }

    /// Lưu nội dung HTML đã render của một report vào cache.
    #[allow(dead_code)]
    pub async fn cache_rendered_report_html(&self, state: &Arc<AppState>, report_id: i32, html_content: String) -> Result<(), anyhow::Error> {
        if let Some(ref cache_system) = state.cache_system {
            let cache_key = format!("rendered_html_report_{}", report_id);
            // Sử dụng chiến lược cache ShortTerm (5 phút) cho HTML đã render
            let strategy = crate::service_islands::layer1_infrastructure::cache_system_island::cache_manager::CacheStrategy::ShortTerm;
            cache_system.cache_manager.set_with_strategy(&cache_key, serde_json::json!(html_content), strategy).await?;
            let report_type = if report_id == -1 { "latest report" } else { &format!("report #{}", report_id) };
            println!("💾 Layer 3: Đã cache HTML đã render cho {}", report_type);
        }
        Ok(())
    }

    /// Lưu nội dung compressed data của một report vào cache.
    pub async fn cache_rendered_report_compressed(&self, state: &Arc<AppState>, report_id: i32, compressed_data: Vec<u8>) -> Result<(), anyhow::Error> {
        if let Some(ref cache_system) = state.cache_system {
            let cache_key = format!("compressed_report_{}", report_id);
            // Sử dụng chiến lược cache ShortTerm (5 phút) cho compressed data vì nó đã được optimize
            let strategy = crate::service_islands::layer1_infrastructure::cache_system_island::cache_manager::CacheStrategy::ShortTerm;
            let compressed_json = serde_json::to_value(&compressed_data).unwrap_or_default();
            cache_system.cache_manager.set_with_strategy(&cache_key, compressed_json, strategy).await?;
            let report_type = if report_id == -1 { "latest report" } else { &format!("report #{}", report_id) };
            let size_kb = compressed_data.len() / 1024;
            println!("💾 Layer 3: Đã cache compressed data cho {} ({}KB)", report_type, size_kb);
        }
        Ok(())
    }

    /// Fetch paginated crypto reports list with caching
    /// 
    /// Layer 3 method for fetching reports list with L1/L2 cache integration
    pub async fn fetch_reports_list_with_cache(
        &self,
        state: &Arc<AppState>,
        page: i64,
        per_page: i64,
    ) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        let cache_key = format!("crypto_reports_list_page_{}", page);
        
        // BƯỚC 1: Kiểm tra cache trước (L1 cache)
        if let Some(ref cache_system) = state.cache_system {
            match cache_system.cache_manager.get(&cache_key).await {
                Ok(Some(cached_value)) => {
                    if let Some(html_string) = cached_value.as_str() {
                        println!("🔥 Layer 3: Cache HIT cho reports list page {}", page);
                        return Ok(Some(html_string.to_string()));
                    }
                }
                Ok(None) => {
                    println!("🔍 Layer 3: Cache MISS cho reports list page {}", page);
                }
                Err(e) => {
                    println!("⚠️ Layer 3: Cache access error cho page {}: {}", page, e);
                }
            }
        }

        // BƯỚC 2: Nếu cache miss, fetch từ database và render
        let offset = (page - 1) * per_page;

        // Parallel fetch total count và page rows - giống archive
        let total_fut = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM crypto_report").fetch_one(&state.db);
        let rows_fut = sqlx::query_as::<_, ReportSummaryData>(
            "SELECT id, created_at FROM crypto_report ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(per_page)
        .bind(offset)
        .fetch_all(&state.db);

        let (total_res, rows_res) = tokio::join!(total_fut, rows_fut);

        let total = match total_res {
            Ok(t) => t,
            Err(e) => {
                println!("❌ Layer 3: Database error getting total count: {}", e);
                return Err(format!("Database error: {}", e).into());
            }
        };

        let list = match rows_res {
            Ok(list) => list,
            Err(e) => {
                println!("❌ Layer 3: Database error getting report list: {}", e);
                return Err(format!("Database error: {}", e).into());
            }
        };

        println!("📊 Layer 3: Fetched {} reports for page {}, total: {}", list.len(), page, total);
        
        // BƯỚC 3: Xử lý và format data
        let items: Vec<serde_json::Value> = tokio::task::spawn_blocking(move || {
            use rayon::prelude::*;
            
            list.par_iter()
                .map(|r| {
                    let dt = r.created_at + chrono::Duration::hours(7);
                    let created_date = dt.format("%d/%m/%Y").to_string();
                    let created_time = format!("{} UTC+7", dt.format("%H:%M:%S"));
                    serde_json::json!({ 
                        "id": r.id, 
                        "created_date": created_date, 
                        "created_time": created_time 
                    })
                })
                .collect()
        }).await.unwrap_or_else(|e| {
            println!("❌ Layer 3: Date formatting task join error: {:#?}", e);
            Vec::new()
        });

        // BƯỚC 4: Tính toán pagination
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
            println!("❌ Layer 3: Pagination task join error: {:#?}", e);
            (1, vec![Some(1)])
        });

        // BƯỚC 5: Build reports context
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

        // BƯỚC 6: Template rendering
        let tera = state.tera.clone();
        let reports_clone = reports.clone();
        
        let render_result = tokio::task::spawn_blocking(move || {
            let mut context = tera::Context::new();
            context.insert("reports", &reports_clone);
            tera.render("crypto/routes/reports/list.html", &context)
        }).await;

        match render_result {
            Ok(Ok(html)) => {
                println!("✅ Layer 3: Reports list template rendered successfully - {} items, page {} of {}", items.len(), page, pages);
                
                // BƯỚC 7: Cache rendered HTML
                if let Some(ref cache_system) = state.cache_system {
                    let strategy = crate::service_islands::layer1_infrastructure::cache_system_island::cache_manager::CacheStrategy::ShortTerm;
                    if let Err(e) = cache_system.cache_manager.set_with_strategy(&cache_key, serde_json::json!(html), strategy).await {
                        println!("⚠️ Layer 3: Failed to cache reports list page {}: {}", page, e);
                    } else {
                        println!("💾 Layer 3: Cached reports list page {} for 5 minutes", page);
                    }
                }
                
                Ok(Some(html))
            }
            Ok(Err(e)) => {
                println!("❌ Layer 3: Reports list template render error: {:#?}", e);
                Err(format!("Template render error: {}", e).into())
            }
            Err(e) => {
                println!("❌ Layer 3: Reports list task join error: {:#?}", e);
                Err(format!("Task join error: {}", e).into())
            }
        }
    }
}
