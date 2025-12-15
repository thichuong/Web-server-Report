//! Crypto Reports HTTP Request Handlers
//!
//! This module contains all HTTP request handlers related to crypto reports functionality.
//! Based on `archive_old_code/handlers/crypto.rs`
//! ONLY uses Template Engine - NO manual HTML creation

use axum::{
    body::Body,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use flate2::{write::GzEncoder, Compression};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::error::Error as StdError;
use std::hash::{Hash, Hasher};
use std::io::Write;
use std::sync::{atomic::Ordering, Arc};
use tracing::{debug, error, info, warn};

use crate::service_islands::layer5_business_logic::crypto_reports::rendering::{
    generate_breadcrumbs_and_related, generate_complete_geo_metadata,
};

// Import from current state - will be refactored when lower layers are implemented
use crate::service_islands::layer1_infrastructure::AppState;

// Import from our specialized components
use super::report_creator::ReportCreator;
use super::template_orchestrator::TemplateOrchestrator;
use crate::service_islands::layer5_business_logic::shared::error::Layer5Result;

/// Rendered content ready for HTTP response
/// Decouples business logic from HTTP transport
pub struct RenderedContent {
    pub data: Vec<u8>,
    pub cache_control: &'static str,
    pub cache_status: &'static str,
}

impl IntoResponse for RenderedContent {
    fn into_response(self) -> Response {
        Response::builder()
            .status(StatusCode::OK)
            .header("cache-control", self.cache_control)
            .header("content-type", "text/html; charset=utf-8")
            .header("content-encoding", "gzip")
            .header("x-render-mode", "declarative-shadow-dom")
            .header("x-cache", self.cache_status)
            .body(Body::from(self.data))
            .unwrap_or_else(|_| Response::new(Body::from("Response build error")))
            .into_response()
    }
}

/// Crypto Handlers
///
/// Contains all HTTP request handlers for crypto reports-related operations.
/// These handlers manage crypto report generation and API interactions.
/// ONLY uses Template Engine like `archive_old_code/handlers/crypto.rs`
pub struct CryptoHandlers {
    pub report_creator: ReportCreator,
    pub template_orchestrator: TemplateOrchestrator,
}

impl Default for CryptoHandlers {
    fn default() -> Self {
        Self::new()
    }
}

impl CryptoHandlers {
    /// Create a new `CryptoHandlers` instance
    #[must_use]
    pub fn new() -> Self {
        let report_creator = ReportCreator::new();
        let template_orchestrator = TemplateOrchestrator::new(report_creator.clone());

        Self {
            report_creator,
            template_orchestrator,
        }
    }

    /// Health check for crypto handlers
    #[must_use]
    pub fn health_check(&self) -> bool {
        // Verify handlers are functioning properly
        let report_creator_ok = self.report_creator.health_check();
        let template_orchestrator_ok = self.template_orchestrator.health_check();

        report_creator_ok && template_orchestrator_ok
    }

    /// Create cached response with proper headers
    ///
    /// From `archive_old_code/handlers/crypto.rs::create_cached_response`
    #[allow(dead_code)]
    #[must_use]
    pub fn create_cached_response(&self, html: String, cache_status: &str) -> Response {
        Response::builder()
            .status(StatusCode::OK)
            .header("cache-control", "public, max-age=15")
            .header("content-type", "text/html; charset=utf-8")
            .header("x-cache", cache_status)
            .body(html)
            .unwrap_or_else(|e| {
                warn!("⚠️ Failed to build cached response: {}", e);
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body("Response build error".to_string())
                    .unwrap_or_else(|_| Response::new("Response build error".to_string()))
            })
            .into_response()
    }

    /// Create compressed HTTP response with proper headers
    ///
    /// Helper function to create HTTP response with gzip compression headers
    #[must_use]
    pub fn create_compressed_response(compressed_data: Vec<u8>) -> Response {
        Response::builder()
            .status(StatusCode::OK)
            .header("cache-control", "public, max-age=15")
            .header("x-cache", "compressed")
            .header("content-type", "text/html; charset=utf-8")
            .header("content-encoding", "gzip")
            .body(Body::from(compressed_data))
            .unwrap_or_else(|e| {
                warn!("⚠️ Failed to build compressed response: {}", e);
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::from("Response build error"))
                    .unwrap_or_else(|_| Response::new(Body::from("Response build error")))
            })
            .into_response()
    }

    /// Compress HTML string to gzip format
    ///
    /// Helper function to compress HTML strings for templates that don't use compression
    /// ✅ PUBLIC: Can be called from route handlers for DSD routes
    ///
    /// # Errors
    ///
    /// Returns error if compression fails
    pub fn compress_html_to_gzip(html: &str) -> Result<Vec<u8>, Box<dyn StdError + Send + Sync>> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(html.as_bytes())?;
        let compressed_data = encoder.finish()?;

        let original_size = html.len();
        let compressed_size = compressed_data.len();
        #[allow(clippy::cast_precision_loss)]
        let compression_ratio = (1.0 - (compressed_size as f64 / original_size as f64)) * 100.0;

        info!(
            "🗜️  CryptoHandlers: HTML compressed - Original: {}KB, Compressed: {}KB, Ratio: {:.1}%",
            original_size / 1024,
            compressed_size / 1024,
            compression_ratio
        );

        Ok(compressed_data)
    }

    /// Crypto Index with Tera template engine - FULL IMPLEMENTATION
    ///
    /// Exactly like `archive_old_code/handlers/crypto.rs::crypto_index` - Complete L1/L2 caching
    /// Enhanced with pre-loaded chart modules and HTML caching for optimal performance
    /// Now returns compressed data for optimal transfer speed
    /// ✅ OPTIMIZED: Uses Arc<String> to avoid cloning chart modules content
    ///
    /// # Errors
    ///
    /// Returns error if database fetch or template rendering fails
    /// Crypto Index with Tera template engine - FULL IMPLEMENTATION
    ///
    /// Exactly like `archive_old_code/handlers/crypto.rs::crypto_index` - Complete L1/L2 caching
    /// Enhanced with pre-loaded chart modules and HTML caching for optimal performance
    /// Now returns compressed data for optimal transfer speed
    /// ✅ OPTIMIZED: Uses Arc<String> to avoid cloning chart modules content
    ///
    /// # Errors
    ///
    /// Returns error if database fetch or template rendering fails
    /// Crypto Index with Tera template engine - FULL IMPLEMENTATION
    ///
    /// Exactly like `archive_old_code/handlers/crypto.rs::crypto_index` - Complete L1/L2 caching
    /// Enhanced with pre-loaded chart modules and HTML caching for optimal performance
    /// Now returns compressed data for optimal transfer speed
    /// ✅ OPTIMIZED: Uses Arc<String> to avoid cloning chart modules content
    ///
    /// # Errors
    ///
    /// Returns error if database fetch or template rendering fails
    pub async fn crypto_index_with_tera(
        &self,
        state: &Arc<AppState>,
        chart_modules_content: Option<Arc<String>>, // ✅ Already Arc, no clone needed
    ) -> Result<Vec<u8>, Box<dyn StdError + Send + Sync>> {
        debug!("🚀 Layer 5: Nhận yêu cầu cho crypto_index (latest report)");

        // Increment request counter to monitor performance
        let request_count = state.request_counter.fetch_add(1, Ordering::Relaxed);

        // Log every 100 requests for monitoring
        if request_count.is_multiple_of(100) {
            info!("Processed {} requests to crypto_index", request_count);
        }

        // BƯỚC 1: HỎI LAYER 3 ĐỂ LẤY COMPRESSED DATA TỪ CACHE CHO LATEST REPORT
        // (Không gọi trực tiếp Layer 1)
        let data_service = &self.report_creator.data_service; // Truy cập data_service

        // Check cache for compressed data first (preferred)
        // Check cache for compressed data first (preferred)
        if let Ok(Some(cached_compressed)) =
            data_service.get_rendered_report_compressed(state, -1).await
        {
            info!(
                "✅ Layer 5: Nhận compressed data từ cache cho latest report. Trả về ngay lập tức."
            );
            return Ok(cached_compressed);
        }

        debug!("🔍 Layer 5: Cache miss cho latest report. Bắt đầu quy trình render.");

        // BƯỚC 2: NẾU CACHE MISS, TIẾP TỤC LOGIC HIỆN TẠI
        // Fetch from DB (không cần chart modules vì đã có pre-loaded)
        let db_res = self
            .report_creator
            .fetch_and_cache_latest_report(state)
            .await;

        match db_res {
            Ok(Some(report)) => {
                // ✅ MEMORY OPTIMIZED: Move report ownership to avoid cloning in template_orchestrator

                // Ensure chart modules are available (fetch async if needed)
                // This keeps the rendering logic purely synchronous
                let modules_content = if let Some(content) = chart_modules_content {
                    Some(content)
                } else {
                    Some(Arc::new(self.report_creator.get_chart_modules_content()))
                };

                // Template rendering with TemplateOrchestrator (Synchronous)
                // Template rendering with TemplateOrchestrator (Synchronous)
                match self.template_orchestrator.render_crypto_report_view(
                    &state.tera,
                    report,          // ✅ Move ownership - no clone needed!
                    modules_content, // ✅ Arc<String> passed directly, zero clone
                    None,
                ) {
                    Ok(compressed_data) => {
                        info!("✅ Layer 5: Render thành công cho latest report. Yêu cầu Layer 3 cache lại compressed data.");
                        // BƯỚC 3: SAU KHI RENDER THÀNH CÔNG, YÊU CẦU LAYER 3 LƯU LẠI COMPRESSED DATA
                        // ✅ MEMORY OPTIMIZED: Pass reference instead of cloning entire Vec<u8>
                        if let Err(e) = data_service
                            .cache_rendered_report_compressed(state, -1, &compressed_data)
                            .await
                        {
                            warn!(
                                "⚠️ Layer 5: Không thể cache compressed data cho latest report: {}",
                                e
                            );
                        }
                        info!("✅ Template rendered from DB via TemplateOrchestrator - crypto_index complete");
                        Ok(compressed_data)
                    }
                    Err(e) => {
                        error!("❌ TemplateOrchestrator render error: {}", e);
                        Err("Template render error".into())
                    }
                }
            }
            Ok(None) => {
                warn!("⚠️ No reports found in database - rendering empty template via TemplateOrchestrator");

                // Use TemplateOrchestrator for empty template (Synchronous)
                match self
                    .template_orchestrator
                    .render_empty_template(&state.tera)
                {
                    Ok(html) => {
                        info!("✅ Empty template rendered successfully via TemplateOrchestrator");
                        // Compress the empty template HTML
                        match Self::compress_html_to_gzip(&html) {
                            Ok(compressed_data) => Ok(compressed_data),
                            Err(e) => {
                                error!("❌ Failed to compress empty template: {}", e);
                                Err(format!("Empty template compression error: {e}").into())
                            }
                        }
                    }
                    Err(e) => {
                        error!("❌ TemplateOrchestrator empty template render error: {}", e);
                        Err(format!("Empty template render error: {e}").into())
                    }
                }
            }
            Err(e) => {
                error!("❌ Database error in crypto_index: {}", e);
                Err(format!("Database error: {e}").into())
            }
        }
    }

    /// Crypto Report by ID handler - Specific crypto report view
    ///
    /// Similar to `crypto_index_with_tera` but for specific report ID
    /// Exactly like `archive_old_code/handlers/crypto.rs` pattern - Complete L1/L2 caching
    /// Enhanced with rendered HTML caching for optimal performance
    /// Now returns compressed data for optimal transfer speed
    /// ✅ OPTIMIZED: Uses Arc<String> to avoid cloning chart modules content
    ///
    /// # Errors
    ///
    /// Returns error if database fetch or template rendering fails
    /// Returns error if database fetch or template rendering fails
    /// Crypto Report by ID handler - Specific crypto report view
    ///
    /// Similar to `crypto_index_with_tera` but for specific report ID
    /// Exactly like `archive_old_code/handlers/crypto.rs` pattern - Complete L1/L2 caching
    /// Enhanced with rendered HTML caching for optimal performance
    /// Now returns compressed data for optimal transfer speed
    /// ✅ OPTIMIZED: Uses Arc<String> to avoid cloning chart modules content
    ///
    /// # Errors
    ///
    /// Returns error if database fetch or template rendering fails
    /// Returns error if database fetch or template rendering fails
    pub async fn crypto_report_by_id_with_tera(
        &self,
        state: &Arc<AppState>,
        report_id: i32,
        chart_modules_content: Option<Arc<String>>, // ✅ Already Arc, no clone needed
    ) -> Result<Vec<u8>, Box<dyn StdError + Send + Sync>> {
        debug!("🚀 Layer 5: Nhận yêu cầu cho report #{}", report_id);

        // Increment request counter to monitor performance
        let request_count = state.request_counter.fetch_add(1, Ordering::Relaxed);

        // Log every 100 requests for monitoring
        if request_count.is_multiple_of(100) {
            info!(
                "Processed {} requests to crypto_report_by_id",
                request_count
            );
        }

        // BƯỚC 1: HỎI LAYER 3 ĐỂ LẤY COMPRESSED DATA TỪ CACHE
        // (Không gọi trực tiếp Layer 1)
        let data_service = &self.report_creator.data_service; // Truy cập data_service

        // Check cache for compressed data first (preferred)
        if let Ok(Some(cached_compressed)) = data_service
            .get_rendered_report_compressed(state, report_id)
            .await
        {
            info!("✅ Layer 5: Nhận compressed data từ cache. Trả về ngay lập tức.");
            return Ok(cached_compressed);
        }

        debug!(
            "🔍 Layer 5: Cache miss cho report #{}. Bắt đầu quy trình render.",
            report_id
        );

        // BƯỚC 2: NẾU CACHE MISS, TIẾP TỤC LOGIC HIỆN TẠI
        // Fetch from DB (không cần chart modules vì đã có pre-loaded)
        let db_res = self
            .report_creator
            .fetch_and_cache_report_by_id(state, report_id)
            .await;

        match db_res {
            Ok(Some(report)) => {
                // ✅ MEMORY OPTIMIZED: Move report ownership to avoid cloning in template_orchestrator

                // Ensure chart modules are available (fetch async if needed)
                let modules_content = if let Some(content) = chart_modules_content {
                    Some(content)
                } else {
                    Some(Arc::new(self.report_creator.get_chart_modules_content()))
                };

                // Template rendering with TemplateOrchestrator (Synchronous)
                match self.template_orchestrator.render_crypto_report_view(
                    &state.tera,
                    report,          // ✅ Move ownership - no clone needed!
                    modules_content, // ✅ Arc<String> passed directly, zero clone
                    None,
                ) {
                    Ok(compressed_data) => {
                        info!("✅ Layer 5: Render thành công cho report #{}. Yêu cầu Layer 3 cache lại compressed data.", report_id);
                        // BƯỚC 3: SAU KHI RENDER THÀNH CÔNG, YÊU CẦU LAYER 3 LƯU LẠI COMPRESSED DATA
                        // ✅ MEMORY OPTIMIZED: Pass reference instead of cloning entire Vec<u8>
                        if let Err(e) = data_service
                            .cache_rendered_report_compressed(state, report_id, &compressed_data)
                            .await
                        {
                            warn!(
                                "⚠️ Layer 5: Không thể cache compressed data cho report #{}: {}",
                                report_id, e
                            );
                        }
                        info!("✅ Template rendered from DB via TemplateOrchestrator - crypto_report_by_id complete");
                        Ok(compressed_data)
                    }
                    Err(e) => {
                        error!("❌ TemplateOrchestrator render error: {}", e);
                        Err("Template render error".into())
                    }
                }
            }
            Ok(None) => {
                warn!("⚠️ No reports found in database - rendering empty template via TemplateOrchestrator");

                // Use TemplateOrchestrator for empty template (Synchronous)
                match self
                    .template_orchestrator
                    .render_empty_template(&state.tera)
                {
                    Ok(html) => {
                        info!("✅ Empty template rendered successfully via TemplateOrchestrator");
                        // Compress the empty template HTML
                        match Self::compress_html_to_gzip(&html) {
                            Ok(compressed_data) => Ok(compressed_data),
                            Err(e) => {
                                error!("❌ Failed to compress empty template: {}", e);
                                Err(format!("Empty template compression error: {e}").into())
                            }
                        }
                    }
                    Err(e) => {
                        error!("❌ TemplateOrchestrator empty template render error: {}", e);
                        Err(format!("Empty template render error: {e}").into())
                    }
                }
            }
            Err(e) => {
                error!("❌ Database error in crypto_report_by_id: {}", e);
                Err(format!("Database error: {e}").into())
            }
        }
    }

    /// Crypto Reports List handler - Paginated list of all reports
    ///
    /// Delegated to Layer 3 with cache integration - similar to `crypto_index_with_tera` pattern
    /// Returns compressed data (Vec<u8>) for optimal transfer speed
    ///
    /// # Errors
    ///
    /// Returns error if database fetch fails
    /// Returns error if database fetch fails
    /// Crypto Reports List handler - Paginated list of all reports
    ///
    /// Delegated to Layer 3 with cache integration - similar to `crypto_index_with_tera` pattern
    /// Returns compressed data (Vec<u8>) for optimal transfer speed
    ///
    /// # Errors
    ///
    /// Returns error if database fetch fails
    /// Returns error if database fetch fails
    pub async fn crypto_reports_list_with_tera(
        &self,
        state: &Arc<AppState>,
        page: i64,
    ) -> Result<Vec<u8>, Box<dyn StdError + Send + Sync>> {
        info!(
            "📋 Layer 5: Nhận yêu cầu cho crypto reports list page {}",
            page
        );

        // Increment request counter to monitor performance
        let request_count = state.request_counter.fetch_add(1, Ordering::Relaxed);

        // Log every 50 requests for monitoring
        if request_count.is_multiple_of(50) {
            info!(
                "Processed {} requests to crypto_reports_list",
                request_count
            );
        }

        // BƯỚC 1: ỦY QUYỀN CHO LAYER 3 ĐỂ XỬ LÝ CACHE VÀ DATABASE (returns compressed data)
        let data_service = &self.report_creator.data_service; // Truy cập data_service
        let per_page: i64 = 10;

        match data_service
            .fetch_reports_list_with_cache(state, page, per_page)
            .await
        {
            Ok(Some(compressed_data)) => {
                let size_kb = compressed_data.len() / 1024;
                info!(
                    "✅ Layer 5: Nhận compressed data từ Layer 3 cho reports list page {} ({}KB)",
                    page, size_kb
                );
                Ok(compressed_data)
            }
            Ok(None) => {
                warn!(
                    "⚠️ Layer 5: Layer 3 trả về None cho reports list page {}",
                    page
                );
                Err("No reports list data available".into())
            }
            Err(e) => {
                error!(
                    "❌ Layer 5: Layer 3 error cho reports list page {}: {}",
                    page, e
                );
                Err(e)
            }
        }
    }

    /// Serve sandboxed report content for iframe
    ///
    /// Delegates to `ReportCreator` for actual sandboxed content generation.
    /// Returns `Layer5Result` for proper error handling.
    ///
    /// # Errors
    ///
    /// Returns error if report generation fails
    /// Returns error if report generation fails
    /// Serve sandboxed report content for iframe
    ///
    /// Delegates to `ReportCreator` for actual sandboxed content generation.
    /// Returns `Layer5Result` for proper error handling.
    ///
    /// # Errors
    ///
    /// Returns error if report generation fails
    /// Returns error if report generation fails
    pub async fn serve_sandboxed_report(
        &self,
        state: &Arc<AppState>,
        report_id: i32,
        sandbox_token: &str,
        language: Option<&str>,
        chart_modules_content: Option<&str>,
    ) -> Result<axum::response::Response, Box<dyn StdError + Send + Sync>> {
        info!("CryptoHandlers: Delegating sandboxed content request to ReportCreator for report {} with token {}", report_id, sandbox_token);

        // Delegate to ReportCreator and map error for backward compatibility
        self.report_creator
            .serve_sandboxed_report(
                state,
                report_id,
                sandbox_token,
                language,
                chart_modules_content,
            )
            .await
            .map_err(super::super::shared::error::Layer5Error::into_boxed)
    }

    /// Serve Shadow DOM content for Declarative Shadow DOM architecture
    ///
    /// Delegates to `ReportCreator` for actual Shadow DOM content generation.
    /// This is the modern replacement for `serve_sandboxed_report`.
    ///
    /// # Errors
    ///
    /// Returns error if report generation fails
    /// Returns error if report generation fails
    pub async fn serve_shadow_dom_content(
        &self,
        state: &Arc<AppState>,
        report_id: i32,
        shadow_dom_token: &str,
        language: Option<&str>,
        chart_modules_content: Option<&str>,
    ) -> Result<axum::response::Response, Box<dyn StdError + Send + Sync>> {
        info!("CryptoHandlers: Delegating Shadow DOM content request to ReportCreator for report {} with token {}", report_id, shadow_dom_token);

        // Delegate to ReportCreator and map error for backward compatibility
        self.report_creator
            .serve_shadow_dom_content(
                state,
                report_id,
                shadow_dom_token,
                language,
                chart_modules_content,
            )
            .await
            .map_err(super::super::shared::error::Layer5Error::into_boxed)
    }

    /// Detect preferred language from request
    /// Priority: Query param > Cookie > Accept-Language header > Default (vi)
    pub fn detect_preferred_language(
        query_params: &HashMap<String, String>,
        headers: &HeaderMap,
    ) -> Option<String> {
        // 1. Check query parameter (?lang=en or ?lang=vi)
        if let Some(lang) = query_params.get("lang") {
            let lang = lang.to_lowercase();
            if lang == "en" || lang == "vi" {
                debug!("🌐 [Language] Detected from query param: {}", lang);
                return Some(lang);
            }
        }

        // 2. Check Cookie header for preferred_language or language
        if let Some(cookie_header) = headers.get("cookie") {
            if let Ok(cookie_str) = cookie_header.to_str() {
                // Parse cookies manually
                for cookie in cookie_str.split(';') {
                    let parts: Vec<&str> = cookie.trim().splitn(2, '=').collect();
                    if let [name_part, value_part] = parts.as_slice() {
                        let (name, value) = (name_part.trim(), value_part.trim());
                        if name == "preferred_language" || name == "language" {
                            let lang = value.to_lowercase();
                            if lang == "en" || lang == "vi" {
                                debug!("🌐 [Language] Detected from cookie: {}", lang);
                                return Some(lang);
                            }
                        }
                    }
                }
            }
        }

        // 3. Check Accept-Language header
        if let Some(accept_lang) = headers.get("accept-language") {
            if let Ok(lang_str) = accept_lang.to_str() {
                // Parse Accept-Language: "vi-VN,vi;q=0.9,en-US;q=0.8,en;q=0.7"
                for lang_tag in lang_str.split(',') {
                    let lang = lang_tag.split(';').next().unwrap_or("").trim();
                    if lang.starts_with("en") {
                        debug!("🌐 [Language] Detected from Accept-Language: en");
                        return Some("en".to_string());
                    } else if lang.starts_with("vi") {
                        debug!("🌐 [Language] Detected from Accept-Language: vi");
                        return Some("vi".to_string());
                    }
                }
            }
        }

        // 4. Default to Vietnamese
        debug!("🌐 [Language] Using default: vi");
        None // None means use default (vi) in generate_shadow_dom_content
    }

    /// Render Crypto Index DSD (Latest Report)
    /// Encapsulates all logic for the `crypto_index` route
    ///
    /// # Errors
    ///
    /// Returns error if database fetch or template rendering fails
    #[allow(clippy::too_many_lines)] // Orchestration function requiring multiple steps (cache, DB, DSD, metadata, rendering)
    #[allow(clippy::needless_pass_by_value)] // Arc is passed by value to maintain API compatibility
    pub async fn render_crypto_index_dsd(
        &self,
        state: &Arc<AppState>,
        params: &HashMap<String, String>,
        headers: &HeaderMap,
        chart_modules_content: Arc<String>,
        report_id_opt: Option<i32>, // Optional specific ID, defaults to latest if None
    ) -> Layer5Result<RenderedContent> {
        debug!("🌓 [Handler] render_crypto_index_dsd called - using Declarative Shadow DOM architecture");

        // Determine report ID value (use provided or -1 for latest)
        let report_id_value = report_id_opt.unwrap_or(-1);

        debug!(
            "🚀 [Handler] render_crypto_index_dsd called for {}",
            if report_id_value == -1 {
                "latest report".to_string()
            } else {
                format!("report ID: {report_id_value}")
            }
        );

        // STEP 1: Quick cache check with default language (Vietnamese - 90% of traffic)
        // This avoids expensive language detection for cache hits
        let data_service = &self.report_creator.data_service;
        let default_language = "vi";

        if let Ok(Some(cached_compressed)) = data_service
            .get_rendered_report_dsd_compressed(state, report_id_value, default_language)
            .await
        {
            info!(
                "✅ [Handler] DSD cache HIT (default language) - returning compressed HTML for {}",
                if report_id_value == -1 {
                    "latest".to_string()
                } else {
                    format!("#{report_id_value}")
                }
            );

            return Ok(RenderedContent {
                data: cached_compressed,
                cache_control: "public, max-age=300",
                cache_status: "HIT",
            });
        }

        // STEP 1.1: Cache miss with default language - now detect preferred language for rendering
        let preferred_language =
            Self::detect_preferred_language(params, headers).unwrap_or_else(|| "vi".to_string());

        debug!(
            "🔍 [Handler] DSD cache MISS (default language) - detected preferred language: {}",
            preferred_language
        );

        // STEP 1.2: If preferred language differs from default, try cache with preferred language
        if preferred_language != default_language {
            if let Ok(Some(cached_compressed)) = data_service
                .get_rendered_report_dsd_compressed(state, report_id_value, &preferred_language)
                .await
            {
                info!(
                    "✅ [Handler] DSD cache HIT (preferred language: {}) - returning compressed HTML for {}",
                    preferred_language,
                    if report_id_value == -1 {
                        "latest".to_string()
                    } else {
                        format!("#{report_id_value}")
                    }
                );

                return Ok(RenderedContent {
                    data: cached_compressed,
                    cache_control: "public, max-age=300",
                    cache_status: "HIT",
                });
            }
        }

        debug!("🔍 [Handler] DSD cache MISS - generating fresh HTML");

        // STEP 2: Fetch report from database (uses existing data cache)
        let report_result = if report_id_value == -1 {
            self.report_creator
                .fetch_and_cache_latest_report(state)
                .await
        } else {
            self.report_creator
                .fetch_and_cache_report_by_id(state, report_id_value)
                .await
        };

        let report = match report_result {
            Ok(Some(report)) => report,
            Ok(None) => {
                warn!("⚠️ [Handler] No report found for DSD view");
                return Err(crate::service_islands::layer5_business_logic::shared::error::Layer5Error::NotFound("Report not found".to_string()));
            }
            Err(e) => {
                error!("❌ [Handler] Database error fetching report for DSD: {}", e);
                return Err(e.into());
            }
        };

        // STEP 3: Generate shadow_dom_token
        let mut hasher = DefaultHasher::new();
        report.id.hash(&mut hasher);
        report.created_at.hash(&mut hasher);
        let shadow_dom_token = format!("sb_{:x}", hasher.finish());

        // STEP 4: Get chart modules content and generate shadow DOM content
        let sandboxed_report = self
            .report_creator
            .create_sandboxed_report(&report, Some(chart_modules_content.as_str()));
        let shadow_dom_content = self.report_creator.generate_shadow_dom_content(
            &sandboxed_report,
            Some(&preferred_language),
            Some(chart_modules_content.as_str()),
        );

        info!(
            "🌐 [Handler] render_crypto_index_dsd rendering with language: {}",
            preferred_language
        );

        // STEP 5: Generate GEO metadata for AI bots (Grok, GPT, Claude)
        let (geo_meta_tags, geo_json_ld, geo_title) =
            generate_complete_geo_metadata(&report, Some(&preferred_language));
        debug!(
            "📊 [Handler] GEO metadata generated for report {} - title: {}",
            report.id, geo_title
        );

        // STEP 5.1: Fetch related reports for internal linking (GEO optimization)
        let related_reports_data = match data_service
            .fetch_related_reports(state, report.id, 3)
            .await
        {
            Ok(reports) => reports,
            Err(e) => {
                warn!("⚠️ [Handler] Failed to fetch related reports: {}", e);
                vec![] // Fallback to empty list on error
            }
        };

        // STEP 5.2: Generate breadcrumbs and related reports data
        let (breadcrumb_items, breadcrumbs_schema, related_reports) =
            generate_breadcrumbs_and_related(report.id, &related_reports_data);
        debug!(
            "📊 [Handler] Breadcrumbs and {} related reports generated for report {}",
            related_reports.len(),
            report.id
        );

        // STEP 6: Render template with GEO metadata
        let mut context = tera::Context::new();
        context.insert("report", &report);
        context.insert("shadow_dom_token", &shadow_dom_token);
        context.insert("shadow_dom_content", &shadow_dom_content);
        context.insert("chart_modules_content", chart_modules_content.as_ref());
        context.insert(
            "websocket_url",
            &std::env::var("WEBSOCKET_SERVICE_URL")
                .unwrap_or_else(|_| "ws://localhost:8081/ws".to_string()),
        );
        // GEO metadata for AI optimization
        context.insert("geo_meta_tags", &geo_meta_tags);
        context.insert("geo_json_ld", &geo_json_ld);
        context.insert("geo_title", &geo_title);
        // Breadcrumbs and related reports for internal linking
        context.insert("breadcrumb_items", &breadcrumb_items);
        context.insert("breadcrumbs_schema", &breadcrumbs_schema);
        context.insert("related_reports", &related_reports);

        let html = match state
            .tera
            .render("crypto/routes/reports/view_dsd.html", &context)
        {
            Ok(html) => html,
            Err(e) => {
                error!("❌ [Handler] Failed to render DSD template: {}", e);
                return Err(crate::service_islands::layer5_business_logic::shared::error::Layer5Error::TemplateRender(e.to_string()));
            }
        };

        // STEP 7: Compress HTML
        let compressed_data = match Self::compress_html_to_gzip(&html) {
            Ok(data) => data,
            Err(e) => {
                error!("❌ [Handler] Failed to compress DSD HTML: {}", e);
                return Err(crate::service_islands::layer5_business_logic::shared::error::Layer5Error::Compression(e.to_string()));
            }
        };

        // STEP 8: Cache response (compressed DSD content)
        // ✅ MEMORY OPTIMIZED: pass slice reference to avoid large clones
        if let Err(e) = data_service
            .cache_rendered_report_dsd_compressed(
                state,
                report_id_value,
                &compressed_data,
                &preferred_language,
            )
            .await
        {
            warn!("⚠️ [Handler] Failed to cache DSD compressed content: {}", e);
        }

        info!("✅ [Handler] render_crypto_index_dsd completed successfully");

        // STEP 9: Return compressed response
        Ok(RenderedContent {
            data: compressed_data,
            cache_control: "public, max-age=300",
            cache_status: "MISS",
        })
    }

    /// Render Crypto Report by ID DSD
    /// Encapsulates all logic for the `crypto_view_report` route
    /// Render Crypto Report by ID DSD
    /// Encapsulates all logic for the `crypto_view_report` route
    ///
    /// # Errors
    ///
    /// Returns error if database fetch or template rendering fails
    /// Render Crypto Report by ID DSD
    /// Encapsulates all logic for the `crypto_view_report` route
    ///
    /// # Errors
    ///
    /// Returns error if database fetch or template rendering fails
    pub async fn render_crypto_report_dsd(
        &self,
        state: &Arc<AppState>,
        report_id: i32,
        params: &HashMap<String, String>,
        headers: &HeaderMap,
        chart_modules_content: Arc<String>,
    ) -> Layer5Result<RenderedContent> {
        // Reuse the logic from render_crypto_index_dsd but with specific ID
        self.render_crypto_index_dsd(
            state,
            params,
            headers,
            chart_modules_content,
            Some(report_id),
        )
        .await
    }
}
