//! Template Renderer - Dashboard template rendering engine
//!
//! Handles template rendering for dashboard views, PDF generation,
//! and report visualization using Tera templating engine.

use crate::features::shared_components::{SharedComponents, templates::TemplateRegistry};
use crate::features::cache_system::CacheSystem;
use crate::models::Report;
use serde_json::json;
use std::collections::HashMap;
use std::error::Error as StdError;
use std::sync::Arc;
use tera::Context;
use tokio::fs;

/// Template rendering engine for dashboard views
pub struct TemplateRenderer {
    template_registry: Arc<TemplateRegistry>,
    cache_system: Option<Arc<CacheSystem>>,
}

impl TemplateRenderer {
    pub fn new(shared_components: &SharedComponents, cache_system: &CacheSystem) -> Self {
        Self {
            template_registry: shared_components.get_template_registry(),
            cache_system: Some(Arc::new(cache_system.clone())),
        }
    }

    /// Render crypto dashboard view with report data
    pub async fn render_dashboard_view(&self, report_id: Option<i32>) -> Result<String, Box<dyn StdError + Send + Sync>> {
        // Get report from cache or database
        let report = if let Some(id) = report_id {
            self.get_report_by_id(id).await?
        } else {
            self.get_latest_report().await?
        };

        // Get chart modules content
        let chart_modules_content = self.get_chart_modules_content().await;

        // Render using helper function
        self.render_crypto_template(
            "crypto/routes/reports/view.html",
            &report,
            &chart_modules_content,
            None
        ).await
    }

    /// Render PDF template for report
    pub async fn render_pdf_template(&self, report_id: i32) -> Result<String, Box<dyn StdError + Send + Sync>> {
        let report = self.get_report_by_id(report_id).await?;
        let chart_modules_content = self.get_chart_modules_content().await;

        // PDF template specific context
        let mut additional_context = HashMap::new();
        let created_display = (report.created_at + chrono::Duration::hours(7))
            .format("%d-%m-%Y %H:%M")
            .to_string();
        additional_context.insert("created_at_display".to_string(), json!(created_display));

        self.render_crypto_template(
            "crypto/routes/reports/pdf.html",
            &report,
            &chart_modules_content,
            Some(additional_context)
        ).await
    }

    /// Core template rendering function
    pub async fn render_crypto_template(
        &self,
        template: &str,
        report: &Report,
        chart_modules_content: &str,
        additional_context: Option<HashMap<String, serde_json::Value>>
    ) -> Result<String, Box<dyn StdError + Send + Sync>> {
        let tera = self.template_registry.get_tera();
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

    /// Get report by ID from cache or database
    async fn get_report_by_id(&self, id: i32) -> Result<Report, Box<dyn StdError + Send + Sync>> {
        // TODO: Integrate with cache system and database
        // For now, return a default report
        Ok(Report::default_with_id(id))
    }

    /// Get latest report from cache or database
    async fn get_latest_report(&self) -> Result<Report, Box<dyn StdError + Send + Sync>> {
        // TODO: Integrate with cache system and database
        // For now, return a default report
        Ok(Report::default())
    }

    /// Get chart modules JavaScript content
    async fn get_chart_modules_content(&self) -> String {
        // TODO: Integrate with shared_components chart modules
        // For now, return empty content
        String::new()
    }
}

impl Default for TemplateRenderer {
    fn default() -> Self {
        Self {
            template_registry: Arc::new(TemplateRegistry::new()),
            cache_system: None,
        }
    }
}

/// Create cached HTTP response with appropriate headers
pub fn create_cached_response(html: String, cache_status: &str) -> axum::response::Response {
    use axum::{http::StatusCode, response::IntoResponse};
    
    axum::response::Response::builder()
        .status(StatusCode::OK)
        .header("cache-control", "public, max-age=15")
        .header("content-type", "text/html; charset=utf-8")
        .header("x-cache", cache_status)
        .body(html)
        .unwrap()
        .into_response()
}
