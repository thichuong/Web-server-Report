//! Report Creator Component
//! 
//! This component handles report creation business logic for crypto reports,
//! including report data fetching, processing, and chart modules management.

use serde::{Serialize, Deserialize};
use sqlx::{FromRow};
use std::{error::Error as StdError, sync::Arc, path::Path, env, sync::atomic::Ordering};
use tokio::fs::read_dir;

// Import from current state - will be refactored when lower layers are implemented
use crate::state::AppState;
// Import Layer 3 data communication service - proper architecture
use crate::service_islands::layer3_communication::data_communication::CryptoDataService;

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
/// Uses Layer 3 data services for proper architectural separation.
#[derive(Clone)]
pub struct ReportCreator {
    data_service: CryptoDataService,
}

impl ReportCreator {
    /// Create a new ReportCreator
    pub fn new() -> Self {
        Self {
            data_service: CryptoDataService::new(),
        }
    }
    
    /// Health check for report creator
    pub async fn health_check(&self) -> bool {
        // Verify report creation is working
        true // Will implement actual health check
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
    /// Exactly from archive_old_code/utils.rs::get_chart_modules_content
    /// Reads actual chart modules from shared_assets/js/chart_modules/
    pub async fn get_chart_modules_content(&self) -> String {
        // TODO: Add chart modules cache when cache layer is ready
        // For now always read from files (like debug mode in archive)
        let debug = env::var("DEBUG").unwrap_or_default() == "1";
        
        let source_dir = Path::new("shared_assets").join("js").join("chart_modules");
        let priority_order = vec!["gauge.js", "bar.js", "line.js", "doughnut.js"];

        let mut entries = match read_dir(&source_dir).await {
            Ok(rd) => rd,
            Err(_) => return "// No chart modules found".to_string(),
        };

        let mut all_files = Vec::new();
        while let Ok(Some(entry)) = entries.next_entry().await {
            if let Ok(ft) = entry.file_type().await {
                if ft.is_file() {
                    if let Some(name) = entry.file_name().to_str() {
                        if name.ends_with(".js") {
                            all_files.push(name.to_string());
                        }
                    }
                }
            }
        }

        // Order files: priority first, then alphabetically
        let mut ordered = Vec::new();
        for p in &priority_order {
            if let Some(idx) = all_files.iter().position(|f| f == p) {
                ordered.push(all_files.remove(idx));
            }
        }
        all_files.sort();
        ordered.extend(all_files);

        // Parallel file reading with concurrent futures
        let file_futures: Vec<_> = ordered
            .iter()
            .map(|filename| {
                let path = source_dir.join(filename);
                let filename_clone = filename.clone();
                async move {
                    match tokio::fs::read_to_string(&path).await {
                        Ok(content) => {
                            let wrapped = format!(
                                "// ==================== {name} ====================\ntry {{\n{code}\n}} catch (error) {{\n    console.error('Error loading chart module {name}:', error);\n}}\n// ==================== End {name} ====================",
                                name = filename_clone,
                                code = content
                            );
                            wrapped
                        }
                        Err(_) => {
                            format!("// Warning: {name} not found", name = filename_clone)
                        }
                    }
                }
            })
            .collect();

        // Await all file reads concurrently like archive code
        let parts = futures::future::join_all(file_futures).await;

        // Final concatenation in CPU thread pool to avoid blocking async runtime
        let final_content = tokio::task::spawn_blocking(move || {
            parts.join("\n\n")
        }).await.unwrap_or_else(|e| {
            eprintln!("Chart modules concatenation error: {:#?}", e);
            "// Error loading chart modules".to_string()
        });

        final_content
    }

    /// Fetch reports list with pagination
    /// 
    /// Retrieves paginated list of crypto reports with summary information using Layer 3 data service
    pub async fn fetch_reports_list_with_pagination(
        &self,
        state: &Arc<AppState>,
        page: i64,
        per_page: i64,
    ) -> Result<(Vec<ReportListItem>, i64, i64), Box<dyn StdError + Send + Sync>> {
        let offset = (page - 1) * per_page;
        
        // Get total count and reports via data service (Layer 3) in parallel
        let (count_result, reports_result) = tokio::try_join!(
            self.data_service.get_reports_count(state),
            self.data_service.fetch_reports_summary_paginated(state, per_page, offset)
        )?;

        let total_reports = count_result;
        let total_pages = (total_reports + per_page - 1) / per_page;

        // Format reports using rayon for parallel processing (CPU-intensive date formatting)
        // This is business logic - converting data models to presentation models
        let items = tokio::task::spawn_blocking(move || {
            use rayon::prelude::*;
            
            reports_result
                .into_par_iter()
                .map(|report_data| {
                    let local_time = report_data.created_at.format("%d/%m/%Y").to_string();
                    let local_time_detail = report_data.created_at.format("%H:%M:%S").to_string();
                    
                    ReportListItem {
                        id: report_data.id,
                        created_date: local_time,
                        created_time: local_time_detail,
                    }
                })
                .collect::<Vec<_>>()
        }).await?;

        Ok((items, total_pages, total_reports))
    }
    
    /// Create new crypto report
    /// 
    /// This method will handle the creation of new crypto reports with market analysis.
    /// Currently placeholder - will implement with actual report creation logic.
    pub async fn create_crypto_report(&self, market_data: &str) -> Result<i32, Box<dyn std::error::Error + Send + Sync>> {
        // Placeholder implementation
        // Will integrate with market data and report generation logic
        Ok(1) // Return dummy report ID for now
    }
}
