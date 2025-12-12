//! Report Creator Component
//!
//! This component handles report creation business logic for crypto reports,
//! including report data fetching, processing, and chart modules management.
//!
//! Rendering is handled by the `rendering` module:
//! - `ShadowDomRenderer`: Modern Declarative Shadow DOM rendering

use axum::http::StatusCode;
use axum::response::Response;
use std::sync::Arc;
use tracing::{debug, error, info};

// Import from current state - will be refactored when lower layers are implemented
use crate::service_islands::layer1_infrastructure::AppState;
// Import Layer 3 data communication service - proper architecture
use crate::service_islands::layer3_communication::data_communication::CryptoDataService;
// Import Layer 1 infrastructure services
use crate::service_islands::layer1_infrastructure::ChartModulesIsland;

// Import shared utilities
use super::super::shared::{build_error_response, build_not_found_response, Layer5Result};

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
    pub chart_modules_island: ChartModulesIsland,
    pub shadow_dom_renderer: ShadowDomRenderer,
}

impl ReportCreator {
    /// Create a new `ReportCreator`
    #[must_use]
    pub fn new() -> Self {
        Self {
            data_service: CryptoDataService::new(),
            chart_modules_island: ChartModulesIsland::new(),
            shadow_dom_renderer: ShadowDomRenderer::new(),
        }
    }

    /// Health check for report creator
    #[must_use]
    pub fn health_check(&self) -> bool {
        // Verify report creation is working and chart modules are accessible
        self.chart_modules_island.health_check()
    }

    /// Helper to run async code synchronously
    fn block_on<F: std::future::Future>(future: F) -> F::Output {
        // Try to obtain a handle to the current runtime
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            tokio::task::block_in_place(move || handle.block_on(future))
        } else {
            // Fallback for non-tokio contexts
            futures::executor::block_on(future)
        }
    }

    /// Fetch and cache latest report from database
    ///
    /// Retrieves the most recent crypto report with full content using Layer 3 data service.
    /// Uses From trait for automatic conversion from `ReportData` to Report.
    ///
    /// # Errors
    ///
    /// Returns error if database query fails or connection is lost
    /// Fetch and cache latest report from database
    ///
    /// Retrieves the most recent crypto report with full content using Layer 3 data service.
    /// Uses From trait for automatic conversion from `ReportData` to Report.
    ///
    /// # Errors
    ///
    /// Returns error if database query fails or connection is lost
    pub fn fetch_and_cache_latest_report(
        &self,
        state: &Arc<AppState>,
    ) -> Result<Option<Report>, sqlx::Error> {
        debug!("ReportCreator: Fetching latest crypto report from database via data service");

        // Use Layer 3 data service instead of direct database access
        // ✅ SYNC: data_service is now synchronous
        let report_data = self.data_service.fetch_latest_report(state)?;

        if let Some(data) = report_data {
            // Use From trait for automatic conversion - zero boilerplate!
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
    /// Fetch and cache specific report by ID
    ///
    /// Retrieves a crypto report by its ID with full content using Layer 3 data service.
    /// Uses From trait for automatic conversion from `ReportData` to Report.
    ///
    /// # Errors
    ///
    /// Returns error if database query fails or connection is lost
    pub fn fetch_and_cache_report_by_id(
        &self,
        state: &Arc<AppState>,
        report_id: i32,
    ) -> Result<Option<Report>, sqlx::Error> {
        debug!(
            "ReportCreator: Fetching crypto report {} via data service",
            report_id
        );

        // Use Layer 3 data service instead of direct database access
        // ✅ SYNC: data_service is now synchronous
        let report_data = self.data_service.fetch_report_by_id(state, report_id)?;

        if let Some(data) = report_data {
            // Use From trait for automatic conversion - zero boilerplate!
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

    /// Get chart modules content
    ///
    /// Delegates to Layer 1 `ChartModulesIsland` for proper architectural separation.
    /// This method provides a business logic wrapper around the infrastructure service.
    /// Get chart modules content
    ///
    /// Delegates to Layer 1 `ChartModulesIsland` for proper architectural separation.
    /// This method provides a business logic wrapper around the infrastructure service.
    pub fn get_chart_modules_content(&self) -> String {
        debug!("ReportCreator: Requesting chart modules from Layer 1 Infrastructure");

        // Delegate to Layer 1 infrastructure service
        // ✅ SYNC-WRAPPER: Layer 1 is async, so we wrap it
        Self::block_on(self.chart_modules_island.get_cached_chart_modules_content())
    }

    /// Get available chart modules
    ///
    /// Returns a list of available chart module names via Layer 1 infrastructure.
    #[must_use]
    pub fn get_available_chart_modules(&self) -> Vec<String> {
        Self::block_on(self.chart_modules_island.get_available_modules())
    }

    // ========================================
    // Delegation Methods for Rendering
    // ========================================

    /// Create sandboxed report (delegates to shadow DOM renderer)
    ///
    /// Creates a secure sandboxed version of the report for Shadow DOM delivery.
    #[must_use]
    pub fn create_sandboxed_report(
        &self,
        report: &Report,
        chart_modules_content: Option<&str>,
    ) -> SandboxedReport {
        self.shadow_dom_renderer
            .create_sandboxed_report(report, chart_modules_content)
    }

    /// Generate Shadow DOM content (delegates to shadow DOM renderer)
    #[must_use]
    pub fn generate_shadow_dom_content(
        &self,
        sandboxed_report: &SandboxedReport,
        language: Option<&str>,
        chart_modules_content: Option<&str>,
    ) -> String {
        self.shadow_dom_renderer.generate_shadow_dom_content(
            sandboxed_report,
            language,
            chart_modules_content,
        )
    }

    /// Helper to fetch report by ID (handles -1 for latest)
    #[inline]
    fn fetch_report(
        &self,
        state: &Arc<AppState>,
        report_id: i32,
    ) -> Result<Option<Report>, sqlx::Error> {
        if report_id == -1 {
            self.fetch_and_cache_latest_report(state)
        } else {
            self.fetch_and_cache_report_by_id(state, report_id)
        }
    }

    /// Serve sandboxed report content (delegates to shadow DOM renderer)
    ///
    /// Uses `Layer5Result` for proper error handling without Box<dyn Error>.
    /// This method provides backward compatibility for the API endpoint.
    ///
    /// # Errors
    ///
    /// Returns error if database query fails, report not found, or rendering fails
    /// Serve sandboxed report content (delegates to shadow DOM renderer)
    ///
    /// Uses `Layer5Result` for proper error handling without Box<dyn Error>.
    /// This method provides backward compatibility for the API endpoint.
    ///
    /// # Errors
    ///
    /// Returns error if database query fails, report not found, or rendering fails
    pub fn serve_sandboxed_report(
        &self,
        state: &Arc<AppState>,
        report_id: i32,
        sandbox_token: &str,
        language: Option<&str>,
        chart_modules_content: Option<&str>,
    ) -> Layer5Result<Response> {
        match self.fetch_report(state, report_id) {
            Ok(Some(report)) => self.shadow_dom_renderer.serve_shadow_dom_content(
                state,
                &report,
                sandbox_token,
                language,
                chart_modules_content,
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

    /// Serve Shadow DOM content (delegates to shadow DOM renderer)
    ///
    /// Uses `Layer5Result` for proper error handling without Box<dyn Error>.
    ///
    /// # Errors
    ///
    /// Returns error if database query fails, report not found, or rendering fails
    /// Serve Shadow DOM content (delegates to shadow DOM renderer)
    ///
    /// Uses `Layer5Result` for proper error handling without Box<dyn Error>.
    ///
    /// # Errors
    ///
    /// Returns error if database query fails, report not found, or rendering fails
    pub fn serve_shadow_dom_content(
        &self,
        state: &Arc<AppState>,
        report_id: i32,
        shadow_dom_token: &str,
        language: Option<&str>,
        chart_modules_content: Option<&str>,
    ) -> Layer5Result<Response> {
        match self.fetch_report(state, report_id) {
            Ok(Some(report)) => self.shadow_dom_renderer.serve_shadow_dom_content(
                state,
                &report,
                shadow_dom_token,
                language,
                chart_modules_content,
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
