//! Report Creator Component
//!
//! This component handles report creation business logic for crypto reports,
//! including report data fetching, processing, and chart modules management.
//!
//! Rendering strategies have been moved to the `rendering` module:
//! - IframeRenderer: Legacy iframe-based rendering
//! - ShadowDomRenderer: Modern Declarative Shadow DOM rendering

use std::{sync::Arc, error::Error as StdError};
use tracing::{info, debug};
use axum::{
    response::Response,
};

// Import from current state - will be refactored when lower layers are implemented
use crate::service_islands::layer1_infrastructure::AppState;
// Import Layer 3 data communication service - proper architecture
use crate::service_islands::layer3_communication::data_communication::CryptoDataService;
// Import Layer 1 infrastructure services
use crate::service_islands::layer1_infrastructure::ChartModulesIsland;

// Import rendering modules
use super::rendering::{
    IframeRenderer,
    ShadowDomRenderer,
};

// Re-export for backward compatibility
pub use super::rendering::{Report, SandboxedReport};

/// Report Creator
///
/// Manages report creation business logic with market analysis capabilities.
/// Uses Layer 3 data services and Layer 1 infrastructure services for proper architectural separation.
/// Delegates rendering to specialized renderers (Iframe and Shadow DOM).
#[derive(Clone)]
pub struct ReportCreator {
    pub data_service: CryptoDataService,
    pub chart_modules_island: ChartModulesIsland,
    pub iframe_renderer: IframeRenderer,
    pub shadow_dom_renderer: ShadowDomRenderer,
}

impl ReportCreator {
    /// Create a new ReportCreator
    pub fn new() -> Self {
        Self {
            data_service: CryptoDataService::new(),
            chart_modules_island: ChartModulesIsland::new(),
            iframe_renderer: IframeRenderer::new(),
            shadow_dom_renderer: ShadowDomRenderer::new(),
        }
    }

    /// Health check for report creator
    pub async fn health_check(&self) -> bool {
        // Verify report creation is working and chart modules are accessible
        self.chart_modules_island.health_check().await
    }

    /// Fetch and cache latest report from database
    ///
    /// Retrieves the most recent crypto report with full content using Layer 3 data service
    pub async fn fetch_and_cache_latest_report(
        &self,
        state: &Arc<AppState>,
    ) -> Result<Option<Report>, sqlx::Error> {
        debug!("🔍 ReportCreator: Fetching latest crypto report from database via data service");

        // Use Layer 3 data service instead of direct database access
        let report_data = self.data_service.fetch_latest_report(state).await?;

        if let Some(data) = report_data {
            // Convert data layer model to business layer model
            let report = Report {
                id: data.id,
                html_content: data.html_content,
                css_content: data.css_content,
                js_content: data.js_content,
                html_content_en: data.html_content_en,
                js_content_en: data.js_content_en,
                created_at: data.created_at,
            };

            // Update latest id cache (business logic concern)
            state.cached_latest_id.store(report.id, std::sync::atomic::Ordering::Relaxed);
            debug!("💾 ReportCreator: Cached latest crypto report {} from data service", report.id);

            // TODO: Implement L1/L2 caching logic when cache layers are ready

            Ok(Some(report))
        } else {
            info!("📭 ReportCreator: No latest crypto report available");
            Ok(None)
        }
    }

    /// Fetch and cache specific report by ID
    ///
    /// Retrieves a crypto report by its ID with full content using Layer 3 data service
    pub async fn fetch_and_cache_report_by_id(
        &self,
        state: &Arc<AppState>,
        report_id: i32,
    ) -> Result<Option<Report>, sqlx::Error> {
        debug!("🔍 ReportCreator: Fetching crypto report {} via data service", report_id);

        // Use Layer 3 data service instead of direct database access
        let report_data = self.data_service.fetch_report_by_id(state, report_id).await?;

        if let Some(data) = report_data {
            // Convert data layer model to business layer model
            let report = Report {
                id: data.id,
                html_content: data.html_content,
                css_content: data.css_content,
                js_content: data.js_content,
                html_content_en: data.html_content_en,
                js_content_en: data.js_content_en,
                created_at: data.created_at,
            };

            debug!("💾 ReportCreator: Successfully processed crypto report {} from data service", report.id);

            // TODO: Implement L1/L2 caching logic when cache layers are ready

            Ok(Some(report))
        } else {
            info!("📭 ReportCreator: Crypto report {} not found via data service", report_id);
            Ok(None)
        }
    }

    /// Get chart modules content
    ///
    /// Delegates to Layer 1 ChartModulesIsland for proper architectural separation.
    /// This method provides a business logic wrapper around the infrastructure service.
    pub async fn get_chart_modules_content(&self) -> String {
        debug!("📊 ReportCreator: Requesting chart modules from Layer 1 Infrastructure");

        // Delegate to Layer 1 infrastructure service
        self.chart_modules_island.get_cached_chart_modules_content().await
    }

    /// Get available chart modules
    ///
    /// Returns a list of available chart module names via Layer 1 infrastructure.
    #[allow(dead_code)]
    pub async fn get_available_chart_modules(&self) -> Vec<String> {
        self.chart_modules_island.get_available_modules().await
    }

    // ========================================
    // Delegation Methods for Rendering
    // ========================================

    /// Create sandboxed report (delegates to iframe renderer)
    ///
    /// This maintains backward compatibility with existing code
    pub fn create_sandboxed_report(&self, report: &Report, chart_modules_content: Option<&str>) -> SandboxedReport {
        self.iframe_renderer.create_sandboxed_report(report, chart_modules_content)
    }

    /// Generate Shadow DOM content (delegates to shadow DOM renderer)
    pub fn generate_shadow_dom_content(&self, sandboxed_report: &SandboxedReport, language: Option<&str>, chart_modules_content: Option<&str>) -> String {
        self.shadow_dom_renderer.generate_shadow_dom_content(sandboxed_report, language, chart_modules_content)
    }

    /// Serve sandboxed report content for iframe (delegates to iframe renderer)
    pub async fn serve_sandboxed_report(
        &self,
        state: &Arc<AppState>,
        report_id: i32,
        sandbox_token: &str,
        language: Option<&str>,
        chart_modules_content: Option<&str>
    ) -> Result<Response, Box<dyn StdError + Send + Sync>> {
        // Fetch report
        let report_result = if report_id == -1 {
            self.fetch_and_cache_latest_report(state).await
        } else {
            self.fetch_and_cache_report_by_id(state, report_id).await
        };

        match report_result {
            Ok(Some(report)) => {
                self.iframe_renderer.serve_sandboxed_report(state, &report, sandbox_token, language, chart_modules_content).await
            }
            Ok(None) => {
                use axum::http::StatusCode;
                use axum::body::Body;
                use axum::response::IntoResponse;
                Ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .header("content-type", "text/plain")
                    .body(Body::from("Report not found"))
                    .unwrap()
                    .into_response()
                )
            }
            Err(e) => {
                use axum::http::StatusCode;
                use axum::body::Body;
                use axum::response::IntoResponse;
                use tracing::error;
                error!("❌ ReportCreator: Database error: {}", e);
                Ok(Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header("content-type", "text/plain")
                    .body(Body::from("Database error"))
                    .unwrap()
                    .into_response()
                )
            }
        }
    }

    /// Serve Shadow DOM content (delegates to shadow DOM renderer)
    pub async fn serve_shadow_dom_content(
        &self,
        state: &Arc<AppState>,
        report_id: i32,
        shadow_dom_token: &str,
        language: Option<&str>,
        chart_modules_content: Option<&str>
    ) -> Result<Response, Box<dyn StdError + Send + Sync>> {
        // Fetch report
        let report_result = if report_id == -1 {
            self.fetch_and_cache_latest_report(state).await
        } else {
            self.fetch_and_cache_report_by_id(state, report_id).await
        };

        match report_result {
            Ok(Some(report)) => {
                self.shadow_dom_renderer.serve_shadow_dom_content(state, &report, shadow_dom_token, language, chart_modules_content).await
            }
            Ok(None) => {
                use axum::http::StatusCode;
                use axum::body::Body;
                use axum::response::IntoResponse;
                Ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .header("content-type", "text/plain")
                    .body(Body::from("Report not found"))
                    .unwrap()
                    .into_response()
                )
            }
            Err(e) => {
                use axum::http::StatusCode;
                use axum::body::Body;
                use axum::response::IntoResponse;
                use tracing::error;
                error!("❌ ReportCreator: Database error: {}", e);
                Ok(Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header("content-type", "text/plain")
                    .body(Body::from("Database error"))
                    .unwrap()
                    .into_response()
                )
            }
        }
    }
}
