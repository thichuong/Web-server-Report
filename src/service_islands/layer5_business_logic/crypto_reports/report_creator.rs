//! Report Creator Component
//! 
//! This component handles report creation business logic for crypto reports,
//! including report data fetching, processing, and chart modules management.

use serde::{Serialize, Deserialize};
use sqlx::{FromRow};
use std::{error::Error as StdError, sync::Arc, path::Path, env};
use tokio::fs::read_dir;

// Import from current state - will be refactored when lower layers are implemented
use crate::state::AppState;

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
pub struct ReportCreator {
    // Component state will be added here
}

impl ReportCreator {
    /// Create a new ReportCreator
    pub fn new() -> Self {
        Self {
            // Initialize component
        }
    }
    
    /// Health check for report creator
    pub async fn health_check(&self) -> bool {
        // Verify report creation is working
        true // Will implement actual health check
    }

    /// Fetch and cache latest report from database
    /// 
    /// Retrieves the most recent crypto report with full content
    pub async fn fetch_and_cache_latest_report(
        &self,
        state: &Arc<AppState>,
    ) -> Result<Option<Report>, Box<dyn StdError + Send + Sync>> {
        println!("🔍 Fetching latest crypto report from database");
        
        // TODO: Implement L1/L2 cache check here
        // For now, fetch directly from database like archive_old_code/handlers/crypto.rs
        
        let query = r#"
            SELECT id, html_content, css_content, js_content, html_content_en, js_content_en, created_at
            FROM crypto_report 
            ORDER BY created_at DESC 
            LIMIT 1
        "#;
        
        let result = sqlx::query_as::<_, Report>(query)
            .fetch_optional(&state.db)
            .await?;
            
        if let Some(ref report) = result {
            println!("💾 Fetched latest crypto report {} from database", report.id);
        } else {
            println!("⚠️ No crypto reports found in database");
        }
        
        // TODO: Cache the result in L1/L2 cache
        // self.cache_manager.set_l1("latest_crypto_report", &result, Duration::from_secs(300)).await;
        
        Ok(result)
    }

    /// Fetch and cache specific report by ID
    /// 
    /// Retrieves a crypto report by its ID with full content
    pub async fn fetch_and_cache_report_by_id(
        &self,
        state: &Arc<AppState>,
        report_id: i32,
    ) -> Result<Option<Report>, Box<dyn StdError + Send + Sync>> {
        println!("🔍 Fetching crypto report {} from database", report_id);
        
        // TODO: Implement L1/L2 cache check here
        // For now, fetch directly from database like archive_old_code/handlers/crypto.rs
        
        let query = r#"
            SELECT id, html_content, css_content, js_content, html_content_en, js_content_en, created_at
            FROM crypto_report 
            WHERE id = $1
        "#;
        
        let result = sqlx::query_as::<_, Report>(query)
            .bind(report_id)
            .fetch_optional(&state.db)
            .await?;
            
        if let Some(ref report) = result {
            println!("💾 Fetched crypto report {} from database successfully", report.id);
        } else {
            println!("⚠️ Crypto report {} not found in database", report_id);
        }
        
        // TODO: Cache the result in L1/L2 cache
        // let cache_key = format!("crypto_report_{}", report_id);
        // self.cache_manager.set_l1(&cache_key, &result, Duration::from_secs(600)).await;
        
        Ok(result)
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

        // Sort files according to priority order
        all_files.sort_by(|a, b| {
            let pos_a = priority_order.iter().position(|x| x == a).unwrap_or(usize::MAX);
            let pos_b = priority_order.iter().position(|x| x == b).unwrap_or(usize::MAX);
            pos_a.cmp(&pos_b)
        });

        let mut combined_content = String::new();
        for filename in all_files {
            let file_path = source_dir.join(&filename);
            if let Ok(content) = tokio::fs::read_to_string(&file_path).await {
                if debug {
                    combined_content.push_str(&format!("\n// Chart module: {}\n", filename));
                }
                combined_content.push_str(&content);
                combined_content.push('\n');
            }
        }

        if combined_content.is_empty() {
            "// No chart modules content available".to_string()
        } else {
            combined_content
        }
    }

    /// Fetch reports list with pagination
    /// 
    /// Retrieves paginated list of crypto reports with summary information
    pub async fn fetch_reports_list_with_pagination(
        &self,
        state: &Arc<AppState>,
        page: i64,
        per_page: i64,
    ) -> Result<(Vec<ReportListItem>, i64, i64), Box<dyn StdError + Send + Sync>> {
        let offset = (page - 1) * per_page;
        
        // Get total count and reports in parallel
        let count_query = "SELECT COUNT(*) as total FROM crypto_report";
        let reports_query = r#"
            SELECT id, created_at
            FROM crypto_report 
            ORDER BY created_at DESC 
            LIMIT $1 OFFSET $2
        "#;

        let (count_result, reports_result) = tokio::try_join!(
            sqlx::query_scalar::<_, i64>(count_query).fetch_one(&state.db),
            sqlx::query_as::<_, ReportSummary>(reports_query)
                .bind(per_page)
                .bind(offset)
                .fetch_all(&state.db)
        )?;

        let total_reports = count_result;
        let total_pages = (total_reports + per_page - 1) / per_page;

        // Format reports using rayon for parallel processing (CPU-intensive date formatting)
        let items = tokio::task::spawn_blocking(move || {
            use rayon::prelude::*;
            
            reports_result
                .into_par_iter()
                .map(|report| {
                    let local_time = report.created_at.format("%d/%m/%Y").to_string();
                    let local_time_detail = report.created_at.format("%H:%M:%S").to_string();
                    
                    ReportListItem {
                        id: report.id,
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
