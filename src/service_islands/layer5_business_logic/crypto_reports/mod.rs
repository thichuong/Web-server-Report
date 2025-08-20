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
}

impl CryptoReportsIsland {
    /// Initialize the Crypto Reports Island
    /// 
    /// Creates a new Crypto Reports Island with all its components properly initialized.
    pub async fn new() -> Result<Self, anyhow::Error> {
        println!("📊 Initializing Crypto Reports Island...");
        
        let handlers = handlers::CryptoHandlers::new();
        let pdf_generator = pdf_generator::PdfGenerator::new();
        let report_creator = report_creator::ReportCreator::new();
        let data_manager = data_manager::DataManager::new();
        
        println!("✅ Crypto Reports Island initialized successfully!");
        
        Ok(Self {
            handlers,
            pdf_generator,
            report_creator,
            data_manager,
        })
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
