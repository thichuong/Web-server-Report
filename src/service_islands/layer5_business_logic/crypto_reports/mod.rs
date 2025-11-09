//! Crypto Reports Island - Layer 5: Business Logic
//! 
//! This island handles all crypto report-related business operations including:
//! - Advanced report creation with market analysis
//! - Data processing and insights generation
//! - Comprehensive API endpoints

pub mod handlers;
pub mod report_creator;
pub mod data_manager;
pub mod template_orchestrator;
#[cfg(test)]
pub mod tests;

use std::sync::Arc;
// WebSocketServiceIsland moved to separate service - only needed for with_dependencies()
#[cfg(feature = "with_websocket")]
use crate::service_islands::layer3_communication::websocket_service::WebSocketServiceIsland;
#[cfg(feature = "with_websocket")]
use crate::service_islands::layer5_business_logic::market_data_service::MarketDataService;

/// Crypto Reports Island
/// 
/// The main crypto reports service island that coordinates all crypto report-related
/// functionality. This island is responsible for creating reports, processing data,
/// and managing crypto-specific APIs.
/// 
/// ✅ STRICT ARCHITECTURE: Follows proper Service Islands dependency flow
/// Layer 5 → Layer 3 → Layer 2 (no direct Layer 2 access)
pub struct CryptoReportsIsland {
    pub handlers: handlers::CryptoHandlers,
    pub report_creator: report_creator::ReportCreator,
    pub data_manager: data_manager::DataManager,
    pub template_orchestrator: template_orchestrator::TemplateOrchestrator,
    /// ✅ Layer 5 Market Data Service: Common service for market data operations
    #[cfg(feature = "with_websocket")]
    pub market_data_service: Option<MarketDataService>,
}

impl CryptoReportsIsland {
    /// Initialize Crypto Reports Island without dependencies
    ///
    /// For main service that reads from cache/streams instead of calling external APIs directly.
    /// Market data service is not available in this mode.
    pub async fn new() -> Result<Self, anyhow::Error> {
        println!("📊 Initializing Crypto Reports Island (cache-read mode)...");

        let report_creator = report_creator::ReportCreator::new();
        let handlers = handlers::CryptoHandlers::new();
        let data_manager = data_manager::DataManager::new();
        let template_orchestrator = template_orchestrator::TemplateOrchestrator::new(report_creator.clone());

        println!("✅ Crypto Reports Island initialized (using cache for market data)!");

        Ok(Self {
            handlers,
            report_creator,
            data_manager,
            template_orchestrator,
        })
    }

    /// Initialize Crypto Reports Island with proper Service Islands dependencies
    ///
    /// ✅ STRICT: Only takes Layer 3 dependency (WebSocket Service) which has Layer 2 dependency.
    /// This follows strict Service Islands Architecture: Layer 5 → Layer 3 → Layer 2
    #[cfg(feature = "with_websocket")]
    pub async fn with_dependencies(websocket_service: Arc<WebSocketServiceIsland>) -> Result<Self, anyhow::Error> {
        println!("📊 Initializing Crypto Reports Island with strict Layer 3 dependency...");

        let report_creator = report_creator::ReportCreator::new();
        let handlers = handlers::CryptoHandlers::new();
        let data_manager = data_manager::DataManager::new();
        let template_orchestrator = template_orchestrator::TemplateOrchestrator::new(report_creator.clone());

        // Initialize Market Data Service with Layer 3 dependency
        let market_data_service = MarketDataService::new(websocket_service.clone());

        println!("✅ Crypto Reports Island initialized with strict Service Islands Architecture!");

        Ok(Self {
            handlers,
            report_creator,
            data_manager,
            template_orchestrator,
            market_data_service: Some(market_data_service),
        })
    }
    
    /// Health check for Crypto Reports Island
    /// 
    /// Verifies that all components of the Crypto Reports Island are functioning properly.
    pub async fn health_check(&self) -> bool {
        // Check all components
        let handlers_ok = self.handlers.health_check().await;
        let creator_ok = self.report_creator.health_check().await;
        let manager_ok = self.data_manager.health_check().await;
        let orchestrator_ok = self.template_orchestrator.health_check().await;
        
        // Check market data service if available
        #[cfg(feature = "with_websocket")]
        let market_data_ok = if let Some(market_data_service) = &self.market_data_service {
            market_data_service.health_check().await
        } else {
            println!("  ⚠️ Market Data Service not configured (using fallback)");
            true // Not critical if using fallback
        };
        #[cfg(not(feature = "with_websocket"))]
        let market_data_ok = {
            println!("  ℹ️ Market Data Service not available (main service mode)");
            true
        };

        handlers_ok && creator_ok && manager_ok && orchestrator_ok && market_data_ok
    }
}
