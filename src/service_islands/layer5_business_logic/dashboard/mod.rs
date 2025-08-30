//! Dashboard Island - Layer 5: Business Logic
//! 
//! This island handles all dashboard-related business operations including:
//! - Dashboard UI rendering and management
//! - Report viewing and navigation
//! - Template processing with caching
//! - User interface components

pub mod handlers;
pub mod template_renderer;
pub mod report_manager;
pub mod ui_components;

use std::sync::Arc;
use crate::service_islands::layer3_communication::websocket_service::WebSocketServiceIsland;
use crate::service_islands::layer5_business_logic::market_data_service::MarketDataService;


/// Dashboard Island
/// 
/// The main dashboard service island that coordinates all dashboard-related
/// functionality. This island is responsible for rendering dashboards,
/// managing reports, and handling UI components.
/// 
/// ✅ STRICT ARCHITECTURE: Follows proper Service Islands dependency flow
/// Layer 5 → Layer 3 → Layer 2 (no direct Layer 2 access)
pub struct DashboardIsland {
    pub handlers: handlers::DashboardHandlers,
    pub template_renderer: template_renderer::TemplateRenderer,
    pub report_manager: report_manager::ReportManager,
    pub ui_components: ui_components::UIComponents,
    /// ✅ Layer 5 Market Data Service: Common service for market data operations
    pub market_data_service: Option<MarketDataService>,
}

impl DashboardIsland {
    /// Initialize Dashboard Island with proper Service Islands dependencies
    /// 
    /// ✅ STRICT: Only takes Layer 3 dependency (WebSocket Service) which has Layer 2 dependency.
    /// This follows strict Service Islands Architecture: Layer 5 → Layer 3 → Layer 2
    pub async fn with_dependencies(websocket_service: Arc<WebSocketServiceIsland>) -> Result<Self, anyhow::Error> {
        println!("🎯 Initializing Dashboard Island with strict Layer 3 dependency...");
        
        let handlers = handlers::DashboardHandlers::new();
        let template_renderer = template_renderer::TemplateRenderer::new();
        let report_manager = report_manager::ReportManager::new();
        let ui_components = ui_components::UIComponents::new();
        
        // Initialize Market Data Service with Layer 3 dependency
        let market_data_service = MarketDataService::new(websocket_service.clone());
        
        println!("✅ Dashboard Island initialized with strict Service Islands Architecture!");
        
        Ok(Self {
            handlers,
            template_renderer,
            report_manager,
            ui_components,
            market_data_service: Some(market_data_service),
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
        
        // Check market data service if available
        let market_data_ok = if let Some(market_data_service) = &self.market_data_service {
            market_data_service.health_check().await
        } else {
            println!("  ⚠️ Market Data Service not configured (using fallback)");
            true // Not critical if using fallback
        };
        
        handlers_ok && renderer_ok && manager_ok && ui_ok && market_data_ok
    }
}
