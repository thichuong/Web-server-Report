//! Report Manager - Report data management and caching
//!
//! Handles report retrieval, caching, and database operations
//! for the dashboard system.

use crate::features::cache_system::CacheSystem;
use crate::features::external_apis::ExternalApis;
use crate::models::{Report, ReportListItem, ReportSummary};
use serde_json::json;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Report management component
pub struct ReportManager {
    cache_system: Option<Arc<CacheSystem>>,
    external_apis: Option<Arc<ExternalApis>>,
    cached_latest_id: AtomicUsize,
}

impl ReportManager {
    pub fn new(cache_system: &CacheSystem, external_apis: &ExternalApis) -> Self {
        Self {
            cache_system: Some(Arc::new(cache_system.clone())),
            external_apis: Some(Arc::new(external_apis.clone())),
            cached_latest_id: AtomicUsize::new(0),
        }
    }

    /// Fetch and cache report by ID
    pub async fn fetch_and_cache_report_by_id(&self, id: i32) -> Result<Option<Report>, sqlx::Error> {
        // Try L1 cache first (if available)
        if let Some(cache) = &self.cache_system {
            let cache_key = format!("crypto_report:{}", id);
            if let Ok(Some(cached_report)) = cache.get::<Report>(&cache_key).await {
                println!("🔥 L1 Cache HIT for report {}", id);
                return Ok(Some(cached_report));
            }
        }

        // Cache miss - fetch from database
        // TODO: Integrate with actual database
        let report = self.mock_fetch_report_by_id(id).await?;
        
        if let (Some(report), Some(cache)) = (&report, &self.cache_system) {
            // Cache in both L1 and L2
            let cache_key = format!("crypto_report:{}", report.id);
            if let Err(e) = cache.set(&cache_key, report).await {
                eprintln!("⚠️ Failed to cache report in Redis: {}", e);
            } else {
                println!("💾 Cached crypto report {} in Redis (key: {})", report.id, cache_key);
            }

            // Update latest id if newer
            let current_latest = self.cached_latest_id.load(Ordering::Relaxed) as i32;
            if current_latest == 0 || report.id > current_latest {
                self.cached_latest_id.store(report.id as usize, Ordering::Relaxed);
            }
        }
        
        Ok(report)
    }

    /// Fetch and cache latest report
    pub async fn fetch_and_cache_latest_report(&self) -> Result<Option<Report>, sqlx::Error> {
        // Try L2 cache for latest report first
        if let Some(cache) = &self.cache_system {
            if let Ok(Some(cached_report)) = cache.get::<Report>("crypto_latest_report").await {
                println!("🔥 L2 Cache HIT for latest report");
                self.cached_latest_id.store(cached_report.id as usize, Ordering::Relaxed);
                return Ok(Some(cached_report));
            }
        }

        // Cache miss - fetch from database
        let report = self.mock_fetch_latest_report().await?;
        
        if let (Some(report), Some(cache)) = (&report, &self.cache_system) {
            self.cached_latest_id.store(report.id as usize, Ordering::Relaxed);
            
            // Cache in L2 Redis with TTL for latest report
            if let Err(e) = cache.set_with_ttl("crypto_latest_report", report, 300).await {
                eprintln!("⚠️ Failed to cache latest report in Redis: {}", e);
            } else {
                println!("💾 Cached latest crypto report {} in Redis (key: crypto_latest_report, TTL: 5min)", report.id);
            }
        }
        
        Ok(report)
    }

    /// Get report list with pagination
    pub async fn get_report_list(&self, page: Option<i32>) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        let page = page.unwrap_or(1).max(1);
        let per_page = 10;
        let offset = (page - 1) * per_page;

        // TODO: Integrate with actual database
        let reports = self.mock_fetch_report_list(offset, per_page).await?;
        let total_reports = self.mock_count_total_reports().await?;
        
        let total_pages = (total_reports + per_page - 1) / per_page;
        let has_next = page < total_pages;
        let has_prev = page > 1;

        Ok(json!({
            "reports": {
                "items": reports,
                "total": total_reports,
                "per_page": per_page,
                "current_page": page,
                "total_pages": total_pages,
                "has_next": has_next,
                "has_prev": has_prev,
                "next_page": if has_next { Some(page + 1) } else { None },
                "prev_page": if has_prev { Some(page - 1) } else { None }
            }
        }))
    }

    /// Get cached latest report ID
    pub fn get_cached_latest_id(&self) -> i32 {
        self.cached_latest_id.load(Ordering::Relaxed) as i32
    }

    /// Check if report exists in cache
    pub async fn is_report_cached(&self, id: i32) -> bool {
        if let Some(cache) = &self.cache_system {
            let cache_key = format!("crypto_report:{}", id);
            cache.exists(&cache_key).await.unwrap_or(false)
        } else {
            false
        }
    }

    // Mock database functions - TODO: Replace with actual database integration
    
    async fn mock_fetch_report_by_id(&self, id: i32) -> Result<Option<Report>, sqlx::Error> {
        // Mock implementation
        if id > 0 {
            Ok(Some(Report::default_with_id(id)))
        } else {
            Ok(None)
        }
    }

    async fn mock_fetch_latest_report(&self) -> Result<Option<Report>, sqlx::Error> {
        // Mock implementation - return latest report
        Ok(Some(Report::default_with_id(1)))
    }

    async fn mock_fetch_report_list(&self, offset: i32, limit: i32) -> Result<Vec<ReportListItem>, sqlx::Error> {
        // Mock implementation
        let mut reports = Vec::new();
        for i in 1..=limit {
            if offset + i <= 20 { // Mock total of 20 reports
                reports.push(ReportListItem {
                    id: offset + i,
                    title: format!("Báo cáo Crypto #{}", offset + i),
                    summary: format!("Tóm tắt báo cáo số {}", offset + i),
                    created_at: chrono::Utc::now() - chrono::Duration::days(i as i64),
                });
            }
        }
        Ok(reports)
    }

    async fn mock_count_total_reports(&self) -> Result<i32, sqlx::Error> {
        // Mock implementation
        Ok(20)
    }
}

impl Default for ReportManager {
    fn default() -> Self {
        Self {
            cache_system: None,
            external_apis: None,
            cached_latest_id: AtomicUsize::new(0),
        }
    }
}
