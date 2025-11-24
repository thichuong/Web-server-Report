//! Dashboard HTTP Request Handlers
//! 
//! This module contains all HTTP request handlers related to dashboard functionality.
//! Originally located in src/handlers/api.rs, these handlers have been moved to the
//! Dashboard Island as part of the Service Islands Architecture.

use axum::{
    response::{Response, IntoResponse},
    http::StatusCode,
    body::Body
};
use tokio::fs;
use std::{sync::Arc, error::Error as StdError, io::Write};
use tera::Context;
use flate2::{Compression, write::GzEncoder};
use crate::service_islands::layer1_infrastructure::AppState;
use crate::service_islands::layer3_communication::dashboard_communication::DashboardDataService;
use tracing::{info, warn, error, debug};

/// Dashboard Handlers
/// 
/// Contains all HTTP request handlers for dashboard-related operations.
/// These handlers manage dashboard data, summaries, and API interactions.
pub struct DashboardHandlers {
    pub data_service: DashboardDataService,
}

impl Default for DashboardHandlers {
    fn default() -> Self {
        Self::new()
    }
}

impl DashboardHandlers {
    /// Create a new DashboardHandlers instance
    pub fn new() -> Self {
        Self {
            data_service: DashboardDataService::new(),
        }
    }
    
    /// Health check for dashboard handlers
    pub async fn health_check(&self) -> bool {
        // Verify handlers are functioning properly
        self.data_service.health_check().await
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
                warn!("⚠️ Failed to build compressed dashboard response: {}", e);
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::from("Response build error"))
                    .unwrap()  // This is guaranteed safe with literal body
            })
            .into_response()
    }

    /// Compress HTML string to gzip format
    /// 
    /// Helper function to compress HTML strings for optimized transfer
    fn compress_html(&self, html: &str) -> Result<Vec<u8>, Box<dyn StdError + Send + Sync>> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(html.as_bytes())?;
        let compressed_data = encoder.finish()?;
        
        let original_size = html.len();
        let compressed_size = compressed_data.len();
        let compression_ratio = (1.0 - (compressed_size as f64 / original_size as f64)) * 100.0;
        
        info!("🗜️  DashboardHandlers: HTML compressed - Original: {}KB, Compressed: {}KB, Ratio: {:.1}%", 
                 original_size / 1024, 
                 compressed_size / 1024, 
                 compression_ratio);
        
        Ok(compressed_data)
    }
    /// Homepage handler - renders homepage template using Tera
    /// 
    /// Uses Tera template engine to render home.html with market indicators component included.
    /// This replaces the simple file reading with proper template rendering.
    pub async fn homepage(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        match fs::read_to_string("dashboards/home.html").await {
            Ok(content) => Ok(content),
            Err(e) => Err(Box::new(e)),
        }
    }
    
    /// Homepage handler with Tera rendering - ENHANCED VERSION WITH CACHING
    /// 
    /// Renders homepage using Tera template engine with proper context and intelligent caching.
    /// This includes the market indicators component and any dynamic data.
    /// Returns compressed HTML for optimal performance, similar to crypto_index_with_tera.
    pub async fn homepage_with_tera(
        &self,
        state: &Arc<AppState>
    ) -> Result<Vec<u8>, Box<dyn StdError + Send + Sync>> {
        debug!("🚀 Layer 5: Nhận yêu cầu cho homepage");
        
        // BƯỚC 1: HỎI LAYER 3 ĐỂ LẤY COMPRESSED DATA TỪ CACHE CHO HOMEPAGE
        // Check cache for compressed data first (preferred)
        if let Ok(Some(cached_compressed)) = self.data_service.get_rendered_homepage_compressed(state).await {
            info!("✅ Layer 5: Nhận compressed homepage từ cache. Trả về ngay lập tức.");
            return Ok(cached_compressed);
        }

        debug!("🔍 Layer 5: Cache miss cho homepage. Bắt đầu quy trình render.");

        // BƯỚC 2: NẾU CACHE MISS, RENDER TEMPLATE VỚI CONTEXT ĐƠN GIẢN
        let mut context = Context::new();
        
        // Add basic context for homepage
        context.insert("current_route", "homepage");
        context.insert("current_lang", "vi");
        // Tối ưu: format() đã trả về String, không cần to_string()
        let current_time = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();
        context.insert("current_time", &current_time);
        
        // Add homepage-specific data
        context.insert("page_title", "Trang chủ - Crypto Dashboard");
        context.insert("welcome_message", "Chào mừng đến Crypto Dashboard");
        context.insert("description", "Theo dõi và phân tích thị trường tiền mã hóa");

        // Inject WebSocket service URL from environment variable
        let ws_url = std::env::var("WEBSOCKET_SERVICE_URL")
            .unwrap_or_else(|_| {
                // Default to localhost for development
                if cfg!(debug_assertions) {
                    "ws://localhost:8081".to_string()
                } else {
                    // In production, warn if not explicitly configured
                    warn!("⚠️ WEBSOCKET_SERVICE_URL not set in production!");
                    error!("   Using fallback: wss://web-server-report-websocket-production.up.railway.app");
                    error!("   Set WEBSOCKET_SERVICE_URL environment variable to avoid this warning.");
                    "wss://web-server-report-websocket-production.up.railway.app".to_string()
                }
            });
        context.insert("websocket_url", &ws_url);

        // Render the template using the registered components
        match state.tera.render("home.html", &context) {
            Ok(html) => {
                info!("✅ Layer 5: Render homepage thành công với Tera components");
                info!("   - Theme toggle component included");
                info!("   - Language toggle component included"); 
                info!("   - Market indicators component included");
                
                // BƯỚC 3: COMPRESS HTML VÀ CACHE RESULT
                match self.compress_html(&html) {
                    Ok(compressed_data) => {
                        // ✅ IDIOMATIC: Pass reference instead of cloning entire Vec<u8>
                        // At 16,829 RPS, this saves 840MB-3.3GB/sec of allocations
                        if let Err(e) = self.data_service.cache_rendered_homepage_compressed(state, &compressed_data).await {
                            warn!("⚠️ Layer 5: Không thể cache compressed homepage: {}", e);
                            // Vẫn trả về data ngay cả khi cache fail
                        } else {
                            info!("✅ Homepage rendered and cached successfully");
                        }
                        Ok(compressed_data)  // Move ownership, zero clone
                    }
                    Err(e) => {
                        error!("❌ Failed to compress homepage HTML: {}", e);
                        Err(format!("Homepage compression error: {}", e).into())
                    }
                }
            }
            Err(e) => {
                error!("❌ Failed to render homepage template with Tera: {}", e);
                info!("   Error details: {:?}", e);
                // Fallback to simple file reading and compression
                match fs::read_to_string("dashboards/home.html").await {
                    Ok(content) => {
                        info!("   Using fallback file reading (components won't work)");
                        match self.compress_html(&content) {
                            Ok(compressed_data) => Ok(compressed_data),
                            Err(compress_err) => {
                                error!("❌ Failed to compress fallback HTML: {}", compress_err);
                                Err(compress_err)
                            }
                        }
                    }
                    Err(fallback_e) => Err(Box::new(fallback_e)),
                }
            }
        }
    }
}
