// src/features/shared_components/templates/mod.rs
//
// Template utilities and helpers for Tera template engine
// Provides common template functions and context management

use tera::{Tera, Context};
use std::collections::HashMap;

/// Template registration utilities
pub struct TemplateRegistry;

impl TemplateRegistry {
    /// Register all shared templates with logical names
    pub fn register_shared_templates(tera: &mut Tera) -> Result<(), anyhow::Error> {
        // Shared components
        tera.add_template_file(
            "shared_components/theme_toggle.html",
            Some("shared/components/theme_toggle.html")
        )?;
        
        tera.add_template_file(
            "shared_components/language_toggle.html", 
            Some("shared/components/language_toggle.html")
        )?;
        
        println!("✅ Registered shared templates");
        Ok(())
    }
    
    /// Register crypto dashboard templates
    pub fn register_crypto_templates(tera: &mut Tera) -> Result<(), anyhow::Error> {
        // Crypto dashboard templates with logical names used across the codebase
        tera.add_template_file(
            "dashboards/crypto_dashboard/routes/reports/view.html",
            Some("crypto/routes/reports/view.html")
        )?;
        
        tera.add_template_file(
            "dashboards/crypto_dashboard/routes/reports/pdf.html",
            Some("crypto/routes/reports/pdf.html")
        )?;
        
        tera.add_template_file(
            "dashboards/crypto_dashboard/routes/reports/list.html",
            Some("crypto/routes/reports/list.html")
        )?;
        
        // Legacy compatibility mappings
        tera.add_template_file(
            "shared_components/theme_toggle.html",
            Some("crypto/components/theme_toggle.html")
        )?;
        
        tera.add_template_file(
            "shared_components/language_toggle.html",
            Some("crypto/components/language_toggle.html")
        )?;
        
        println!("✅ Registered crypto dashboard templates");
        Ok(())
    }
    
    /// Initialize complete template engine with all required templates
    pub async fn initialize_template_engine() -> Result<Tera, anyhow::Error> {
        let mut tera = Tera::new("dashboards/**/*.html")?;
        
        // Disable auto-escaping for safe content (we handle escaping manually)
        tera.autoescape_on(vec![]);
        
        // Register all template collections
        Self::register_shared_templates(&mut tera)?;
        Self::register_crypto_templates(&mut tera)?;
        
        Ok(tera)
    }
}

/// Template context utilities
pub mod context_utils {
    use super::*;
    use chrono::{DateTime, Utc, Duration};
    
    /// Create context for report rendering
    pub fn create_report_context(
        report: &crate::models::Report,
        chart_modules_content: &str,
    ) -> Context {
        let mut context = Context::new();
        context.insert("report", report);
        context.insert("chart_modules_content", chart_modules_content);
        
        // Formatted created date in UTC+7 timezone for display
        let created_display = (report.created_at + Duration::hours(7))
            .format("%d-%m-%Y %H:%M")
            .to_string();
        context.insert("created_at_display", &created_display);
        
        context
    }
    
    /// Create context for report list with pagination
    pub fn create_report_list_context(
        items: Vec<crate::models::ReportListItem>,
        pagination: PaginationData,
    ) -> Context {
        let mut context = Context::new();
        
        let reports = serde_json::json!({
            "items": items,
            "total": pagination.total,
            "per_page": pagination.per_page,
            "page": pagination.page,
            "pages": pagination.pages,
            "has_prev": pagination.page > 1,
            "has_next": pagination.page < pagination.pages,
            "prev_num": if pagination.page > 1 { pagination.page - 1 } else { 1 },
            "next_num": if pagination.page < pagination.pages { pagination.page + 1 } else { pagination.pages },
            "page_numbers": pagination.page_numbers,
            "display_start": pagination.display_start,
            "display_end": pagination.display_end,
        });
        
        context.insert("reports", &reports);
        context
    }
    
    /// Create empty state context when no data available
    pub fn create_empty_context() -> Context {
        let mut context = Context::new();
        context.insert("empty_state", &true);
        context.insert("message", "Không có dữ liệu");
        context
    }
}

/// Pagination data structure
#[derive(Debug, Clone)]
pub struct PaginationData {
    pub total: i64,
    pub per_page: i64,
    pub page: i64,
    pub pages: i64,
    pub page_numbers: Vec<i64>,
    pub display_start: i64,
    pub display_end: i64,
}

impl PaginationData {
    /// Calculate pagination from total count and current page
    pub fn new(total: i64, page: i64, per_page: i64, items_count: usize) -> Self {
        let pages = (total + per_page - 1) / per_page;
        let offset = (page - 1) * per_page;
        
        // Pagination window calculation (show 5 pages around current)
        let start_page = std::cmp::max(1, page - 2);
        let end_page = std::cmp::min(pages, start_page + 4);
        let page_numbers: Vec<i64> = (start_page..=end_page).collect();
        
        let display_start = if total == 0 { 0 } else { offset + 1 };
        let display_end = offset + (items_count as i64);
        
        Self {
            total,
            per_page,
            page,
            pages,
            page_numbers,
            display_start,
            display_end,
        }
    }
}

/// Template rendering utilities with error handling
pub mod rendering {
    use super::*;
    use anyhow::Result;
    
    /// Render template in background thread to avoid blocking async runtime
    pub async fn render_template_async(
        tera: &Tera,
        template_name: &str,
        context: &Context,
    ) -> Result<String> {
        let tera_clone = tera.clone();
        let template_name = template_name.to_string();
        let context_clone = context.clone();
        
        let html = tokio::task::spawn_blocking(move || {
            tera_clone.render(&template_name, &context_clone)
        })
        .await??;
        
        Ok(html)
    }
    
    /// Create HTTP response with caching headers
    pub fn create_cached_response(html: String, cache_status: &str) -> axum::response::Response {
        use axum::response::Response;
        use axum::http::StatusCode;
        
        Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "text/html; charset=utf-8")
            .header("cache-control", "public, max-age=300")
            .header("x-cache-status", cache_status)
            .body(html)
            .unwrap()
    }
}
