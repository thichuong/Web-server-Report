//! Dashboard Island - Layer 5: Business Logic
//!
//! This island handles all dashboard-related business operations including:
//! - Dashboard UI rendering and management
//! - Report viewing and navigation
//! - Template processing with caching
//! - User interface components
//!
//! Note: WebSocket functionality has been moved to a separate service (Web-server-Report-websocket)

use tracing::info;

pub mod handlers;
pub mod template_renderer;
pub mod report_manager;
pub mod ui_components;

/// Dashboard Island
///
/// The main dashboard service island that coordinates all dashboard-related
/// functionality. This island is responsible for rendering dashboards,
/// managing reports, and handling UI components.
///
/// Reads market data from Redis Stream (populated by WebSocket service)
pub struct DashboardIsland {
    pub handlers: handlers::DashboardHandlers,
    pub template_renderer: template_renderer::TemplateRenderer,
    pub report_manager: report_manager::ReportManager,
    pub ui_components: ui_components::UIComponents,
}

impl DashboardIsland {
    /// Initialize Dashboard Island
    ///
    /// Reads market data from Redis Stream (populated by WebSocket service)
    pub async fn new() -> Result<Self, anyhow::Error> {
        info!("🎯 Initializing Dashboard Island...");

        let handlers = handlers::DashboardHandlers::new();
        let template_renderer = template_renderer::TemplateRenderer::new();
        let report_manager = report_manager::ReportManager::new();
        let ui_components = ui_components::UIComponents::new();

        info!("✅ Dashboard Island initialized!");

        Ok(Self {
            handlers,
            template_renderer,
            report_manager,
            ui_components,
        })
    }

    /// Health check for Dashboard Island
    ///
    /// Verifies that all components of the Dashboard Island are functioning properly.
    pub async fn health_check(&self) -> bool {
        // Check all components
        let handlers_ok = self.handlers.health_check().await;
        let renderer_ok = self.template_renderer.health_check().await;
        let manager_ok = self.report_manager.health_check().await;
        let ui_ok = self.ui_components.health_check().await;

        handlers_ok && renderer_ok && manager_ok && ui_ok
    }
}
