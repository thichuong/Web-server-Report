//! Report Creator Component
//!
//! This component handles report creation business logic for crypto reports,
//! including report data fetching and processing.
//!
//! Rendering is handled by the `rendering` module:
//! - `ShadowDomRenderer`: Modern Declarative Shadow DOM rendering

use axum::http::StatusCode;
use axum::response::Response;
use std::sync::Arc;
use tracing::{debug, error, info};

// Import from current state
use crate::state::AppState;
// Import Layer 3 data communication service
use crate::services::data_communication::CryptoDataService;

// Import shared utilities
use super::super::shared::{Layer5Result, build_error_response, build_not_found_response};

// Import rendering modules
use super::rendering::ShadowDomRenderer;

// Re-export for backward compatibility
pub use super::rendering::{Report, SandboxedReport};

/// Report Creator
///
/// Manages report creation business logic with market analysis capabilities.
/// Uses Layer 3 data services and Layer 1 infrastructure services for proper architectural separation.
/// Delegates rendering to `ShadowDomRenderer` for modern Declarative Shadow DOM rendering.
#[derive(Clone)]
pub struct ReportCreator {
    pub data_service: CryptoDataService,
    pub shadow_dom_renderer: ShadowDomRenderer,
}

impl ReportCreator {
    /// Create a new `ReportCreator`
    #[must_use]
    pub fn new() -> Self {
        Self {
            data_service: CryptoDataService::new(),
            shadow_dom_renderer: ShadowDomRenderer::new(),
        }
    }

    /// Health check for report creator
    #[must_use]
    pub fn health_check(&self) -> bool {
        true
    }

    /// Fetch and cache latest report from database
    ///
    /// Retrieves the most recent crypto report with full content using Layer 3 data service.
    /// Uses From trait for automatic conversion from `ReportData` to Report.
    ///
    /// # Errors
    ///
    /// Returns error if database query fails or connection is lost
    pub async fn fetch_and_cache_latest_report(
        &self,
        state: &Arc<AppState>,
    ) -> Result<Option<Report>, sqlx::Error> {
        debug!("ReportCreator: Fetching latest crypto report from database via data service");

        let report_data = self.data_service.fetch_latest_report(state).await?;

        if let Some(data) = report_data {
            let report: Report = data.into();

            // Update latest id cache (business logic concern)
            state
                .cached_latest_id
                .store(report.id, std::sync::atomic::Ordering::Relaxed);
            debug!(
                "ReportCreator: Cached latest crypto report {} from data service",
                report.id
            );

            Ok(Some(report))
        } else {
            info!("ReportCreator: No latest crypto report available");
            Ok(None)
        }
    }

    /// Fetch and cache specific report by ID
    ///
    /// Retrieves a crypto report by its ID with full content using Layer 3 data service.
    /// Uses From trait for automatic conversion from `ReportData` to Report.
    ///
    /// # Errors
    ///
    /// Returns error if database query fails or connection is lost
    pub async fn fetch_and_cache_report_by_id(
        &self,
        state: &Arc<AppState>,
        report_id: i32,
    ) -> Result<Option<Report>, sqlx::Error> {
        debug!(
            "ReportCreator: Fetching crypto report {} via data service",
            report_id
        );

        let report_data = self
            .data_service
            .fetch_report_by_id(state, report_id)
            .await?;

        if let Some(data) = report_data {
            let report: Report = data.into();

            debug!(
                "ReportCreator: Successfully processed crypto report {} from data service",
                report.id
            );

            Ok(Some(report))
        } else {
            info!(
                "ReportCreator: Crypto report {} not found via data service",
                report_id
            );
            Ok(None)
        }
    }

    // ========================================
    // Delegation Methods for Rendering
    // ========================================

    /// Create sandboxed report (delegates to shadow DOM renderer)
    ///
    /// Creates a secure sandboxed version of the report for Shadow DOM delivery.
    #[must_use]
    pub fn create_sandboxed_report(&self, report: &Report) -> SandboxedReport {
        self.shadow_dom_renderer.create_sandboxed_report(report)
    }

    /// Generate Shadow DOM content (delegates to shadow DOM renderer)
    #[must_use]
    pub fn generate_shadow_dom_content(
        &self,
        sandboxed_report: &SandboxedReport,
        language: Option<&str>,
    ) -> String {
        self.shadow_dom_renderer
            .generate_shadow_dom_content(sandboxed_report, language)
    }

    /// Helper to fetch report by ID (handles -1 for latest)
    #[inline]
    async fn fetch_report(
        &self,
        state: &Arc<AppState>,
        report_id: i32,
    ) -> Result<Option<Report>, sqlx::Error> {
        if report_id == -1 {
            self.fetch_and_cache_latest_report(state).await
        } else {
            self.fetch_and_cache_report_by_id(state, report_id).await
        }
    }

    /// Serve Shadow DOM content (delegates to shadow DOM renderer)
    ///
    /// Uses `Layer5Result` for proper error handling without Box<dyn Error>.
    ///
    /// # Errors
    ///
    /// Returns error if database query fails, report not found, or rendering fails
    pub async fn serve_shadow_dom_content(
        &self,
        state: &Arc<AppState>,
        report_id: i32,
        shadow_dom_token: &str,
        language: Option<&str>,
    ) -> Layer5Result<Response> {
        match self.fetch_report(state, report_id).await {
            Ok(Some(report)) => self.shadow_dom_renderer.serve_shadow_dom_content(
                state,
                &report,
                shadow_dom_token,
                language,
            ),
            Ok(None) => Ok(build_not_found_response("Report not found")),
            Err(e) => {
                error!("ReportCreator: Database error: {}", e);
                Ok(build_error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Database error",
                ))
            }
        }
    }
}

impl Default for ReportCreator {
    fn default() -> Self {
        Self::new()
    }
}
