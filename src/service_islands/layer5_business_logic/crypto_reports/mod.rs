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
use crate::service_islands::layer3_communication::websocket_service::WebSocketServiceIsland;

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
    /// ✅ Layer 3 dependency: WebSocket Service for proper architecture flow
    pub websocket_service: Option<Arc<WebSocketServiceIsland>>,
}

impl CryptoReportsIsland {
    /// Initialize Crypto Reports Island with proper Service Islands dependencies
    /// 
    /// ✅ STRICT: Only takes Layer 3 dependency (WebSocket Service) which has Layer 2 dependency.
    /// This follows strict Service Islands Architecture: Layer 5 → Layer 3 → Layer 2
    pub async fn with_dependencies(websocket_service: Arc<WebSocketServiceIsland>) -> Result<Self, anyhow::Error> {
        println!("📊 Initializing Crypto Reports Island with strict Layer 3 dependency...");
        
        let report_creator = report_creator::ReportCreator::new();
        let handlers = handlers::CryptoHandlers::new();
        let data_manager = data_manager::DataManager::new();
        let template_orchestrator = template_orchestrator::TemplateOrchestrator::new(report_creator.clone());
        
        println!("✅ Crypto Reports Island initialized with strict Service Islands Architecture!");
        
        Ok(Self {
            handlers,
            report_creator,
            data_manager,
            template_orchestrator,
            websocket_service: Some(websocket_service),
        })
    }
    
    /// Fetch real-time market data via proper Service Islands Architecture
    /// 
    /// ✅ STRICT ARCHITECTURE: Layer 5 → Layer 3 → Layer 2 flow ONLY
    /// This method requests market data through Layer 3 (Communication) which fetches from Layer 2 (External APIs).
    /// This maintains proper Service Islands Architecture dependency flow.
    pub async fn fetch_realtime_market_data(&self) -> Result<serde_json::Value, anyhow::Error> {
        // ✅ STRICT: Only Layer 5 → Layer 3 → Layer 2 flow allowed
        if let Some(websocket_service) = &self.websocket_service {
            println!("🔄 Layer 5 requesting market data via Layer 3 (strict architecture)...");
            match websocket_service.fetch_market_data().await {
                Ok(market_data) => {
                    println!("✅ Layer 5 received market data via Layer 3 → Layer 2 successfully");
                    
                    // 🔍 DEBUG: Log market data received by Layer 5
                    if let Some(btc_price) = market_data.get("btc_price_usd") {
                        println!("  🔍 [Layer 5 via Layer 3] BTC Price received: ${:?}", btc_price);
                    }
                    if let Some(market_cap) = market_data.get("market_cap_usd") {
                        println!("  🔍 [Layer 5 via Layer 3] Market Cap received: ${:?}", market_cap);
                    }
                    if let Some(mc_change) = market_data.get("market_cap_change_percentage_24h_usd") {
                        println!("  🔍 [Layer 5 via Layer 3] Market Cap Change 24h received: {:?}%", mc_change);
                    }
                    if let Some(btc_dom) = market_data.get("btc_market_cap_percentage") {
                        println!("  🔍 [Layer 5 via Layer 3] BTC Dominance received: {:?}%", btc_dom);
                    }
                    if let Some(eth_dom) = market_data.get("eth_market_cap_percentage") {
                        println!("  🔍 [Layer 5 via Layer 3] ETH Dominance received: {:?}%", eth_dom);
                    }
                    if let Some(fng) = market_data.get("fng_value") {
                        println!("  🔍 [Layer 5 via Layer 3] Fear & Greed received: {:?}", fng);
                    }
                    if let Some(us_indices) = market_data.get("us_stock_indices") {
                        println!("  🔍 [Layer 5 via Layer 3] US Stock Indices received: {:?} symbols", 
                            us_indices.as_object().map_or(0, |obj| obj.len()));
                    }
                    
                    // ✅ DIRECT USE: Return Layer 3 normalized data directly (no redundant normalization)
                    println!("🔧 [Layer 5] Using Layer 3 normalized data directly for better architecture");
                    Ok(market_data)
                }
                Err(e) => {
                    println!("❌ Layer 5 → Layer 3 → Layer 2 flow failed: {}", e);
                    Err(anyhow::anyhow!("Service Islands Architecture flow failed: {}", e))
                }
            }
        } else {
            println!("❌ Layer 5 has no Layer 3 dependency - cannot fetch market data");
            Err(anyhow::anyhow!("No Layer 3 WebSocket Service dependency available - architecture violation"))
        }
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
        
        handlers_ok && creator_ok && manager_ok && orchestrator_ok
    }
}
