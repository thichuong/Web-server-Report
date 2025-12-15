//! Dashboard HTTP Request Handlers
//!
//! This module contains all HTTP request handlers related to dashboard functionality.
//! Originally located in src/handlers/api.rs, these handlers have been moved to the
//! Dashboard Island as part of the Service Islands Architecture.

use crate::service_islands::layer1_infrastructure::AppState;
use crate::service_islands::layer3_communication::dashboard_communication::DashboardDataService;
use axum::{
    body::Body,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use flate2::{write::GzEncoder, Compression};
use std::{error::Error as StdError, io::Write, sync::Arc};
use tera::Context;

use tracing::{debug, error, info, warn};

use tokio::sync::OnceCell;

/// Dashboard Handlers
///
/// Contains all HTTP request handlers for dashboard-related operations.
/// These handlers manage dashboard data, summaries, and API interactions.
pub struct DashboardHandlers {
    pub data_service: DashboardDataService,
    /// Cached homepage content (pre-rendered at startup)
    pub cached_homepage: OnceCell<Vec<u8>>,
}

impl Default for DashboardHandlers {
    fn default() -> Self {
        Self::new()
    }
}

impl DashboardHandlers {
    /// Create a new `DashboardHandlers` instance
    #[must_use]
    pub fn new() -> Self {
        Self {
            data_service: DashboardDataService::new(),
            cached_homepage: OnceCell::new(),
        }
    }

    /// Health check for dashboard handlers
    #[must_use]
    pub fn health_check(&self) -> bool {
        // Verify handlers are functioning properly
        self.data_service.health_check()
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
                warn!("⚠️ Failed to build compressed dashboard response: {}", e);
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::from("Response build error"))
                    .unwrap_or_else(|_| Response::new(Body::from("Response build error")))
            })
            .into_response()
    }

    /// Compress HTML string to gzip format
    ///
    /// Helper function to compress HTML strings for optimized transfer
    ///
    /// # Errors
    ///
    /// Returns error if compression fails
    fn compress_html(html: &str) -> Result<Vec<u8>, Box<dyn StdError + Send + Sync>> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(html.as_bytes())?;
        let compressed_data = encoder.finish()?;

        let original_size = html.len();
        let compressed_size = compressed_data.len();
        #[allow(clippy::cast_precision_loss)]
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
    ///
    /// # Errors
    ///
    /// Returns error if file reading fails or template parsing fails
    pub fn homepage(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        match std::fs::read_to_string("dashboards/home.html") {
            Ok(content) => Ok(content),
            Err(e) => Err(Box::new(e)),
        }
    }

    /// Initialize homepage cache
    ///
    /// Pre-renders the homepage and stores it in the cache.
    /// Should be called during application startup.
    /// Initialize homepage cache
    ///
    /// Pre-renders the homepage and stores it in the cache.
    /// Should be called during application startup.
    pub fn init_homepage_cache(&self, state: &Arc<AppState>) {
        info!("🏗️ Pre-rendering homepage to cache...");
        match Self::render_homepage_internal(state) {
            Ok(data) => {
                if self.cached_homepage.set(data).is_err() {
                    warn!("⚠️ Homepage cache already initialized");
                } else {
                    info!("✅ Homepage pre-rendered and cached in RAM");
                }
            }
            Err(e) => error!("❌ Failed to pre-render homepage: {}", e),
        }
    }

    /// Internal function to render homepage
    fn render_homepage_internal(
        state: &Arc<AppState>,
    ) -> Result<Vec<u8>, Box<dyn StdError + Send + Sync>> {
        // Render template with context
        let mut context = Context::new();

        // Add basic context for homepage
        context.insert("current_route", "homepage");
        context.insert("current_lang", "vi");
        // Fixed time for pre-rendered page - client side JS handles updates if needed
        let current_time = chrono::Utc::now()
            .format("%Y-%m-%d %H:%M:%S UTC")
            .to_string();
        context.insert("current_time", &current_time);

        // Add homepage-specific data
        context.insert("page_title", "Trang chủ - Crypto Dashboard");
        context.insert("welcome_message", "Chào mừng đến Crypto Dashboard");
        context.insert(
            "description",
            "Theo dõi và phân tích thị trường tiền mã hóa",
        );

        // Inject WebSocket service URL from environment variable
        let ws_url = std::env::var("WEBSOCKET_SERVICE_URL").unwrap_or_else(|_| {
            if cfg!(debug_assertions) {
                "ws://localhost:8081".to_string()
            } else {
                warn!("⚠️ WEBSOCKET_SERVICE_URL not set in production!");
                "wss://web-server-report-websocket-production.up.railway.app".to_string()
            }
        });
        context.insert("websocket_url", &ws_url);

        // Render the template using the registered components
        // Use synchronous render as it's fast enough
        match state.tera.render("home.html", &context) {
            Ok(html) => {
                info!("✅ Layer 5: Render homepage internal successful");
                Self::compress_html(&html)
            }
            Err(e) => {
                error!("❌ Failed to render homepage template: {}", e);
                // Fallback to simple file reading
                let html_content = std::fs::read_to_string("dashboards/home.html")?;
                Self::compress_html(&html_content)
            }
        }
    }

    /// Homepage handler with Tera rendering - OPTIMIZED RAM CACHING
    ///
    /// Returns the pre-rendered homepage from RAM.
    ///
    /// # Errors
    ///
    /// Returns error if cache retrieval fails or rendering fails
    pub fn homepage_with_tera(
        &self,
        state: &Arc<AppState>,
    ) -> Result<Vec<u8>, Box<dyn StdError + Send + Sync>> {
        // Optimized: Return cached content from RAM
        if let Some(cached) = self.cached_homepage.get() {
            debug!("⚡ Serving homepage from RAM cache");
            return Ok(cached.clone());
        }

        // Fallback: If not initialized, render and return (lazy init)
        debug!("⚠️ Homepage cache miss (lazy init)");
        let data = Self::render_homepage_internal(state)?;

        // Try to set cache for next time
        let _ = self.cached_homepage.set(data.clone());

        Ok(data)
    }
}
