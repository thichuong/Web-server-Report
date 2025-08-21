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
pub mod template_orchestrator;
#[cfg(test)]
pub mod tests;


use std::sync::Arc;
use crate::service_islands::layer3_communication::websocket_service::WebSocketServiceIsland;

/// Crypto Reports Island
/// 
/// The main crypto reports service island that coordinates all crypto report-related
/// functionality. This island is responsible for creating reports, generating PDFs,
/// processing data, and managing crypto-specific APIs.
/// 
/// ✅ STRICT ARCHITECTURE: Follows proper Service Islands dependency flow
/// Layer 5 → Layer 3 → Layer 2 (no direct Layer 2 access)
pub struct CryptoReportsIsland {
    pub handlers: handlers::CryptoHandlers,
    pub pdf_generator: pdf_generator::PdfGenerator,
    pub report_creator: report_creator::ReportCreator,
    pub data_manager: data_manager::DataManager,
    pub template_orchestrator: template_orchestrator::TemplateOrchestrator,
    /// ✅ Layer 3 dependency: WebSocket Service for proper architecture flow
    pub websocket_service: Option<Arc<WebSocketServiceIsland>>,
}

impl CryptoReportsIsland {
    /// Initialize the Crypto Reports Island (basic, no dependencies)
    /// 
    /// Creates a new Crypto Reports Island with basic components.
    /// Use `with_dependencies()` for full functionality with lower layers.
    pub async fn new() -> Result<Self, anyhow::Error> {
        println!("📊 Initializing Crypto Reports Island (basic mode)...");
        
        let report_creator = report_creator::ReportCreator::new();
        let handlers = handlers::CryptoHandlers::new();
        let pdf_generator = pdf_generator::PdfGenerator::new();
        let data_manager = data_manager::DataManager::new();
        let template_orchestrator = template_orchestrator::TemplateOrchestrator::new(report_creator.clone());
        
        println!("✅ Crypto Reports Island initialized (basic mode)!");
        
        Ok(Self {
            handlers,
            pdf_generator,
            report_creator,
            data_manager,
            template_orchestrator,
            websocket_service: None,
        })
    }
    
    /// Initialize Crypto Reports Island with proper Service Islands dependencies
    /// 
    /// ✅ STRICT: Only takes Layer 3 dependency (WebSocket Service) which has Layer 2 dependency.
    /// This follows strict Service Islands Architecture: Layer 5 → Layer 3 → Layer 2
    pub async fn with_dependencies(websocket_service: Arc<WebSocketServiceIsland>) -> Result<Self, anyhow::Error> {
        println!("📊 Initializing Crypto Reports Island with strict Layer 3 dependency...");
        
        let report_creator = report_creator::ReportCreator::new();
        let handlers = handlers::CryptoHandlers::new();
        let pdf_generator = pdf_generator::PdfGenerator::new();
        let data_manager = data_manager::DataManager::new();
        let template_orchestrator = template_orchestrator::TemplateOrchestrator::new(report_creator.clone());
        
        println!("✅ Crypto Reports Island initialized with strict Service Islands Architecture!");
        
        Ok(Self {
            handlers,
            pdf_generator,
            report_creator,
            data_manager,
            template_orchestrator,
            websocket_service: Some(websocket_service),
        })
    }
    
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
            match websocket_service.fetch_market_data_for_layer5().await {
                Ok(market_data) => {
                    println!("✅ Layer 5 received market data via Layer 3 → Layer 2 successfully");
                    
                    // 🔍 DEBUG: Log market data received by Layer 5
                    if let Some(btc_price) = market_data.get("btc_price_usd") {
                        println!("  🔍 [Layer 5 via Layer 3] BTC Price received: ${:?}", btc_price);
                    }
                    if let Some(market_cap) = market_data.get("market_cap_usd") {
                        println!("  🔍 [Layer 5 via Layer 3] Market Cap received: ${:?}", market_cap);
                    }
                    if let Some(fng) = market_data.get("fng_value") {
                        println!("  🔍 [Layer 5 via Layer 3] Fear & Greed received: {:?}", fng);
                    }
                    
                    // ✅ NORMALIZE: Ensure consistent field names for JavaScript client
                    let normalized_data = serde_json::json!({
                        "btc_price_usd": market_data.get("btc_price_usd").unwrap_or(&serde_json::Value::Number(serde_json::Number::from(0))),
                        "btc_change_24h": market_data.get("btc_change_24h").unwrap_or(&serde_json::Value::Number(serde_json::Number::from(0))),
                        "market_cap_usd": market_data.get("market_cap_usd").unwrap_or(&serde_json::Value::Number(serde_json::Number::from(0))),
                        "volume_24h_usd": market_data.get("volume_24h_usd").unwrap_or(&serde_json::Value::Number(serde_json::Number::from(0))),
                        "fng_value": market_data.get("fng_value").unwrap_or(&serde_json::Value::Number(serde_json::Number::from(50))),
                        "rsi_14": market_data.get("rsi_14").unwrap_or(&serde_json::Value::Number(serde_json::Number::from_f64(50.0).unwrap())),
                        "data_sources": market_data.get("data_sources").unwrap_or(&serde_json::Value::Object(serde_json::Map::new())),
                        "fetch_duration_ms": market_data.get("fetch_duration_ms").unwrap_or(&serde_json::Value::Number(serde_json::Number::from(0))),
                        "partial_failure": market_data.get("partial_failure").unwrap_or(&serde_json::Value::Bool(false)),
                        "last_updated": market_data.get("last_updated").unwrap_or(&serde_json::Value::String(chrono::Utc::now().to_rfc3339())),
                        "timestamp": market_data.get("timestamp").unwrap_or(&serde_json::Value::String(chrono::Utc::now().to_rfc3339()))
                    });
                    
                    println!("🔧 [Layer 5 via Layer 3] Data normalized for client compatibility");
                    Ok(normalized_data)
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
                        "partial_failure": market_data.get("partial_failure").unwrap_or(&serde_json::Value::Bool(false)),
                        "last_updated": market_data.get("last_updated").unwrap_or(&serde_json::Value::String(chrono::Utc::now().to_rfc3339())),
                        "timestamp": market_data.get("timestamp").unwrap_or(&serde_json::Value::String(chrono::Utc::now().to_rfc3339()))
                    });
                    
                    println!("🔧 [Layer 5 via Layer 3] Data normalized for client compatibility");
                    Ok(normalized_data)
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
        let orchestrator_ok = self.template_orchestrator.health_check().await;
        
        handlers_ok && pdf_ok && creator_ok && manager_ok && orchestrator_ok
    }
}
