//! Dashboard Service Island - Layer 5: Business Logic
//!
//! Provides dashboard UI components, report rendering, template management,
//! and business logic for the crypto investment dashboard.
//!
//! ## Dependencies (Layer 5 - Business Logic)
//! - Layer 1: shared_components (templates, models, utilities)
//! - Layer 1: cache_system (report caching, template caching)
//! - Layer 2: external_apis (market data for dashboard)
//! - Layer 3: websocket_service (real-time dashboard updates)
//! - Layer 4: health_system (dashboard health monitoring)
//!
//! ## Service Island Architecture
//! This module handles all dashboard-related business logic including
//! report rendering, UI components, and user interactions.

pub mod template_renderer;
pub mod report_manager;
pub mod ui_components;
pub mod handlers;

pub use template_renderer::TemplateRenderer;
pub use report_manager::ReportManager;
pub use ui_components::UiComponents;
pub use handlers::dashboard_routes;

use crate::features::shared_components::SharedComponents;
use crate::features::cache_system::CacheSystem;
use crate::features::external_apis::ExternalApis;
use crate::features::websocket_service::WebSocketService;

/// Dashboard Service Island - Layer 5 (Business Logic)
/// 
/// Provides comprehensive dashboard functionality including report rendering,
/// UI components, template management, and real-time updates.
pub struct Dashboard {
    pub template_renderer: TemplateRenderer,
    pub report_manager: ReportManager,
    pub ui_components: UiComponents,
}

impl Dashboard {
    pub fn new(
        shared_components: &SharedComponents,
        cache_system: &CacheSystem,
        external_apis: &ExternalApis,
        websocket_service: &WebSocketService,
    ) -> Self {
        Self {
            template_renderer: TemplateRenderer::new(shared_components, cache_system),
            report_manager: ReportManager::new(cache_system, external_apis),
            ui_components: UiComponents::new(shared_components, websocket_service),
        }
    }

    /// Render crypto dashboard with real-time data
    pub async fn render_crypto_dashboard(&self, report_id: Option<i32>) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        self.template_renderer.render_dashboard_view(report_id).await
    }

    /// Render PDF template for reports
    pub async fn render_pdf_template(&self, report_id: i32) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        self.template_renderer.render_pdf_template(report_id).await
    }

    /// Get report list with pagination
    pub async fn get_report_list(&self, page: Option<i32>) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        self.report_manager.get_report_list(page).await
    }

    /// Get homepage HTML
    pub async fn render_homepage(&self) -> Result<String, std::io::Error> {
        self.ui_components.render_homepage().await
    }

    /// Get Axum routes for dashboard endpoints
    pub fn routes() -> axum::Router<std::sync::Arc<crate::state::AppState>> {
        handlers::dashboard_routes()
    }
}

impl Default for Dashboard {
    fn default() -> Self {
        // This default is for testing only - production should use proper dependencies
        Self {
            template_renderer: TemplateRenderer::default(),
            report_manager: ReportManager::default(),
            ui_components: UiComponents::default(),
        }
    }
}

use crate::features::{Feature, FeatureHealthStatus};
use crate::features::{
    external_apis::ExternalApisFeature,
    cache_system::CacheSystemFeature,
    websocket_service::WebSocketServiceFeature,
};
use axum::{Router, routing::get};
use std::sync::Arc;

/// Dashboard Feature - Provides market data visualization and APIs  
pub struct DashboardFeature {
    external_apis: Arc<ExternalApisFeature>,
    cache_system: Arc<CacheSystemFeature>,
    websocket_service: Arc<WebSocketServiceFeature>,
}

impl DashboardFeature {
    pub async fn new(
        external_apis: Arc<ExternalApisFeature>,
        cache_system: Arc<CacheSystemFeature>, 
        websocket_service: Arc<WebSocketServiceFeature>
    ) -> Result<Self, anyhow::Error> {
        Ok(Self {
            external_apis,
            cache_system,
            websocket_service,
        })
    }
    
    /// Get HTTP routes for dashboard endpoints
    pub fn routes(&self) -> Router<Arc<crate::state::AppState>> {
        Router::new()
            .route("/api/crypto/dashboard-summary", get(crate::handlers::api_dashboard_summary))
            .route("/api/crypto/dashboard-summary/cached", get(crate::handlers::dashboard_summary_api))
            .route("/api/crypto/dashboard-summary/refresh", get(crate::handlers::force_refresh_dashboard))
            .route("/api/crypto/rate-limit-status", get(crate::handlers::api_rate_limit_status))
    }
}

#[async_trait::async_trait]
impl Feature for DashboardFeature {
    type Config = (Arc<ExternalApisFeature>, Arc<CacheSystemFeature>, Arc<WebSocketServiceFeature>);
    type Error = anyhow::Error;
    
    async fn new(config: Self::Config) -> Result<Self, Self::Error> {
        Self::new(config.0, config.1, config.2).await
    }
    
    fn name(&self) -> &'static str {
        "dashboard"
    }
    
    async fn health_check(&self) -> FeatureHealthStatus {
        // Check if dashboard can fetch data successfully
        match self.external_apis.data_service().fetch_dashboard_summary().await {
            Ok(_) => FeatureHealthStatus::healthy("dashboard")
                .with_details(serde_json::json!({
                    "external_apis": "connected",
                    "real_time_updates": "active"
                })),
            Err(e) => FeatureHealthStatus::unhealthy("dashboard", &format!("Data fetch failed: {}", e))
                .with_details(serde_json::json!({
                    "error": e.to_string()
                }))
        }
    }
}
