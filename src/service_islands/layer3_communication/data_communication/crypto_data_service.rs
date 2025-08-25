//! Crypto Data Service
//! 
//! Layer 3 data communication service for crypto reports.
//! Handles all database operations for crypto reports, isolating business logic
//! from infrastructure concerns.

use serde::{Serialize, Deserialize};
use sqlx::{FromRow};
use std::sync::Arc;

// Import from current state - will be refactored when lower layers are implemented
use crate::state::AppState;

/// Report model for data layer - matches business logic model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ReportData {
    pub id: i32,
    pub html_content: String,
    pub css_content: Option<String>,
    pub js_content: Option<String>,
    pub html_content_en: Option<String>,
    pub js_content_en: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Report summary for data layer
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ReportSummaryData {
    pub id: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Crypto Data Service
/// 
/// Layer 3 service responsible for all crypto report database operations.
/// Acts as the communication layer between business logic and infrastructure.
#[derive(Clone)]
pub struct CryptoDataService {
    // Service state will be added here when needed
}

impl CryptoDataService {
    /// Create a new CryptoDataService
    pub fn new() -> Self {
        Self {
            // Initialize service
        }
    }

    /// Fetch latest crypto report from database with intelligent caching (L1+L2)
    /// 
    /// Pure data layer operation with dual cache integration - checks L1 cache first, then L2, then database
    pub async fn fetch_latest_report(
        &self,
        state: &Arc<AppState>,
    ) -> Result<Option<ReportData>, sqlx::Error> {
        let cache_key = "crypto_latest_report_data";
        
        // Try cache first if available (L1 then L2 fallback with promotion) - OPTIMIZED
        if let Some(ref cache_system) = state.cache_system {
            match cache_system.cache_manager.get(cache_key).await {
                Ok(Some(cached_data)) => {
                    match serde_json::from_value::<ReportData>(cached_data) {
                        Ok(cached_report) => {
                            println!("🔥 CryptoDataService: L1 Cache HIT for latest report {}", cached_report.id);
                            return Ok(Some(cached_report));
                        }
                        Err(e) => {
                            println!("⚠️ CryptoDataService: L1 Cache deserialization error: {}", e);
                        }
                    }
                }
                Ok(None) => {
                    println!("🔍 CryptoDataService: L1 Cache MISS for latest report");
                }
                Err(e) => {
                    println!("⚠️ CryptoDataService: L1 Cache access error: {}", e);
                }
            }
        }
        
        println!("🗄️ CryptoDataService: Fetching latest crypto report from database");
        
        let report = sqlx::query_as::<_, ReportData>(
            "SELECT id, html_content, css_content, js_content, html_content_en, js_content_en, created_at FROM crypto_report ORDER BY created_at DESC LIMIT 1",
        ).fetch_optional(&state.db).await?;
        
        if let Some(ref report) = report {
            println!("📊 CryptoDataService: Retrieved latest crypto report {} from database", report.id);
            
            // Cache the result for 10 minutes in both L1 and L2
            if let Some(ref cache_system) = state.cache_system {
                if let Ok(report_json) = serde_json::to_value(report) {
                    match cache_system.cache_manager.set_with_strategy(
                        cache_key, 
                        report_json,
                        crate::service_islands::layer1_infrastructure::cache_system_island::cache_manager::CacheStrategy::ShortTerm // 5 minutes
                    ).await {
                        Ok(_) => println!("💾 CryptoDataService: Cached latest report {} for 5 minutes", report.id),
                        Err(e) => println!("⚠️ CryptoDataService: L1 Cache set error: {}", e),
                    }
                }
            }
        } else {
            println!("📭 CryptoDataService: No crypto reports found in database");
        }
        
        Ok(report)
    }

    /// Fetch crypto report by ID from database with intelligent caching (L1+L2)
    /// 
    /// Pure data layer operation with dual cache integration - retrieves specific report by ID
    pub async fn fetch_report_by_id(
        &self,
        state: &Arc<AppState>,
        report_id: i32,
    ) -> Result<Option<ReportData>, sqlx::Error> {
        let cache_key = format!("crypto_report_data_{}", report_id);
        
        // Try cache first if available (L1 then L2 fallback with promotion) - OPTIMIZED
        if let Some(ref cache_system) = state.cache_system {
            match cache_system.cache_manager.get(&cache_key).await {
                Ok(Some(cached_data)) => {
                    match serde_json::from_value::<ReportData>(cached_data) {
                        Ok(cached_report) => {
                            println!("🔥 CryptoDataService: Cache HIT for report {}", report_id);
                            return Ok(Some(cached_report));
                        }
                        Err(e) => {
                            println!("⚠️ CryptoDataService: L1 Cache deserialization error for report {}: {}", report_id, e);
                        }
                    }
                }
                Ok(None) => {
                    println!("🔍 CryptoDataService: L1 Cache MISS for report {}", report_id);
                }
                Err(e) => {
                    println!("⚠️ CryptoDataService: L1 Cache access error for report {}: {}", report_id, e);
                }
            }
        }
        
        println!("🗄️ CryptoDataService: Fetching crypto report {} from database", report_id);
        
        let report = sqlx::query_as::<_, ReportData>(
            "SELECT id, html_content, css_content, js_content, html_content_en, js_content_en, created_at FROM crypto_report WHERE id = $1",
        )
        .bind(report_id)
        .fetch_optional(&state.db).await?;
        
        if let Some(ref report) = report {
            println!("📊 CryptoDataService: Retrieved crypto report {} from database", report.id);
            
            // Cache the result for 10 minutes in both L1 and L2
            if let Some(ref cache_system) = state.cache_system {
                if let Ok(report_json) = serde_json::to_value(report) {
                    match cache_system.cache_manager.set_with_strategy(
                        &cache_key, 
                        report_json,
                        crate::service_islands::layer1_infrastructure::cache_system_island::cache_manager::CacheStrategy::ShortTerm // 10 minutes
                    ).await {
                        Ok(_) => println!("💾 CryptoDataService: Cached report {} for 10 minutes", report.id),
                        Err(e) => println!("⚠️ CryptoDataService: L1 Cache set error for report {}: {}", report.id, e),
                    }
                }
            }
        } else {
            println!("📭 CryptoDataService: Crypto report {} not found in database", report_id);
        }
        
        Ok(report)
    }

    /// Lấy nội dung HTML đã render của một report từ cache.
    pub async fn get_rendered_report_html(&self, state: &Arc<AppState>, report_id: i32) -> Result<Option<String>, anyhow::Error> {
        if let Some(ref cache_system) = state.cache_system {
            let cache_key = format!("rendered_html_report_{}", report_id);
            // Chỉ cần lấy dưới dạng String, không cần deserialize
            if let Ok(Some(cached_value)) = cache_system.cache_manager.get(&cache_key).await {
                if let Some(html_string) = cached_value.as_str() {
                    let report_type = if report_id == -1 { "latest report" } else { &format!("report #{}", report_id) };
                    println!("🔥 Layer 3: Cache HIT cho HTML đã render của {}", report_type);
                    return Ok(Some(html_string.to_string()));
                }
            }
        }
        Ok(None)
    }

    /// Lưu nội dung HTML đã render của một report vào cache.
    pub async fn cache_rendered_report_html(&self, state: &Arc<AppState>, report_id: i32, html_content: String) -> Result<(), anyhow::Error> {
        if let Some(ref cache_system) = state.cache_system {
            let cache_key = format!("rendered_html_report_{}", report_id);
            // Sử dụng chiến lược cache ShortTerm (5 phút) cho HTML đã render
            let strategy = crate::service_islands::layer1_infrastructure::cache_system_island::cache_manager::CacheStrategy::ShortTerm;
            cache_system.cache_manager.set_with_strategy(&cache_key, serde_json::json!(html_content), strategy).await?;
            let report_type = if report_id == -1 { "latest report" } else { &format!("report #{}", report_id) };
            println!("💾 Layer 3: Đã cache HTML đã render cho {}", report_type);
        }
        Ok(())
    }
}
