//! Crypto Reports Island - Layer 5: Business Logic
//! 
//! This island handles all crypto report-related business operations including:
//! - Advanced report creation with market analysis
//! - PDF generation with print optimization
//! - Data processing and insights generation
//! - Comprehensive API endpoints

pub mod handlers;
pub mod pdf_generator;
pub mod report_creator;
pub mod data_manager;


use std::sync::Arc;
use crate::service_islands::layer2_external_services::external_apis_island::ExternalApisIsland;

/// Crypto Reports Island
/// 
/// The main crypto reports service island that coordinates all crypto report-related
/// functionality. This island is responsible for creating reports, generating PDFs,
/// processing data, and managing crypto-specific APIs.
pub struct CryptoReportsIsland {
    pub handlers: handlers::CryptoHandlers,
    pub pdf_generator: pdf_generator::PdfGenerator,
    pub report_creator: report_creator::ReportCreator,
    pub data_manager: data_manager::DataManager,
    /// Layer 2 dependency: External APIs for real-time market data
    pub external_apis: Option<Arc<ExternalApisIsland>>,
}

impl CryptoReportsIsland {
    /// Initialize the Crypto Reports Island (basic, no dependencies)
    /// 
    /// Creates a new Crypto Reports Island with basic components.
    /// Use `with_dependencies()` for full functionality with lower layers.
    pub async fn new() -> Result<Self, anyhow::Error> {
        println!("📊 Initializing Crypto Reports Island (basic mode)...");
        
        let handlers = handlers::CryptoHandlers::new();
        let pdf_generator = pdf_generator::PdfGenerator::new();
        let report_creator = report_creator::ReportCreator::new();
        let data_manager = data_manager::DataManager::new();
        
        println!("✅ Crypto Reports Island initialized (basic mode)!");
        
        Ok(Self {
            handlers,
            pdf_generator,
            report_creator,
            data_manager,
            external_apis: None,
        })
    }
    
    /// Initialize Crypto Reports Island with Layer 2 dependencies
    /// 
    /// Creates a fully functional Crypto Reports Island with dependencies from lower layers.
    /// This enables real-time market data integration and advanced report features.
    pub async fn with_dependencies(external_apis: Arc<ExternalApisIsland>) -> Result<Self, anyhow::Error> {
        println!("📊 Initializing Crypto Reports Island with Layer 2 dependencies...");
        
        let handlers = handlers::CryptoHandlers::new();
        let pdf_generator = pdf_generator::PdfGenerator::new();
        let report_creator = report_creator::ReportCreator::new();
        let data_manager = data_manager::DataManager::new();
        
        println!("✅ Crypto Reports Island initialized with External APIs integration!");
        
        Ok(Self {
            handlers,
            pdf_generator,
            report_creator,
            data_manager,
            external_apis: Some(external_apis),
        })
    }
    
    /// Fetch real-time market data from Layer 2 External APIs
    /// 
    /// This method allows Layer 5 to get current market data directly from External APIs Island.
    /// Used for creating reports with the latest market information.
    pub async fn fetch_realtime_market_data(&self) -> Result<serde_json::Value, anyhow::Error> {
        if let Some(external_apis) = &self.external_apis {
            println!("🔄 Layer 5 fetching real-time market data from Layer 2...");
            match external_apis.fetch_dashboard_summary().await {
                Ok(market_data) => {
                    println!("✅ Layer 5 received market data from Layer 2 successfully");
                    
                    // 🔍 DEBUG: Log market data received by Layer 5
                    if let Some(btc_price) = market_data.get("btc_price_usd") {
                        println!("  🔍 [Layer 5] BTC Price received: ${:?}", btc_price);
                    }
                    if let Some(market_cap) = market_data.get("market_cap") {
                        println!("  🔍 [Layer 5] Market Cap received: ${:?}", market_cap);
                    }
                    if let Some(fng) = market_data.get("fng_value") {
                        println!("  🔍 [Layer 5] Fear & Greed received: {:?}", fng);
                    }
                    
                    Ok(market_data)
                }
                Err(e) => {
                    println!("❌ Layer 5 failed to fetch market data from Layer 2: {}", e);
                    Err(anyhow::anyhow!("Failed to fetch market data: {}", e))
                }
            }
        } else {
            println!("⚠️ Layer 5 has no Layer 2 dependencies - cannot fetch real-time data");
            Err(anyhow::anyhow!("No External APIs dependency available"))
        }
    }
    
    /// Fetch BTC price specifically for Layer 5 business logic
    pub async fn fetch_btc_price_for_reports(&self) -> Result<serde_json::Value, anyhow::Error> {
        if let Some(external_apis) = &self.external_apis {
            println!("₿ Layer 5 fetching BTC price from Layer 2 for report generation...");
            match external_apis.fetch_btc_price().await {
                Ok(btc_data) => {
                    println!("✅ Layer 5 received BTC data from Layer 2");
                    
                    // Debug log for Layer 5
                    if let Some(price) = btc_data.get("btc_price_usd") {
                        println!("  🔍 [Layer 5 Reports] BTC Price: ${:?}", price);
                    }
                    
                    Ok(btc_data)
                }
                Err(e) => {
                    println!("❌ Layer 5 BTC fetch failed: {}", e);
                    Err(anyhow::anyhow!("Failed to fetch BTC price: {}", e))
                }
            }
        } else {
            Err(anyhow::anyhow!("No External APIs dependency available"))
        }
    }
    
    /// Health check for Crypto Reports Island
    /// 
    /// Verifies that all components of the Crypto Reports Island are functioning properly.
    pub async fn health_check(&self) -> bool {
        // Check all components
        let handlers_ok = self.handlers.health_check().await;
        let pdf_ok = self.pdf_generator.health_check().await;
        let creator_ok = self.report_creator.health_check().await;
        let manager_ok = self.data_manager.health_check().await;
        
        handlers_ok && pdf_ok && creator_ok && manager_ok
    }
}
