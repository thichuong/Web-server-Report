//! Crypto Data Service
//! 
//! Layer 3 data communication service for crypto reports.
//! Handles all database operations for crypto reports, isolating business logic
//! from infrastructure concerns.

use serde::{Serialize, Deserialize};
use sqlx::{FromRow};
use std::{error::Error as StdError, sync::Arc};

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
    
    /// Health check for data service
    pub async fn health_check(&self) -> bool {
        // Verify database connectivity
        true // Will implement actual health check
    }

    /// Fetch latest crypto report from database with L2 caching
    /// 
    /// Pure data layer operation with cache integration - checks cache first, then database
    pub async fn fetch_latest_report(
        &self,
        state: &Arc<AppState>,
    ) -> Result<Option<ReportData>, sqlx::Error> {
        let cache_key = "crypto_latest_report_data";
        
        // Try L2 cache first if available
        if let Some(ref cache_system) = state.cache_system {
            if let Ok(Some(cached_data)) = cache_system.cache_manager.get(cache_key).await {
                if let Ok(cached_report) = serde_json::from_value::<ReportData>(cached_data) {
                    println!("🔥 CryptoDataService: L2 Cache HIT for latest report {}", cached_report.id);
                    return Ok(Some(cached_report));
                }
            }
        }
        
        println!("🗄️ CryptoDataService: Fetching latest crypto report from database");
        
        let report = sqlx::query_as::<_, ReportData>(
            "SELECT id, html_content, css_content, js_content, html_content_en, js_content_en, created_at FROM crypto_report ORDER BY created_at DESC LIMIT 1",
        ).fetch_optional(&state.db).await?;
        
        if let Some(ref report) = report {
            println!("📊 CryptoDataService: Retrieved latest crypto report {} from database", report.id);
            
            // Cache the result in L2 for 10 minutes (medium-term strategy)
            if let Some(ref cache_system) = state.cache_system {
                if let Ok(report_json) = serde_json::to_value(report) {
                    let _ = cache_system.cache_manager.set_with_strategy(
                        cache_key, 
                        report_json,
                        crate::service_islands::layer1_infrastructure::cache_system_island::cache_manager::CacheStrategy::MediumTerm
                    ).await;
                    println!("💾 CryptoDataService: Cached latest report {} in L2 cache", report.id);
                }
            }
        } else {
            println!("📭 CryptoDataService: No crypto reports found in database");
        }
        
        Ok(report)
    }

    /// Fetch crypto report by ID from database with L2 caching
    /// 
    /// Pure data layer operation with cache integration - retrieves specific report by ID
    pub async fn fetch_report_by_id(
        &self,
        state: &Arc<AppState>,
        report_id: i32,
    ) -> Result<Option<ReportData>, sqlx::Error> {
        let cache_key = format!("crypto_report_data_{}", report_id);
        
        // Try L2 cache first if available
        if let Some(ref cache_system) = state.cache_system {
            if let Ok(Some(cached_data)) = cache_system.cache_manager.get(&cache_key).await {
                if let Ok(cached_report) = serde_json::from_value::<ReportData>(cached_data) {
                    println!("� CryptoDataService: L2 Cache HIT for report {}", report_id);
                    return Ok(Some(cached_report));
                }
            }
        }
        
        println!("�🗄️ CryptoDataService: Fetching crypto report {} from database", report_id);
        
        let report = sqlx::query_as::<_, ReportData>(
            "SELECT id, html_content, css_content, js_content, html_content_en, js_content_en, created_at FROM crypto_report WHERE id = $1",
        )
        .bind(report_id)
        .fetch_optional(&state.db).await?;
        
        if let Some(ref report) = report {
            println!("📊 CryptoDataService: Retrieved crypto report {} from database", report.id);
            
            // Cache the result in L2 for 3 hours (long-term strategy for individual reports)
            if let Some(ref cache_system) = state.cache_system {
                if let Ok(report_json) = serde_json::to_value(report) {
                    let _ = cache_system.cache_manager.set_with_strategy(
                        &cache_key, 
                        report_json,
                        crate::service_islands::layer1_infrastructure::cache_system_island::cache_manager::CacheStrategy::LongTerm
                    ).await;
                    println!("💾 CryptoDataService: Cached report {} in L2 cache", report.id);
                }
            }
        } else {
            println!("📭 CryptoDataService: Crypto report {} not found in database", report_id);
        }
        
        Ok(report)
    }

    /// Get total count of crypto reports with L2 caching
    /// 
    /// Returns total number of reports for pagination calculations
    pub async fn get_reports_count(
        &self,
        state: &Arc<AppState>,
    ) -> Result<i64, sqlx::Error> {
        let cache_key = "crypto_reports_count";
        
        // Try L2 cache first if available
        if let Some(ref cache_system) = state.cache_system {
            if let Ok(Some(cached_data)) = cache_system.cache_manager.get(cache_key).await {
                if let Some(cached_count) = cached_data.as_i64() {
                    println!("🔥 CryptoDataService: L2 Cache HIT for reports count: {}", cached_count);
                    return Ok(cached_count);
                }
            }
        }
        
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) as total FROM crypto_report"
        ).fetch_one(&state.db).await?;
        
        // Cache the count for 5 minutes (short-term strategy as count changes less frequently)
        if let Some(ref cache_system) = state.cache_system {
            let _ = cache_system.cache_manager.set_with_strategy(
                cache_key, 
                serde_json::json!(count),
                crate::service_islands::layer1_infrastructure::cache_system_island::cache_manager::CacheStrategy::ShortTerm
            ).await;
            println!("💾 CryptoDataService: Cached reports count {} in L2 cache", count);
        }
        
        Ok(count)
    }

    /// Fetch paginated crypto reports summary with L2 caching
    /// 
    /// Pure data layer operation with cache integration - retrieves reports with pagination
    pub async fn fetch_reports_summary_paginated(
        &self,
        state: &Arc<AppState>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ReportSummaryData>, sqlx::Error> {
        let cache_key = format!("crypto_reports_summary_{}_{}", limit, offset);
        
        // Try L2 cache first if available
        if let Some(ref cache_system) = state.cache_system {
            if let Ok(Some(cached_data)) = cache_system.cache_manager.get(&cache_key).await {
                if let Ok(cached_reports) = serde_json::from_value::<Vec<ReportSummaryData>>(cached_data) {
                    println!("🔥 CryptoDataService: L2 Cache HIT for reports summary (limit:{}, offset:{})", limit, offset);
                    return Ok(cached_reports);
                }
            }
        }
        
        let reports = sqlx::query_as::<_, ReportSummaryData>(
            "SELECT id, created_at FROM crypto_report ORDER BY created_at DESC LIMIT $1 OFFSET $2"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.db).await?;
        
        println!("📊 CryptoDataService: Retrieved {} report summaries from database", reports.len());
        
        // Cache the result for 5 minutes (short-term strategy for pagination data)
        if let Some(ref cache_system) = state.cache_system {
            if let Ok(reports_json) = serde_json::to_value(&reports) {
                let _ = cache_system.cache_manager.set_with_strategy(
                    &cache_key, 
                    reports_json,
                    crate::service_islands::layer1_infrastructure::cache_system_island::cache_manager::CacheStrategy::ShortTerm
                ).await;
                println!("💾 CryptoDataService: Cached {} report summaries in L2 cache", reports.len());
            }
        }
        
        Ok(reports)
    }
    
    /// Insert new crypto report with cache invalidation
    /// 
    /// Pure data layer operation with cache management - inserts report and invalidates related cache
    pub async fn insert_crypto_report(
        &self,
        state: &Arc<AppState>,
        html_content: &str,
        css_content: Option<&str>,
        js_content: Option<&str>,
        html_content_en: Option<&str>,
        js_content_en: Option<&str>,
    ) -> Result<i32, sqlx::Error> {
        let report_id = sqlx::query_scalar::<_, i32>(
            r#"
            INSERT INTO crypto_report (html_content, css_content, js_content, html_content_en, js_content_en, created_at)
            VALUES ($1, $2, $3, $4, $5, NOW())
            RETURNING id
            "#
        )
        .bind(html_content)
        .bind(css_content)
        .bind(js_content)
        .bind(html_content_en)
        .bind(js_content_en)
        .fetch_one(&state.db).await?;
        
        println!("💾 CryptoDataService: Inserted new crypto report with ID {}", report_id);
        
        // Invalidate related caches since we have a new report
        if let Some(ref cache_system) = state.cache_system {
            // Invalidate latest report cache
            let _ = cache_system.cache_manager.remove("crypto_latest_report_data").await;
            // Invalidate count cache
            let _ = cache_system.cache_manager.remove("crypto_reports_count").await;
            println!("🧹 CryptoDataService: Invalidated related caches after insert");
        }
        
        Ok(report_id)
    }
}
