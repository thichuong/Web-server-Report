//! Crypto Reports Service Island - Layer 5 (Business Logic)
//!
//! Complete crypto reports functionality with PDF generation,
//! report creation, and data management.
//! Depends on Layers 1-4 for infrastructure services.

use crate::features::{
    shared_components::SharedComponents,
    cache_system::CacheSystem,
    external_apis::ExternalApis,
    websocket_service::WebSocketService,
    health_system::HealthSystem,
};

pub mod pdf_generator;
pub mod report_creator;
pub mod data_manager;
pub mod handlers;

use pdf_generator::PdfGenerator;
use report_creator::ReportCreator;
use data_manager::DataManager;

/// Crypto Reports Service Island
/// Layer 5: Business Logic Layer
/// Dependencies: Layers 1-4 (shared_components, cache_system, external_apis, websocket_service, health_system)
pub struct CryptoReports {
    pub pdf_generator: PdfGenerator,
    pub report_creator: ReportCreator,
    pub data_manager: DataManager,
}

impl CryptoReports {
    pub fn new(
        shared_components: &SharedComponents,
        cache_system: &CacheSystem,
        external_apis: &ExternalApis,
        websocket_service: &WebSocketService,
        health_system: &HealthSystem,
    ) -> Self {
        Self {
            pdf_generator: PdfGenerator::new(shared_components, cache_system),
            report_creator: ReportCreator::new(external_apis, cache_system),
            data_manager: DataManager::new(cache_system, external_apis),
        }
    }
    
    /// Initialize crypto reports service
    pub async fn initialize(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("🏗️ Initializing Crypto Reports Service Island (Layer 5)");
        
        // Initialize all crypto reports components
        self.pdf_generator.initialize().await?;
        self.report_creator.initialize().await?;
        self.data_manager.initialize().await?;
        
        println!("✅ Crypto Reports Service Island initialized with all components:");
        println!("   - PDF Generator: PDF report generation and templates");
        println!("   - Report Creator: Report creation and business logic");
        println!("   - Data Manager: Data processing and management");
        
        Ok(())
    }
    
    /// Generate PDF report
    pub async fn generate_pdf_report(&self, report_id: i32) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        self.pdf_generator.generate_pdf_template(report_id).await
    }
    
    /// Create new report
    pub async fn create_report(&self, data: serde_json::Value) -> Result<crate::models::Report, Box<dyn std::error::Error + Send + Sync>> {
        self.report_creator.create_new_report(data).await
    }
    
    /// Get report data with caching
    pub async fn get_report_data(&self, report_id: i32) -> Result<Option<crate::models::Report>, Box<dyn std::error::Error + Send + Sync>> {
        self.data_manager.get_report_with_cache(report_id).await
    }
    
    /// Get service health status
    pub async fn health_check(&self) -> bool {
        // Check if all crypto reports components are healthy
        self.pdf_generator.is_healthy().await
        && self.report_creator.is_healthy().await
        && self.data_manager.is_healthy().await
    }
}

impl Default for CryptoReports {
    fn default() -> Self {
        // This default is for testing only - production should use proper dependencies
        Self {
            pdf_generator: PdfGenerator::default(),
            report_creator: ReportCreator::default(),
            data_manager: DataManager::default(),
        }
    }
}

use crate::features::{Feature, FeatureHealthStatus};
use crate::features::{
    cache_system::CacheSystemFeature,
    shared_components::SharedComponentsFeature,
};
use axum::{Router, routing::get};
use std::sync::Arc;
use sqlx::PgPool;

/// Crypto Reports Feature - Core business functionality
pub struct CryptoReportsFeature {
    pub db: PgPool,
    cache_system: Arc<CacheSystemFeature>,
    shared_components: Arc<SharedComponentsFeature>,
}

impl CryptoReportsFeature {
    pub async fn new(
        database_url: &str,
        cache_system: Arc<CacheSystemFeature>,
        shared_components: Arc<SharedComponentsFeature>
    ) -> Result<Self, anyhow::Error> {
        let db = sqlx::PgPool::connect(database_url).await?;
        
        Ok(Self {
            db,
            cache_system,
            shared_components,
        })
    }
    
    /// Get HTTP routes for crypto reports endpoints
    pub fn routes(&self) -> Router<Arc<crate::state::AppState>> {
        Router::new()
            .route("/", get(crate::handlers::homepage))
            .route("/crypto_report", get(crate::handlers::crypto_index))
            .route("/crypto_report/:id", get(crate::handlers::crypto_view_report))
            .route("/pdf-template/:id", get(crate::handlers::pdf_template))
            .route("/crypto_reports_list", get(crate::handlers::report_list))
    }
}

#[async_trait::async_trait]
impl Feature for CryptoReportsFeature {
    type Config = (String, Arc<CacheSystemFeature>, Arc<SharedComponentsFeature>); // (DB URL, cache, shared)
    type Error = anyhow::Error;
    
    async fn new(config: Self::Config) -> Result<Self, Self::Error> {
        Self::new(&config.0, config.1, config.2).await
    }
    
    fn name(&self) -> &'static str {
        "crypto_reports"
    }
    
    async fn health_check(&self) -> FeatureHealthStatus {
        // Test database connectivity
        match sqlx::query("SELECT 1").fetch_one(&self.db).await {
            Ok(_) => {
                // Get report count for health details
                let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM crypto_report")
                    .fetch_one(&self.db).await
                    .unwrap_or(0);
                
                FeatureHealthStatus::healthy("crypto_reports")
                    .with_details(serde_json::json!({
                        "database": "connected",
                        "total_reports": count
                    }))
            },
            Err(e) => FeatureHealthStatus::unhealthy("crypto_reports", &format!("Database error: {}", e))
        }
    }
}
