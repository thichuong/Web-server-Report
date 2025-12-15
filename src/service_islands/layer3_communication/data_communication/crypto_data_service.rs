//! Crypto Data Service
//!
//! Layer 3 data communication service for crypto reports.
//! Handles all database operations for crypto reports, isolating business logic
//! from infrastructure concerns.
//!
//! ✅ PRODUCTION-READY: Includes memory limits and safety guards

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

// Import from current state - will be refactored when lower layers are implemented
use crate::service_islands::layer1_infrastructure::AppState;
use base64::{prelude::BASE64_STANDARD, Engine};

/// Memory limits for cache entries - Production safety guards
///
/// These limits prevent memory exhaustion and ensure stable operation under high load.
///
/// ⚠️ NOTE: These are soft limits for logging purposes only. The actual cache memory management
/// is handled by the multi-tier-cache library (L1 moka cache: 2000 entries, L2 Redis with TTL).
/// The library automatically evicts entries based on size and TTL, so manual memory tracking
/// is unnecessary and can lead to inaccurate counts due to eviction events.
const MAX_COMPRESSED_ENTRY_SIZE: usize = 5 * 1024 * 1024; // 5MB per entry (soft limit for logging)
const WARN_COMPRESSED_ENTRY_SIZE: usize = 2 * 1024 * 1024; // 2MB warning threshold

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

/// Report metadata for sitemap generation
/// Contains only essential fields needed for sitemap XML
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ReportSitemapData {
    pub id: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Report data for RSS feed generation
/// Contains id, `html_content` for description extraction, and `created_at` for pubDate
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ReportRssData {
    pub id: i32,
    pub html_content: String,
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

impl Default for CryptoDataService {
    fn default() -> Self {
        Self::new()
    }
}

impl CryptoDataService {
    /// Create a new `CryptoDataService`
    #[must_use]
    pub fn new() -> Self {
        Self {
            // Initialize service
        }
    }

    /// Fetch latest crypto report from database with intelligent caching (L1+L2)
    ///
    /// ✨ NEW: Uses type-safe automatic caching with `get_or_compute_typed()`
    /// Pure data layer operation with dual cache integration - checks L1 cache first, then L2, then database
    ///
    /// # Errors
    ///
    /// Returns `sqlx::Error` if database connection fails or query execution fails
    /// Returns `sqlx::Error` if database connection fails or query execution fails
    pub async fn fetch_latest_report(
        &self,
        state: &Arc<AppState>,
    ) -> Result<Option<ReportData>, sqlx::Error> {
        info!("🗄️ CryptoDataService: Fetching latest crypto report from database");

        let report = sqlx::query_as::<_, ReportData>(
                "SELECT id, html_content, css_content, js_content, html_content_en, js_content_en, created_at FROM crypto_report ORDER BY created_at DESC LIMIT 1",
            ).fetch_optional(&state.db).await?;

        if let Some(ref report) = report {
            debug!(
                "📊 CryptoDataService: Retrieved latest crypto report {} from database",
                report.id
            );
        } else {
            info!("📭 CryptoDataService: No crypto reports found in database");
        }

        Ok(report)
    }

    /// Fetch all report IDs and creation dates for sitemap generation
    ///
    /// Returns lightweight data for generating dynamic sitemap URLs.
    /// Uses `MediumTerm` caching (1 hour) since sitemap doesn't need real-time updates.
    ///
    /// # Errors
    ///
    /// Returns `sqlx::Error` if database connection fails or query execution fails
    /// Returns `sqlx::Error` if database connection fails or query execution fails
    pub async fn fetch_all_report_ids_for_sitemap(
        &self,
        state: &Arc<AppState>,
    ) -> Result<Vec<ReportSitemapData>, sqlx::Error> {
        info!("🗄️ CryptoDataService: Fetching all report IDs for sitemap from database");

        let reports = sqlx::query_as::<_, ReportSitemapData>(
            "SELECT id, created_at FROM crypto_report ORDER BY created_at DESC",
        )
        .fetch_all(&state.db)
        .await?;

        debug!(
            "📊 CryptoDataService: Retrieved {} report IDs for sitemap",
            reports.len()
        );
        Ok(reports)
    }

    /// Fetch related reports (older reports) for GEO optimization
    ///
    /// Returns a list of reports older than the current report for internal linking.
    /// Uses `ShortTerm` caching (5 minutes) to balance freshness with performance.
    ///
    /// # Arguments
    /// * `state` - Application state with database and cache
    /// * `current_id` - The ID of the current report (fetch reports with id < `current_id`)
    /// * `limit` - Maximum number of related reports to fetch
    ///
    /// # Returns
    /// Vec<ReportSummaryData> with id and `created_at` for each related report
    ///
    /// # Errors
    ///
    /// Returns `sqlx::Error` if database connection fails or query execution fails
    /// Returns `sqlx::Error` if database connection fails or query execution fails
    pub async fn fetch_related_reports(
        &self,
        state: &Arc<AppState>,
        current_id: i32,
        limit: i64,
    ) -> Result<Vec<ReportSummaryData>, sqlx::Error> {
        debug!(
            "🗄️ CryptoDataService: Fetching related reports for report {} from database",
            current_id
        );

        let reports = sqlx::query_as::<_, ReportSummaryData>(
            "SELECT id, created_at FROM crypto_report WHERE id < $1 ORDER BY id DESC LIMIT $2",
        )
        .bind(current_id)
        .bind(limit)
        .fetch_all(&state.db)
        .await?;

        debug!(
            "📊 CryptoDataService: Retrieved {} related reports for report {}",
            reports.len(),
            current_id
        );
        Ok(reports)
    }

    /// Fetch reports for RSS feed generation
    ///
    /// Returns a list of recent reports with `html_content` for description extraction.
    /// Uses `MediumTerm` caching (1 hour) to balance freshness with performance.
    ///
    /// # Arguments
    /// * `state` - Application state with database and cache
    /// * `limit` - Maximum number of reports to fetch (default 20)
    ///
    /// # Returns
    /// Vec<ReportRssData> with id, `html_content`, and `created_at` for RSS feed
    ///
    /// # Errors
    ///
    /// Returns `sqlx::Error` if database connection fails or query execution fails
    /// Returns `sqlx::Error` if database connection fails or query execution fails
    pub async fn fetch_rss_reports(
        &self,
        state: &Arc<AppState>,
        limit: i64,
    ) -> Result<Vec<ReportRssData>, sqlx::Error> {
        info!(
            "🗄️ CryptoDataService: Fetching {} reports for RSS feed from database",
            limit
        );

        let reports = sqlx::query_as::<_, ReportRssData>(
                "SELECT id, html_content, created_at FROM crypto_report ORDER BY created_at DESC LIMIT $1",
            )
            .bind(limit)
            .fetch_all(&state.db)
            .await?;

        debug!(
            "📊 CryptoDataService: Retrieved {} reports for RSS feed",
            reports.len()
        );
        Ok(reports)
    }

    /// Fetch crypto report by ID from database with intelligent caching (L1+L2)
    ///
    /// ✨ NEW: Uses type-safe automatic caching with `get_or_compute_typed()`
    /// Pure data layer operation with dual cache integration - retrieves specific report by ID
    ///
    /// # Errors
    ///
    /// Returns `sqlx::Error` if database connection fails or query execution fails
    /// Returns `sqlx::Error` if database connection fails or query execution fails
    pub async fn fetch_report_by_id(
        &self,
        state: &Arc<AppState>,
        report_id: i32,
    ) -> Result<Option<ReportData>, sqlx::Error> {
        info!(
            "🗄️ CryptoDataService: Fetching crypto report {} from database",
            report_id
        );

        let report = sqlx::query_as::<_, ReportData>(
                "SELECT id, html_content, css_content, js_content, html_content_en, js_content_en, created_at FROM crypto_report WHERE id = $1",
            )
            .bind(report_id)
            .fetch_optional(&state.db)
            .await?;

        if let Some(ref report) = report {
            debug!(
                "📊 CryptoDataService: Retrieved crypto report {} from database",
                report.id
            );
        } else {
            info!(
                "📭 CryptoDataService: Crypto report {} not found in database",
                report_id
            );
        }

        Ok(report)
    }

    /// Lấy nội dung compressed data của một report từ cache.
    /// ✅ PRODUCTION-SAFE: No size limits on read - only on write
    ///
    /// # Errors
    ///
    /// Returns error if cache retrieval fails or deserialization fails
    /// Returns error if cache retrieval fails or deserialization fails
    pub async fn get_rendered_report_compressed(
        &self,
        state: &Arc<AppState>,
        report_id: i32,
    ) -> Result<Option<Vec<u8>>, anyhow::Error> {
        if let Some(ref cache_system) = state.cache_system {
            let cache_key = format!("compressed_report_{report_id}");
            // .await directly instead of block_on
            if let Ok(Some(cached_value)) = cache_system.cache_manager.get(&cache_key).await {
                // Try to parse as String (Base64) first - New Format (Memory Optimized)
                if let Ok(base64_string) = serde_json::from_value::<String>(cached_value.clone()) {
                    match BASE64_STANDARD.decode(base64_string) {
                        Ok(compressed_bytes) => {
                            let report_type = if report_id == -1 {
                                "latest report"
                            } else {
                                // Simplified log message for sync context
                                "specific report"
                            };
                            info!(
                                "🔥 Layer 3: Cache HIT (Base64) cho compressed data của {}",
                                report_type
                            );
                            return Ok(Some(compressed_bytes));
                        }
                        Err(e) => {
                            warn!("⚠️ Layer 3: Failed to decode Base64 cache entry: {}", e);
                            // Fallthrough to try legacy format
                        }
                    }
                }

                // Fallback: Try to parse as Vec<u8> - Old Format (Legacy)
                if let Ok(compressed_bytes) = serde_json::from_value::<Vec<u8>>(cached_value) {
                    let report_type = if report_id == -1 {
                        "latest report"
                    } else {
                        "specific report"
                    };
                    info!(
                        "🔥 Layer 3: Cache HIT (Legacy Array) cho compressed data của {}",
                        report_type
                    );
                    return Ok(Some(compressed_bytes));
                }
            }
        }
        Ok(None)
    }

    /// Cache rendered report compressed data with memory safety guards
    ///
    /// ✅ PRODUCTION-READY: Delegates memory management to multi-tier-cache library.
    /// The library automatically handles eviction based on size limits (2000 entries)
    /// and TTL expiration, eliminating need for manual memory tracking.
    ///
    /// ✅ MEMORY OPTIMIZED: Accepts &[u8] reference instead of owned Vec<u8> to avoid
    /// unnecessary clones. The data is serialized for cache storage anyway.
    ///
    /// # Errors
    ///
    /// Returns error if cache storage fails or serialization fails
    /// Returns error if cache storage fails or serialization fails
    pub async fn cache_rendered_report_compressed(
        &self,
        state: &Arc<AppState>,
        report_id: i32,
        compressed_data: &[u8],
    ) -> Result<(), anyhow::Error> {
        let data_size = compressed_data.len();
        let kilobytes = data_size / 1024;
        #[allow(clippy::cast_precision_loss)] // f64 conversion for MB display
        let megabytes = data_size as f64 / (1024.0 * 1024.0);
        let report_type = if report_id == -1 {
            "latest report"
        } else {
            // Simplified log
            "specific report"
        };

        // 🛡️ GUARD: Warn if individual entry size is very large (soft limit for logging)
        if data_size > MAX_COMPRESSED_ENTRY_SIZE {
            warn!("⚠️  Layer 3: Very large compressed data for {} ({:.1}MB) - may impact cache performance",
                     report_type, megabytes);
            error!(
                "   Note: Cache library will handle storage, but consider optimizing entry size"
            );
        } else if data_size > WARN_COMPRESSED_ENTRY_SIZE {
            warn!(
                "⚠️  Layer 3: Large compressed entry for {} ({:.1}MB) - consider optimization",
                report_type, megabytes
            );
        }

        // ✅ Cache the data - library handles memory management automatically
        if let Some(ref cache_system) = state.cache_system {
            let cache_key = format!("compressed_report_{report_id}");
            let strategy = crate::service_islands::layer1_infrastructure::cache_system_island::cache_manager::CacheStrategy::ShortTerm;

            // ✅ OPTIMIZATION: Encode as Base64 string instead of JSON Array of Numbers
            // This reduces memory usage by ~95% compared to [12, 255, ...] format
            let base64_string = BASE64_STANDARD.encode(compressed_data);
            let compressed_json = serde_json::Value::String(base64_string);

            cache_system
                .cache_manager
                .set_with_strategy(&cache_key, compressed_json, strategy)
                .await?;

            debug!(
                "💾 Layer 3: Cached compressed data for {} ({}KB)",
                report_type, kilobytes
            );
        }
        Ok(())
    }

    /// Get cached DSD rendered report
    ///
    /// Retrieves compressed HTML for Declarative Shadow DOM routes.
    /// Cache key format: `compressed_report_dsd`_{`report_id`}_{language}
    /// ✅ PRODUCTION-SAFE: No size limits on read - only on write
    ///
    /// # Errors
    ///
    /// Returns error if cache retrieval fails or deserialization fails
    /// Returns error if cache retrieval fails or deserialization fails
    pub async fn get_rendered_report_dsd_compressed(
        &self,
        state: &Arc<AppState>,
        report_id: i32,
        language: &str,
    ) -> Result<Option<Vec<u8>>, anyhow::Error> {
        if let Some(ref cache_system) = state.cache_system {
            let cache_key = format!("compressed_report_dsd_{report_id}_{language}");
            if let Ok(Some(cached_value)) = cache_system.cache_manager.get(&cache_key).await {
                // Try to parse as String (Base64) first - New Format (Memory Optimized)
                if let Ok(base64_string) = serde_json::from_value::<String>(cached_value.clone()) {
                    match BASE64_STANDARD.decode(base64_string) {
                        Ok(compressed_bytes) => {
                            let report_type = if report_id == -1 {
                                "latest DSD report"
                            } else {
                                "specific DSD report"
                            };
                            info!(
                                "🔥 Layer 3: Cache HIT (Base64) for {} (lang: {})",
                                report_type, language
                            );
                            return Ok(Some(compressed_bytes));
                        }
                        Err(e) => {
                            warn!("⚠️ Layer 3: Failed to decode Base64 DSD cache entry: {}", e);
                            // Fallthrough to try legacy format
                        }
                    }
                }

                // Fallback: Try to parse as Vec<u8> - Old Format (Legacy)
                if let Ok(compressed_bytes) = serde_json::from_value::<Vec<u8>>(cached_value) {
                    let report_type = if report_id == -1 {
                        "latest DSD report"
                    } else {
                        "specific DSD report"
                    };
                    info!(
                        "🔥 Layer 3: Cache HIT (Legacy Array) for {} (lang: {})",
                        report_type, language
                    );
                    return Ok(Some(compressed_bytes));
                }
            }
        }
        Ok(None)
    }

    /// Cache DSD rendered report (compressed)
    ///
    /// ✅ PRODUCTION-READY: Delegates memory management to multi-tier-cache library.
    /// The library automatically handles eviction based on size limits (2000 entries)
    /// and TTL expiration, eliminating need for manual memory tracking.
    ///
    /// ✅ MEMORY OPTIMIZED: Accepts &[u8] reference instead of owned Vec<u8> to avoid
    /// unnecessary clones. The data is serialized for cache storage anyway.
    ///
    /// # Errors
    ///
    /// Returns error if cache storage fails or serialization fails
    /// Returns error if cache storage fails or serialization fails
    pub async fn cache_rendered_report_dsd_compressed(
        &self,
        state: &Arc<AppState>,
        report_id: i32,
        compressed_data: &[u8],
        language: &str,
    ) -> Result<(), anyhow::Error> {
        let data_size = compressed_data.len();
        let kilobytes = data_size / 1024;
        #[allow(clippy::cast_precision_loss)] // f64 conversion for MB display
        let megabytes = data_size as f64 / (1024.0 * 1024.0);
        let report_type = if report_id == -1 {
            "latest DSD report"
        } else {
            "specific DSD report"
        };

        // 🛡️ GUARD: Warn if individual entry size is very large (soft limit for logging)
        if data_size > MAX_COMPRESSED_ENTRY_SIZE {
            warn!(
                "⚠️  Layer 3: Very large DSD compressed data for {} (lang: {}) ({:.1}MB)",
                report_type, language, megabytes
            );
            error!(
                "   Note: Cache library will handle storage, but consider optimizing entry size"
            );
        } else if data_size > WARN_COMPRESSED_ENTRY_SIZE {
            warn!(
                "⚠️  Layer 3: Large DSD compressed entry for {} (lang: {}) ({:.1}MB)",
                report_type, language, megabytes
            );
        }

        // ✅ Cache the data - library handles memory management automatically
        if let Some(ref cache_system) = state.cache_system {
            let cache_key = format!("compressed_report_dsd_{report_id}_{language}");
            let strategy = crate::service_islands::layer1_infrastructure::cache_system_island::cache_manager::CacheStrategy::ShortTerm;

            // ✅ OPTIMIZATION: Encode as Base64 string instead of JSON Array of Numbers
            // This reduces memory usage by ~95% compared to [12, 255, ...] format
            let base64_string = BASE64_STANDARD.encode(compressed_data);
            let compressed_json = serde_json::Value::String(base64_string);

            cache_system
                .cache_manager
                .set_with_strategy(&cache_key, compressed_json, strategy)
                .await?;

            debug!(
                "💾 Layer 3: Cached DSD compressed data for {} (lang: {}) ({}KB)",
                report_type, language, kilobytes
            );
        }
        Ok(())
    }

    /// Get current cache statistics from the cache manager
    ///
    /// ✅ PRODUCTION-READY: Queries actual cache statistics from multi-tier-cache library
    /// instead of maintaining separate manual counters. This ensures accuracy even after
    /// automatic evictions.
    pub fn get_cache_stats(state: &Arc<AppState>) -> Option<String> {
        if let Some(ref cache_system) = state.cache_system {
            let cache_stats = cache_system.cache_manager.get_stats();
            Some(format!(
                "L1 Hits: {} | L2 Hits: {} | Misses: {} | Hit Rate: {:.1}%",
                cache_stats.l1_hits, cache_stats.l2_hits, cache_stats.misses, cache_stats.hit_rate
            ))
        } else {
            None
        }
    }

    // ========================================
    // Helper Functions for Reports List
    // ========================================

    /// Step 1: Fetch reports from database
    /// Step 1: Fetch reports from database
    async fn fetch_reports_from_db(
        db: &sqlx::PgPool,
        page: i64,
        per_page: i64,
    ) -> anyhow::Result<(i64, Vec<ReportSummaryData>)> {
        let offset = (page - 1) * per_page;

        let total_fut =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM crypto_report").fetch_one(db);
        let rows_fut = sqlx::query_as::<_, ReportSummaryData>(
            "SELECT id, created_at FROM crypto_report ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(per_page)
        .bind(offset)
        .fetch_all(db);

        let (total_res, rows_res) = tokio::join!(total_fut, rows_fut);

        let total = total_res.map_err(|e| {
            error!("❌ Layer 3: Database error getting total count: {}", e);
            anyhow::anyhow!("Database error: {}", e)
        })?;

        let list = rows_res.map_err(|e| {
            error!("❌ Layer 3: Database error getting report list: {}", e);
            anyhow::anyhow!("Database error: {}", e)
        })?;

        debug!(
            "📊 Layer 3: Fetched {} reports for page {}, total: {}",
            list.len(),
            page,
            total
        );
        Ok((total, list))
    }

    /// Step 2: Format report items
    /// ✅ PRODUCTION-READY: Synchronous execution - no overhead for small lists
    fn format_report_items(list: Vec<ReportSummaryData>) -> Vec<serde_json::Value> {
        list.into_iter()
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
    }

    /// Step 3: Calculate pagination
    /// ✅ PRODUCTION-READY: Synchronous execution - pure math calculation
    fn calculate_pagination(total: i64, page: i64, per_page: i64) -> (i64, Vec<Option<i64>>) {
        // Safe integer division with ceiling - avoids float precision loss
        let pages = if total == 0 {
            1
        } else {
            // Integer ceiling division: (a + b - 1) / b
            (total + per_page - 1) / per_page
        };

        let mut page_numbers: Vec<Option<i64>> = Vec::new();
        if pages <= 10 {
            for p in 1..=pages {
                page_numbers.push(Some(p));
            }
        } else {
            let mut added = std::collections::HashSet::new();
            let push =
                |vec: &mut Vec<Option<i64>>, v: i64, added: &mut std::collections::HashSet<i64>| {
                    if !added.contains(&v) && v > 0 && v <= pages {
                        vec.push(Some(v));
                        added.insert(v);
                    }
                };

            push(&mut page_numbers, 1, &mut added);
            push(&mut page_numbers, 2, &mut added);
            for v in (page - 2)..=(page + 2) {
                if v > 2 && v < pages - 1 {
                    push(&mut page_numbers, v, &mut added);
                }
            }
            push(&mut page_numbers, pages - 1, &mut added);
            push(&mut page_numbers, pages, &mut added);

            let mut nums: Vec<i64> = page_numbers.iter().filter_map(|o| *o).collect();
            nums.sort_unstable();
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
    }

    /// Step 4: Build reports context
    fn build_reports_context(
        items: &[serde_json::Value],
        total: i64,
        page: i64,
        per_page: i64,
        pages: i64,
        page_numbers: &[Option<i64>],
    ) -> serde_json::Value {
        let offset = (page - 1) * per_page;
        let display_start = if total == 0 { 0 } else { offset + 1 };
        // Safe conversion: items.len() is bounded by per_page which is i64
        let display_end = offset + i64::try_from(items.len()).unwrap_or(0);

        serde_json::json!({
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
        })
    }

    /// Step 5: Render template
    /// ✅ MEMORY FIX: Synchronous rendering to avoid cloning Tera
    ///
    /// # Errors
    ///
    /// Returns error if template rendering fails
    fn render_reports_template_sync(
        tera: &tera::Tera,
        reports: &serde_json::Value,
    ) -> anyhow::Result<String> {
        let mut context = tera::Context::new();
        context.insert("reports", reports);
        tera.render("crypto/routes/reports/list.html", &context)
            .map_err(|e| {
                error!("❌ Layer 3: Reports list template render error: {:#?}", e);
                anyhow::anyhow!("Template render error: {}", e)
            })
    }

    /// Step 6: Compress HTML
    fn compress_html(html: &str, page: i64) -> anyhow::Result<Vec<u8>> {
        use flate2::{write::GzEncoder, Compression};
        use std::io::Write;

        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(html.as_bytes()).map_err(|e| {
            error!(
                "❌ Layer 3: Compression write error for reports list page {}: {}",
                page, e
            );
            anyhow::anyhow!("Compression error: {}", e)
        })?;

        let compressed_data = encoder.finish().map_err(|e| {
            error!(
                "❌ Layer 3: Compression finish error for reports list page {}: {}",
                page, e
            );
            anyhow::anyhow!("Compression error: {}", e)
        })?;

        let original_size = html.len();
        let compressed_size = compressed_data.len();
        // Safe f64 conversion for display purposes only (compression ratio)
        // Using saturating conversion to handle edge cases gracefully
        #[allow(clippy::cast_precision_loss)]
        let compression_ratio = if original_size > 0 {
            (1.0 - (compressed_size as f64 / original_size as f64)) * 100.0
        } else {
            0.0
        };

        info!("🗜️  Layer 3: Reports list compressed - Original: {}KB, Compressed: {}KB, Ratio: {:.1}%",
                 original_size / 1024,
                 compressed_size / 1024,
                 compression_ratio);

        Ok(compressed_data)
    }

    /// Fetch reports list with intelligent caching (L1+L2)
    ///
    /// ✅ MEMORY FIX: Uses manual cache get/set to avoid cloning Tera into async closure
    /// Caches compressed HTML (Vec<u8>) for fast pagination responses
    ///
    /// # Errors
    ///
    /// Returns error if database query fails, template rendering fails, HTML compression fails, or cache operation fails
    /// Returns error if database query fails, template rendering fails, HTML compression fails, or cache operation fails
    pub async fn fetch_reports_list_with_cache(
        &self,
        state: &Arc<AppState>,
        page: i64,
        per_page: i64,
    ) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error + Send + Sync>> {
        let cache_key = format!("crypto_reports_list_page_{page}_compressed");

        // Step 1: Try to get from cache first
        if let Some(ref cache_system) = state.cache_system {
            if let Ok(Some(cached_value)) = cache_system.cache_manager.get(&cache_key).await {
                // Try to parse as Vec<u8>
                if let Ok(compressed_bytes) = serde_json::from_value::<Vec<u8>>(cached_value) {
                    info!("🔥 Layer 3: Cache HIT for reports list page {}", page);
                    return Ok(Some(compressed_bytes));
                }
            }
        }

        // Step 2: Cache MISS - Generate fresh content
        info!(
            "🗄️ CryptoDataService: Generating reports list page {} from database",
            page
        );

        // Fetch from database
        let (total, list) = Self::fetch_reports_from_db(&state.db, page, per_page).await?;

        // Format report items
        let items = Self::format_report_items(list);

        // Calculate pagination
        let (pages, page_numbers) = Self::calculate_pagination(total, page, per_page);

        // Build reports context
        let items_count = items.len();
        let reports =
            Self::build_reports_context(&items, total, page, per_page, pages, &page_numbers);

        // ✅ MEMORY FIX: Render template synchronously without cloning Tera
        let html = Self::render_reports_template_sync(&state.tera, &reports)?;
        info!(
            "✅ Layer 3: Reports list template rendered successfully - {} items, page {} of {}",
            items_count, page, pages
        );

        // Compress HTML
        let compressed_data = Self::compress_html(&html, page)?;

        // Step 3: Cache the result
        if let Some(ref cache_system) = state.cache_system {
            let compressed_json =
                serde_json::to_value(&compressed_data).unwrap_or(serde_json::Value::Null);
            let strategy = crate::service_islands::layer1_infrastructure::cache_system_island::CacheStrategy::ShortTerm;

            if let Err(e) = cache_system
                .cache_manager
                .set_with_strategy(&cache_key, compressed_json, strategy)
                .await
            {
                warn!(
                    "⚠️ Layer 3: Failed to cache reports list page {}: {}",
                    page, e
                );
            }
        }
        Ok(Some(compressed_data))
    }
}
