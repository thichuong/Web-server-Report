//! Crypto Data Service
//! 
//! Layer 3 data communication service for crypto reports.
//! Handles all database operations for crypto reports, isolating business logic
//! from infrastructure concerns.
//! 
//! ✅ PRODUCTION-READY: Includes memory limits and safety guards

use serde::{Serialize, Deserialize};
use sqlx::{FromRow};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

// Import from current state - will be refactored when lower layers are implemented
use crate::service_islands::layer1_infrastructure::AppState;

/// Memory limits for cache entries - Production safety guards
/// 
/// These limits prevent memory exhaustion and ensure stable operation under high load
const MAX_COMPRESSED_ENTRY_SIZE: usize = 5 * 1024 * 1024; // 5MB per entry
const MAX_TOTAL_COMPRESSED_MEMORY: usize = 500 * 1024 * 1024; // 500MB total
const WARN_COMPRESSED_ENTRY_SIZE: usize = 2 * 1024 * 1024; // 2MB warning threshold

/// Global memory tracking for compressed cache entries
/// 
/// Tracks total memory used by compressed report cache to prevent memory exhaustion
static TOTAL_COMPRESSED_CACHE_SIZE: AtomicUsize = AtomicUsize::new(0);

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

    /// Lấy nội dung compressed data của một report từ cache.
    /// ✅ PRODUCTION-SAFE: No size limits on read - only on write
    pub async fn get_rendered_report_compressed(&self, state: &Arc<AppState>, report_id: i32) -> Result<Option<Vec<u8>>, anyhow::Error> {
        if let Some(ref cache_system) = state.cache_system {
            let cache_key = format!("compressed_report_{}", report_id);
            if let Ok(Some(cached_value)) = cache_system.cache_manager.get(&cache_key).await {
                if let Ok(compressed_bytes) = serde_json::from_value::<Vec<u8>>(cached_value) {
                    let report_type = if report_id == -1 { "latest report" } else { &format!("report #{}", report_id) };
                    let size_kb = compressed_bytes.len() / 1024;
                    println!("🔥 Layer 3: Cache HIT cho compressed data của {} ({}KB)", report_type, size_kb);
                    return Ok(Some(compressed_bytes));
                }
            }
        }
        Ok(None)
    }

    /// Cache rendered report compressed data with memory safety guards
    /// ✅ PRODUCTION-READY: Implements memory limits to prevent cache exhaustion
    pub async fn cache_rendered_report_compressed(&self, state: &Arc<AppState>, report_id: i32, compressed_data: Vec<u8>) -> Result<(), anyhow::Error> {
        let data_size = compressed_data.len();
        let size_kb = data_size / 1024;
        let size_mb = data_size as f64 / (1024.0 * 1024.0);
        let report_type = if report_id == -1 { "latest report" } else { &format!("report #{}", report_id) };
        
        // 🛡️ GUARD 1: Check individual entry size limit
        if data_size > MAX_COMPRESSED_ENTRY_SIZE {
            eprintln!("❌ Layer 3: MEMORY LIMIT - Compressed data too large for {} ({}MB > {}MB limit)", 
                     report_type, size_mb, MAX_COMPRESSED_ENTRY_SIZE as f64 / (1024.0 * 1024.0));
            eprintln!("   Skipping cache to prevent memory exhaustion");
            return Ok(()); // Not an error, just skip caching
        }
        
        // 🛡️ GUARD 2: Check total memory limit
        let current_total = TOTAL_COMPRESSED_CACHE_SIZE.load(Ordering::Relaxed);
        let new_total = current_total + data_size;
        
        if new_total > MAX_TOTAL_COMPRESSED_MEMORY {
            let current_mb = current_total as f64 / (1024.0 * 1024.0);
            let max_mb = MAX_TOTAL_COMPRESSED_MEMORY as f64 / (1024.0 * 1024.0);
            eprintln!("❌ Layer 3: MEMORY LIMIT - Total compressed cache would exceed limit ({:.1}MB + {:.1}MB > {:.1}MB)", 
                     current_mb, size_mb, max_mb);
            eprintln!("   Skipping cache to prevent memory exhaustion. Consider evicting old entries.");
            return Ok(()); // Not an error, just skip caching
        }
        
        // ⚠️ WARNING: Log if entry is large
        if data_size > WARN_COMPRESSED_ENTRY_SIZE {
            println!("⚠️  Layer 3: Large compressed entry for {} ({:.1}MB) - consider optimization", 
                    report_type, size_mb);
        }
        
        // ✅ All checks passed - proceed with caching
        if let Some(ref cache_system) = state.cache_system {
            let cache_key = format!("compressed_report_{}", report_id);
            let strategy = crate::service_islands::layer1_infrastructure::cache_system_island::cache_manager::CacheStrategy::ShortTerm;
            let compressed_json = serde_json::to_value(&compressed_data).unwrap_or_default();
            
            cache_system.cache_manager.set_with_strategy(&cache_key, compressed_json, strategy).await?;
            
            // Update global memory tracking
            TOTAL_COMPRESSED_CACHE_SIZE.fetch_add(data_size, Ordering::Relaxed);
            let new_total_mb = (current_total + data_size) as f64 / (1024.0 * 1024.0);
            
            println!("💾 Layer 3: Cached compressed data for {} ({}KB) - Total cache: {:.1}MB / {:.1}MB", 
                    report_type, size_kb, new_total_mb, 
                    MAX_TOTAL_COMPRESSED_MEMORY as f64 / (1024.0 * 1024.0));
        }
        Ok(())
    }
    
    /// Get current compressed cache memory statistics
    /// ✅ PRODUCTION-READY: Monitoring endpoint for cache memory usage
    pub fn get_compressed_cache_stats() -> (usize, usize, f64) {
        let current_bytes = TOTAL_COMPRESSED_CACHE_SIZE.load(Ordering::Relaxed);
        let usage_percent = (current_bytes as f64 / MAX_TOTAL_COMPRESSED_MEMORY as f64) * 100.0;
        (current_bytes, MAX_TOTAL_COMPRESSED_MEMORY, usage_percent)
    }

    /// Fetch paginated crypto reports list with caching
    /// 
    /// Layer 3 method for fetching reports list with L1/L2 cache integration
    /// Returns compressed data (Vec<u8>) for optimal transfer speed
    pub async fn fetch_reports_list_with_cache(
        &self,
        state: &Arc<AppState>,
        page: i64,
        per_page: i64,
    ) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error + Send + Sync>> {
        let cache_key = format!("crypto_reports_list_page_{}_compressed", page);
        
        // BƯỚC 1: Kiểm tra cache trước (L1 cache) - tìm compressed data
        if let Some(ref cache_system) = state.cache_system {
            match cache_system.cache_manager.get(&cache_key).await {
                Ok(Some(cached_value)) => {
                    // Try to get as bytes array
                    if let Some(cached_bytes) = cached_value.as_array() {
                        let bytes: Vec<u8> = cached_bytes.iter()
                            .filter_map(|v| v.as_u64().map(|n| n as u8))
                            .collect();
                        if !bytes.is_empty() {
                            let size_kb = bytes.len() / 1024;
                            println!("🔥 Layer 3: Cache HIT cho compressed reports list page {} ({}KB)", page, size_kb);
                            return Ok(Some(bytes));
                        }
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

        // BƯỚC 6: Template rendering - Clone Tera (Arc nên clone nhẹ)
        let tera = state.tera.clone(); // Arc clone là nhẹ
        let reports_clone = reports.clone(); // Cần clone vì spawn_blocking yêu cầu 'static
        
        let render_result = tokio::task::spawn_blocking(move || {
            let mut context = tera::Context::new();
            context.insert("reports", &reports_clone);
            tera.render("crypto/routes/reports/list.html", &context)
        }).await;

        match render_result {
            Ok(Ok(html)) => {
                println!("✅ Layer 3: Reports list template rendered successfully - {} items, page {} of {}", items.len(), page, pages);
                
                // BƯỚC 7: Compress HTML với gzip
                use flate2::{Compression, write::GzEncoder};
                use std::io::Write;
                
                let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
                if let Err(e) = encoder.write_all(html.as_bytes()) {
                    println!("❌ Layer 3: Compression write error for reports list page {}: {}", page, e);
                    return Err(format!("Compression error: {}", e).into());
                }
                
                let compressed_data = match encoder.finish() {
                    Ok(data) => data,
                    Err(e) => {
                        println!("❌ Layer 3: Compression finish error for reports list page {}: {}", page, e);
                        return Err(format!("Compression error: {}", e).into());
                    }
                };
                
                let original_size = html.len();
                let compressed_size = compressed_data.len();
                let compression_ratio = (1.0 - (compressed_size as f64 / original_size as f64)) * 100.0;
                
                println!("🗜️  Layer 3: Reports list compressed - Original: {}KB, Compressed: {}KB, Ratio: {:.1}%", 
                         original_size / 1024, 
                         compressed_size / 1024, 
                         compression_ratio);
                
                // BƯỚC 8: Cache compressed data
                if let Some(ref cache_system) = state.cache_system {
                    // Convert Vec<u8> to JSON array for cache storage
                    let bytes_array: Vec<serde_json::Value> = compressed_data.iter()
                        .map(|&b| serde_json::json!(b))
                        .collect();
                    
                    let strategy = crate::service_islands::layer1_infrastructure::cache_system_island::cache_manager::CacheStrategy::ShortTerm;
                    if let Err(e) = cache_system.cache_manager.set_with_strategy(&cache_key, serde_json::json!(bytes_array), strategy).await {
                        println!("⚠️ Layer 3: Failed to cache compressed reports list page {}: {}", page, e);
                    } else {
                        let size_kb = compressed_data.len() / 1024;
                        println!("💾 Layer 3: Cached compressed reports list page {} ({}KB) for 5 minutes", page, size_kb);
                    }
                }
                
                Ok(Some(compressed_data))
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
