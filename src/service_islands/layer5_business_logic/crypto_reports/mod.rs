//! Crypto Reports Island - Layer 5: Business Logic
//!
//! This island handles all crypto report-related business operations including:
//! - Advanced report creation with market analysis
//! - Data processing and insights generation
//! - Comprehensive API endpoints
//!
//! Note: WebSocket functionality has been moved to a separate service (Web-server-Report-websocket)

use tracing::info;

pub mod data_manager;
pub mod handlers;
pub mod rendering; // Rendering strategies (iframe and Shadow DOM)
pub mod report_creator;
pub mod template_orchestrator;
#[cfg(test)]
pub mod tests;

// Re-export commonly used types for convenience
pub use rendering::{Report, SandboxedReport};

/// Crypto Reports Island
///
/// The main crypto reports service island that coordinates all crypto report-related
/// functionality. This island is responsible for creating reports, processing data,
/// and managing crypto-specific APIs.
///
/// Reads market data from Redis Stream (populated by WebSocket service)
pub struct CryptoReportsIsland {
    pub handlers: handlers::CryptoHandlers,
    pub report_creator: report_creator::ReportCreator,
    pub data_manager: data_manager::DataManager,
    pub template_orchestrator: template_orchestrator::TemplateOrchestrator,
}

impl CryptoReportsIsland {
    /// Initialize Crypto Reports Island
    ///
    /// Reads market data from Redis Stream (populated by WebSocket service)
    ///
    /// # Errors
    ///
    /// Returns error if any component initialization fails
    pub async fn new() -> Result<Self, anyhow::Error> {
        info!("📊 Initializing Crypto Reports Island...");

        let report_creator = report_creator::ReportCreator::new();
        let handlers = handlers::CryptoHandlers::new();
        let data_manager = data_manager::DataManager::new();
        let template_orchestrator =
            template_orchestrator::TemplateOrchestrator::new(report_creator.clone());

        info!("✅ Crypto Reports Island initialized!");

        Ok(Self {
            handlers,
            report_creator,
            data_manager,
            template_orchestrator,
        })
    }

    /// Health check for Crypto Reports Island
    ///
    /// Verifies that all components of the Crypto Reports Island are functioning properly.
    pub async fn health_check(&self) -> bool {
        // Check all components
        let handlers_ok = self.handlers.health_check().await;
        let creator_ok = self.report_creator.health_check().await;
        let manager_ok = self.data_manager.health_check();
        let orchestrator_ok = self.template_orchestrator.health_check().await;

        handlers_ok && creator_ok && manager_ok && orchestrator_ok
    }
}
