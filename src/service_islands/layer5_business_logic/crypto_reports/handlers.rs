//! Crypto Reports HTTP Request Handlers
//!
//! This module contains all HTTP request handlers related to crypto reports functionality.
//! Based on archive_old_code/handlers/crypto.rs
//! ONLY uses Template Engine - NO manual HTML creation

use axum::{
    response::{Response, IntoResponse},
    http::StatusCode,
    body::Body
};
use std::{sync::Arc, error::Error as StdError, sync::atomic::Ordering, io::Write};
use flate2::{Compression, write::GzEncoder};
use tracing::{info, warn, error, debug};

// Import from current state - will be refactored when lower layers are implemented
use crate::service_islands::layer1_infrastructure::AppState;

// Import from our specialized components
use super::report_creator::ReportCreator;
use super::template_orchestrator::TemplateOrchestrator;

/// Crypto Handlers
/// 
/// Contains all HTTP request handlers for crypto reports-related operations.
/// These handlers manage crypto report generation and API interactions.
/// ONLY uses Template Engine like archive_old_code/handlers/crypto.rs
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
    /// Create a new CryptoHandlers instance
    pub fn new() -> Self {
        let report_creator = ReportCreator::new();
        let template_orchestrator = TemplateOrchestrator::new(report_creator.clone());
        
        Self {
            report_creator,
            template_orchestrator,
        }
    }
    
    /// Health check for crypto handlers
    pub async fn health_check(&self) -> bool {
        // Verify handlers are functioning properly
        let report_creator_ok = self.report_creator.health_check().await;
        let template_orchestrator_ok = self.template_orchestrator.health_check().await;
        
        report_creator_ok && template_orchestrator_ok
    }

    /// Create cached response with proper headers
    ///
    /// From archive_old_code/handlers/crypto.rs::create_cached_response
    #[allow(dead_code)]
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
                    .unwrap()  // This is guaranteed safe with literal body
            })
            .into_response()
    }

    /// Create compressed HTTP response with proper headers
    ///
    /// Helper function to create HTTP response with gzip compression headers
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
                    .unwrap()  // This is guaranteed safe with literal body
            })
            .into_response()
    }

    /// Compress HTML string to gzip format
    ///
    /// Helper function to compress HTML strings for templates that don't use compression
    /// ✅ PUBLIC: Can be called from route handlers for DSD routes
    pub fn compress_html_to_gzip(html: &str) -> Result<Vec<u8>, Box<dyn StdError + Send + Sync>> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(html.as_bytes())?;
        let compressed_data = encoder.finish()?;

        let original_size = html.len();
        let compressed_size = compressed_data.len();
        let compression_ratio = (1.0 - (compressed_size as f64 / original_size as f64)) * 100.0;

        info!("🗜️  CryptoHandlers: HTML compressed - Original: {}KB, Compressed: {}KB, Ratio: {:.1}%",
                 original_size / 1024,
                 compressed_size / 1024,
                 compression_ratio);

        Ok(compressed_data)
    }

    /// Crypto Index with Tera template engine - FULL IMPLEMENTATION
    /// 
    /// Exactly like archive_old_code/handlers/crypto.rs::crypto_index - Complete L1/L2 caching
    /// Enhanced with pre-loaded chart modules and HTML caching for optimal performance
    /// Now returns compressed data for optimal transfer speed
    /// ✅ OPTIMIZED: Uses Arc<String> to avoid cloning chart modules content
    pub async fn crypto_index_with_tera(
        &self, 
        state: &Arc<AppState>,
        chart_modules_content: Option<Arc<String>>,  // ✅ Already Arc, no clone needed
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
        if let Ok(Some(cached_compressed)) = data_service.get_rendered_report_compressed(state, -1).await {
            info!("✅ Layer 5: Nhận compressed data từ cache cho latest report. Trả về ngay lập tức.");
            return Ok(cached_compressed);
        }

        debug!("🔍 Layer 5: Cache miss cho latest report. Bắt đầu quy trình render.");

        // BƯỚC 2: NẾU CACHE MISS, TIẾP TỤC LOGIC HIỆN TẠI
        // Fetch from DB (không cần chart modules vì đã có pre-loaded)
        let db_res = self.report_creator.fetch_and_cache_latest_report(state).await;

        match db_res {
            Ok(Some(report)) => {
                // ✅ MEMORY OPTIMIZED: Move report ownership to avoid cloning in template_orchestrator
                // Template rendering with TemplateOrchestrator
                match self.template_orchestrator.render_crypto_report_view(
                    &state.tera,
                    report,  // ✅ Move ownership - no clone needed!
                    chart_modules_content,  // ✅ Arc<String> passed directly, zero clone
                    None
                ).await {
                    Ok(compressed_data) => {
                        info!("✅ Layer 5: Render thành công cho latest report. Yêu cầu Layer 3 cache lại compressed data.");
                        // BƯỚC 3: SAU KHI RENDER THÀNH CÔNG, YÊU CẦU LAYER 3 LƯU LẠI COMPRESSED DATA
                        // ✅ MEMORY OPTIMIZED: Pass reference instead of cloning entire Vec<u8>
                        if let Err(e) = data_service.cache_rendered_report_compressed(state, -1, &compressed_data).await {
                            warn!("⚠️ Layer 5: Không thể cache compressed data cho latest report: {}", e);
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
                
                // Use TemplateOrchestrator for empty template
                match self.template_orchestrator.render_empty_template(&state.tera).await {
                    Ok(html) => {
                        info!("✅ Empty template rendered successfully via TemplateOrchestrator");
                        // Compress the empty template HTML
                        match Self::compress_html_to_gzip(&html) {
                            Ok(compressed_data) => Ok(compressed_data),
                            Err(e) => {
                                error!("❌ Failed to compress empty template: {}", e);
                                Err(format!("Empty template compression error: {}", e).into())
                            }
                        }
                    }
                    Err(e) => {
                        error!("❌ TemplateOrchestrator empty template render error: {}", e);
                        Err(format!("Empty template render error: {}", e).into())
                    }
                }
            }
            Err(e) => {
                error!("❌ Database error in crypto_index: {}", e);
                Err(format!("Database error: {}", e).into())
            }
        }
    }

    /// Crypto Report by ID handler - Specific crypto report view
    /// 
    /// Similar to crypto_index_with_tera but for specific report ID
    /// Exactly like archive_old_code/handlers/crypto.rs pattern - Complete L1/L2 caching
    /// Enhanced with rendered HTML caching for optimal performance
    /// Now returns compressed data for optimal transfer speed
    /// ✅ OPTIMIZED: Uses Arc<String> to avoid cloning chart modules content
    pub async fn crypto_report_by_id_with_tera(
        &self, 
        state: &Arc<AppState>,
        report_id: i32,
        chart_modules_content: Option<Arc<String>>,  // ✅ Already Arc, no clone needed
    ) -> Result<Vec<u8>, Box<dyn StdError + Send + Sync>> {
        debug!("🚀 Layer 5: Nhận yêu cầu cho report #{}", report_id);
        
        // Increment request counter to monitor performance
        let request_count = state.request_counter.fetch_add(1, Ordering::Relaxed);
        
        // Log every 100 requests for monitoring
        if request_count.is_multiple_of(100) {
            info!("Processed {} requests to crypto_report_by_id", request_count);
        }

        // BƯỚC 1: HỎI LAYER 3 ĐỂ LẤY COMPRESSED DATA TỪ CACHE
        // (Không gọi trực tiếp Layer 1)
        let data_service = &self.report_creator.data_service; // Truy cập data_service
        
        // Check cache for compressed data first (preferred)
        if let Ok(Some(cached_compressed)) = data_service.get_rendered_report_compressed(state, report_id).await {
            info!("✅ Layer 5: Nhận compressed data từ cache. Trả về ngay lập tức.");
            return Ok(cached_compressed);
        }

        debug!("🔍 Layer 5: Cache miss cho report #{}. Bắt đầu quy trình render.", report_id);

        // BƯỚC 2: NẾU CACHE MISS, TIẾP TỤC LOGIC HIỆN TẠI
        // Fetch from DB (không cần chart modules vì đã có pre-loaded)
        let db_res = self.report_creator.fetch_and_cache_report_by_id(state, report_id).await;

        match db_res {
            Ok(Some(report)) => {
                // ✅ MEMORY OPTIMIZED: Move report ownership to avoid cloning in template_orchestrator
                // Template rendering with TemplateOrchestrator
                match self.template_orchestrator.render_crypto_report_view(
                    &state.tera,
                    report,  // ✅ Move ownership - no clone needed!
                    chart_modules_content,  // ✅ Arc<String> passed directly, zero clone
                    None
                ).await {
                    Ok(compressed_data) => {
                        info!("✅ Layer 5: Render thành công cho report #{}. Yêu cầu Layer 3 cache lại compressed data.", report_id);
                        // BƯỚC 3: SAU KHI RENDER THÀNH CÔNG, YÊU CẦU LAYER 3 LƯU LẠI COMPRESSED DATA
                        // ✅ MEMORY OPTIMIZED: Pass reference instead of cloning entire Vec<u8>
                        if let Err(e) = data_service.cache_rendered_report_compressed(state, report_id, &compressed_data).await {
                            warn!("⚠️ Layer 5: Không thể cache compressed data cho report #{}: {}", report_id, e);
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
                
                // Use TemplateOrchestrator for empty template
                match self.template_orchestrator.render_empty_template(&state.tera).await {
                    Ok(html) => {
                        info!("✅ Empty template rendered successfully via TemplateOrchestrator");
                        // Compress the empty template HTML
                        match Self::compress_html_to_gzip(&html) {
                            Ok(compressed_data) => Ok(compressed_data),
                            Err(e) => {
                                error!("❌ Failed to compress empty template: {}", e);
                                Err(format!("Empty template compression error: {}", e).into())
                            }
                        }
                    }
                    Err(e) => {
                        error!("❌ TemplateOrchestrator empty template render error: {}", e);
                        Err(format!("Empty template render error: {}", e).into())
                    }
                }
            }
            Err(e) => {
                error!("❌ Database error in crypto_report_by_id: {}", e);
                Err(format!("Database error: {}", e).into())
            }
        }
    }

    /// Crypto Reports List handler - Paginated list of all reports
    /// 
    /// Delegated to Layer 3 with cache integration - similar to crypto_index_with_tera pattern
    /// Returns compressed data (Vec<u8>) for optimal transfer speed
    pub async fn crypto_reports_list_with_tera(
        &self, 
        state: &Arc<AppState>,
        page: i64
    ) -> Result<Vec<u8>, Box<dyn StdError + Send + Sync>> {
        info!("📋 Layer 5: Nhận yêu cầu cho crypto reports list page {}", page);
        
        // Increment request counter to monitor performance
        let request_count = state.request_counter.fetch_add(1, Ordering::Relaxed);
        
        // Log every 50 requests for monitoring
        if request_count.is_multiple_of(50) {
            info!("Processed {} requests to crypto_reports_list", request_count);
        }

        // BƯỚC 1: ỦY QUYỀN CHO LAYER 3 ĐỂ XỬ LÝ CACHE VÀ DATABASE (returns compressed data)
        let data_service = &self.report_creator.data_service; // Truy cập data_service
        let per_page: i64 = 10;
        
        match data_service.fetch_reports_list_with_cache(state, page, per_page).await {
            Ok(Some(compressed_data)) => {
                let size_kb = compressed_data.len() / 1024;
                info!("✅ Layer 5: Nhận compressed data từ Layer 3 cho reports list page {} ({}KB)", page, size_kb);
                Ok(compressed_data)
            }
            Ok(None) => {
                warn!("⚠️ Layer 5: Layer 3 trả về None cho reports list page {}", page);
                Err("No reports list data available".into())
            }
            Err(e) => {
                error!("❌ Layer 5: Layer 3 error cho reports list page {}: {}", page, e);
                Err(e)
            }
        }
    }

    /// Serve sandboxed report content for iframe
    ///
    /// Delegates to ReportCreator for actual sandboxed content generation.
    /// Returns Layer5Result for proper error handling.
    pub async fn serve_sandboxed_report(
        &self,
        state: &Arc<AppState>,
        report_id: i32,
        sandbox_token: &str,
        language: Option<&str>,
        chart_modules_content: Option<&str>
    ) -> Result<axum::response::Response, Box<dyn StdError + Send + Sync>> {
        info!("CryptoHandlers: Delegating sandboxed content request to ReportCreator for report {} with token {}", report_id, sandbox_token);

        // Delegate to ReportCreator and map error for backward compatibility
        self.report_creator.serve_sandboxed_report(
            state,
            report_id,
            sandbox_token,
            language,
            chart_modules_content
        ).await.map_err(|e| e.into_boxed())
    }

    /// Serve Shadow DOM content for Declarative Shadow DOM architecture
    ///
    /// Delegates to ReportCreator for actual Shadow DOM content generation.
    /// This is the modern replacement for serve_sandboxed_report.
    pub async fn serve_shadow_dom_content(
        &self,
        state: &Arc<AppState>,
        report_id: i32,
        shadow_dom_token: &str,
        language: Option<&str>,
        chart_modules_content: Option<&str>
    ) -> Result<axum::response::Response, Box<dyn StdError + Send + Sync>> {
        info!("CryptoHandlers: Delegating Shadow DOM content request to ReportCreator for report {} with token {}", report_id, shadow_dom_token);

        // Delegate to ReportCreator and map error for backward compatibility
        self.report_creator.serve_shadow_dom_content(
            state,
            report_id,
            shadow_dom_token,
            language,
            chart_modules_content
        ).await.map_err(|e| e.into_boxed())
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
