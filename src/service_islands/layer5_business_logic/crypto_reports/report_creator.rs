//! Report Creator Component
//! 
//! This component handles report creation business logic for crypto reports,
//! including report data fetching, processing, and chart modules management.

use serde::{Serialize, Deserialize};
use sqlx::{FromRow};
use std::sync::Arc;

// Import from current state - will be refactored when lower layers are implemented
use crate::state::AppState;
// Import Layer 3 data communication service - proper architecture
use crate::service_islands::layer3_communication::data_communication::CryptoDataService;
// Import Layer 1 infrastructure services
use crate::service_islands::layer1_infrastructure::ChartModulesIsland;

/// Report model - exactly from archive_old_code/models.rs
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Report {
    pub id: i32,
    pub html_content: String,
    pub css_content: Option<String>,
    pub js_content: Option<String>,
    pub html_content_en: Option<String>,
    pub js_content_en: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Report summary for listing - from archive_old_code/models.rs
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ReportSummary {
    pub id: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Report list item with formatted dates - from archive_old_code/models.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportListItem {
    pub id: i32,
    pub created_date: String,
    pub created_time: String,
}

/// Report Creator
/// 
/// Manages report creation business logic with market analysis capabilities.
/// Uses Layer 3 data services and Layer 1 infrastructure services for proper architectural separation.
#[derive(Clone)]
pub struct ReportCreator {
    pub data_service: CryptoDataService,
    pub chart_modules_island: ChartModulesIsland,
}

impl ReportCreator {
    /// Create a new ReportCreator
    pub fn new() -> Self {
        Self {
            data_service: CryptoDataService::new(),
            chart_modules_island: ChartModulesIsland::new(),
        }
    }
    
    /// Health check for report creator
    pub async fn health_check(&self) -> bool {
        // Verify report creation is working and chart modules are accessible
        self.chart_modules_island.health_check().await
    }

    /// Fetch and cache latest report from database
    /// 
    /// Retrieves the most recent crypto report with full content using Layer 3 data service
    pub async fn fetch_and_cache_latest_report(
        &self,
        state: &Arc<AppState>,
    ) -> Result<Option<Report>, sqlx::Error> {
        println!("🔍 ReportCreator: Fetching latest crypto report from database via data service");
        
        // Use Layer 3 data service instead of direct database access
        let report_data = self.data_service.fetch_latest_report(state).await?;
        
        if let Some(data) = report_data {
            // Convert data layer model to business layer model
            let report = Report {
                id: data.id,
                html_content: data.html_content,
                css_content: data.css_content,
                js_content: data.js_content,
                html_content_en: data.html_content_en,
                js_content_en: data.js_content_en,
                created_at: data.created_at,
            };
            
            // Update latest id cache (business logic concern)
            state.cached_latest_id.store(report.id, std::sync::atomic::Ordering::Relaxed);
            println!("💾 ReportCreator: Cached latest crypto report {} from data service", report.id);
            
            // TODO: Implement L1/L2 caching logic when cache layers are ready
            
            Ok(Some(report))
        } else {
            println!("📭 ReportCreator: No latest crypto report available");
            Ok(None)
        }
    }

    /// Fetch and cache specific report by ID
    /// 
    /// Retrieves a crypto report by its ID with full content using Layer 3 data service
    pub async fn fetch_and_cache_report_by_id(
        &self,
        state: &Arc<AppState>,
        report_id: i32,
    ) -> Result<Option<Report>, sqlx::Error> {
        println!("🔍 ReportCreator: Fetching crypto report {} via data service", report_id);
        
        // Use Layer 3 data service instead of direct database access
        let report_data = self.data_service.fetch_report_by_id(state, report_id).await?;
        
        if let Some(data) = report_data {
            // Convert data layer model to business layer model
            let report = Report {
                id: data.id,
                html_content: data.html_content,
                css_content: data.css_content,
                js_content: data.js_content,
                html_content_en: data.html_content_en,
                js_content_en: data.js_content_en,
                created_at: data.created_at,
            };
            
            println!("💾 ReportCreator: Successfully processed crypto report {} from data service", report.id);
            
            // TODO: Implement L1/L2 caching logic when cache layers are ready
            
            Ok(Some(report))
        } else {
            println!("📭 ReportCreator: Crypto report {} not found via data service", report_id);
            Ok(None)
        }
    }

    /// Get chart modules content
    /// 
    /// Delegates to Layer 1 ChartModulesIsland for proper architectural separation.
    /// This method provides a business logic wrapper around the infrastructure service.
    pub async fn get_chart_modules_content(&self) -> String {
        println!("📊 ReportCreator: Requesting chart modules from Layer 1 Infrastructure");
        
        // Delegate to Layer 1 infrastructure service
        self.chart_modules_island.get_cached_chart_modules_content().await
    }

    /// Get available chart modules
    /// 
    /// Returns a list of available chart module names via Layer 1 infrastructure.
    pub async fn get_available_chart_modules(&self) -> Vec<String> {
        self.chart_modules_island.get_available_modules().await
    }
}
