//! Crypto Reports HTTP Request Handlers
//! 
//! This module contains all HTTP request handlers related to crypto reports functionality.
//! Based on archive_old_code/handlers/crypto.rs
//! ONLY uses Template Engine - NO manual HTML creation

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Serialize, Deserialize};
use std::{collections::HashMap, error::Error as StdError, sync::Arc, sync::atomic::Ordering};
use tera::Context;

// Import from current state - will be refactored when lower layers are implemented
use crate::state::AppState;

/// Report model - exactly from archive_old_code/models.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub id: i32,
    pub html_content: String,
    pub css_content: Option<String>,
    pub js_content: Option<String>,
    pub html_content_en: Option<String>,
    pub js_content_en: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Crypto Handlers
/// 
/// Contains all HTTP request handlers for crypto reports-related operations.
/// These handlers manage crypto report generation, PDF creation, and API interactions.
/// ONLY uses Template Engine like archive_old_code/handlers/crypto.rs
pub struct CryptoHandlers {
    // Component state will be added here as we implement lower layers
}

impl CryptoHandlers {
    /// Create a new CryptoHandlers instance
    pub fn new() -> Self {
        Self {
            // Initialize component state
        }
    }
    
    /// Health check for crypto handlers
    pub async fn health_check(&self) -> bool {
        // Verify handlers are functioning properly
        true // Will implement actual health checks
    }

    /// Helper function for template rendering
    /// 
    /// Directly from archive_old_code/handlers/crypto.rs::render_crypto_template
    /// ONLY Template Engine - NO manual HTML
    pub async fn render_crypto_template(
        &self,
        tera: &tera::Tera, 
        template: &str,
        report: &Report,
        chart_modules_content: &str,
        additional_context: Option<HashMap<String, serde_json::Value>>
    ) -> Result<String, Box<dyn StdError + Send + Sync>> {
        let tera_clone = tera.clone();
        let template_str = template.to_string();
        let report_clone = report.clone();
        let chart_content_clone = chart_modules_content.to_string();
        let additional_clone = additional_context.clone();
        
        let render_result = tokio::task::spawn_blocking(move || {
            let mut context = Context::new();
            context.insert("report", &report_clone);
            context.insert("chart_modules_content", &chart_content_clone);
            
            // Add additional context for different templates
            if let Some(extra) = additional_clone {
                for (key, value) in extra {
                    context.insert(&key, &value);
                }
            }
            
            // Common context for view templates
            if template_str.contains("view.html") {
                context.insert("current_route", "dashboard");
                context.insert("current_lang", "vi");
                context.insert("current_time", &chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string());
                let pdf_url = format!("/pdf-template/{}", report_clone.id);
                context.insert("pdf_url", &pdf_url);
            }
            
            // PDF template specific context
            if template_str.contains("pdf.html") {
                let created_display = (report_clone.created_at + chrono::Duration::hours(7)).format("%d-%m-%Y %H:%M").to_string();
                context.insert("created_at_display", &created_display);
            }

            // ONLY use Tera template engine - NO manual HTML
            tera_clone.render(&template_str, &context)
        }).await;
        
        match render_result {
            Ok(Ok(html)) => Ok(html),
            Ok(Err(e)) => {
                eprintln!("Template render error: {:#?}", e);
                let mut src = e.source();
                while let Some(s) = src {
                    eprintln!("Template render error source: {:#?}", s);
                    src = s.source();
                }
                Err(format!("Template render error: {}", e).into())
            }
            Err(e) => {
                eprintln!("Task join error: {:#?}", e);
                Err(format!("Task join error: {}", e).into())
            }
        }
    }

    /// Create cached response
    /// 
    /// From archive_old_code/handlers/crypto.rs::create_cached_response
    pub fn create_cached_response(&self, html: String, cache_status: &str) -> Response {
        Response::builder()
            .status(StatusCode::OK)
            .header("cache-control", "public, max-age=15")
            .header("content-type", "text/html; charset=utf-8")
            .header("x-cache", cache_status)
            .body(html)
            .unwrap()
            .into_response()
    }

    /// Fetch and cache latest report 
    /// 
    /// From archive_old_code/handlers/crypto.rs::fetch_and_cache_latest_report
    /// Data from database ONLY - NO HTML creation here
    pub async fn fetch_and_cache_latest_report(
        &self,
        state: &Arc<AppState>
    ) -> Result<Option<Report>, Box<dyn StdError + Send + Sync>> {
        // TODO: Implement real database query like archive_old_code
        // let report = sqlx::query_as::<_, Report>(
        //     "SELECT id, html_content, css_content, js_content, html_content_en, js_content_en, created_at FROM crypto_report ORDER BY created_at DESC LIMIT 1",
        // ).fetch_optional(&state.db).await?;
        
        // Mock data from database ONLY - content will be injected into template
        let mock_report = Report {
            id: 1,
            html_content: "<!-- HTML content from database -->".to_string(),
            css_content: Some("/* CSS content from database */".to_string()),
            js_content: Some("// JS content from database".to_string()),
            html_content_en: Some("<!-- English HTML from database -->".to_string()),
            js_content_en: Some("// English JS from database".to_string()),
            created_at: chrono::Utc::now(),
        };
        
        Ok(Some(mock_report))
    }

    /// Get chart modules content
    /// 
    /// Based on archive_old_code/utils.rs::get_chart_modules_content
    pub async fn get_chart_modules_content(&self) -> String {
        // TODO: Read from shared_assets/js/chart_modules/ like archive code
        "console.log('Chart modules loaded');".to_string()
    }
    
    /// Crypto Index handler - Main crypto dashboard
    /// 
    /// Originally from archive_old_code/handlers/crypto.rs::crypto_index
    /// ONLY uses Tera template engine - NO manual HTML creation
    pub async fn crypto_index(&self) -> Result<String, Box<dyn StdError + Send + Sync>> {
        println!("🚀 CryptoHandlers::crypto_index - Template Engine ONLY");
        Err("Need Tera engine access from Service Islands AppState".into())
    }
    
    /// Crypto Index with Tera template engine - CORRECT implementation
    /// 
    /// Exactly like archive_old_code/handlers/crypto.rs - ONLY Template Engine
    pub async fn crypto_index_with_tera(&self, state: &Arc<AppState>) -> Result<String, Box<dyn StdError + Send + Sync>> {
        println!("🚀 CryptoHandlers::crypto_index_with_tera - Template Engine ONLY");
        
        // Step 1: Fetch report data (from database)
        let report = self.fetch_and_cache_latest_report(state).await?
            .ok_or("No report found")?;
        
        // Step 2: Get chart modules content  
        let chart_modules_content = self.get_chart_modules_content().await;
        
        // Step 3: ONLY use Tera template engine - NO manual HTML
        let html = self.render_crypto_template(
            &state.tera,
            "crypto/routes/reports/view.html", // Template from dashboards/crypto_dashboard/routes/reports/view.html
            &report,
            &chart_modules_content,
            None
        ).await?;
        
        println!("✅ Template rendered via Tera engine ONLY");
        Ok(html)
    }

    // NOTE: Crypto handlers implementation following archive_old_code/handlers/crypto.rs
    // Key requirements:
    // 1. MUST use Tera template engine - NO manual HTML creation
    // 2. MUST use "crypto/routes/reports/view.html" template 
    // 3. Template variables: {{ report.css_content }}, {{ report.js_content }}, {{ chart_modules_content }}
    // 4. Implement L1/L2 caching logic like original
    // 5. Parallel chart modules fetching
    // 
    // Current status: Template engine access needed from Service Islands architecture

}
