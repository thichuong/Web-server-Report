//! Template Orchestrator Component
//!
//! This component handles all template rendering operations for crypto reports,
//! including context preparation, chart modules injection, and Tera integration.
//! Follows Service Islands Architecture Layer 5 patterns.

use std::{collections::HashMap, sync::Arc};
use tera::Context;
use tracing::{debug, error, info, warn};

// Import from our specialized components
use super::report_creator::{Report, ReportCreator};

// Import shared utilities
use super::super::shared::{compress_html_to_gzip, get_websocket_url, Layer5Error, Layer5Result};

/// Template Context Data
///
/// Structured container for all template rendering context data
/// Optimized to use Arc for heavy data to avoid expensive clones
#[derive(Debug, Clone)]
pub struct TemplateContext {
    pub report: Arc<Report>, // ✅ Use Arc to avoid cloning full Report data
    pub chart_modules_content: Arc<String>, // ✅ Use Arc to avoid string clones
    pub current_route: String,
    pub current_lang: String,
    pub current_time: String,
    pub pdf_url: String,
    pub additional_context: Option<HashMap<String, serde_json::Value>>,
}

/// Template Orchestrator
///
/// Manages all template rendering operations for crypto reports.
/// Separates template logic from HTTP handlers following Layer 5 architecture.
pub struct TemplateOrchestrator {
    /// Reference to `ReportCreator` for data operations
    pub report_creator: ReportCreator,
}

impl TemplateOrchestrator {
    /// Create a new `TemplateOrchestrator`
    #[must_use] 
    pub fn new(report_creator: ReportCreator) -> Self {
        Self { report_creator }
    }

    /// Health check for template orchestrator
    pub async fn health_check(&self) -> bool {
        // Verify template orchestrator is functioning properly
        self.report_creator.health_check().await
    }

    /// Compress HTML content using shared compression utility
    ///
    /// # Errors
    ///
    /// Returns error if gzip compression fails
    #[inline]
    fn compress_html(&self, html: &str) -> Layer5Result<Vec<u8>> {
        let (data, stats) = compress_html_to_gzip(html)?;
        info!(
            "TemplateOrchestrator: Compression completed - Original: {}KB, Compressed: {}KB, Ratio: {:.1}%",
            stats.original_kb(),
            stats.compressed_kb(),
            stats.ratio_percent
        );
        Ok(data)
    }

    /// Prepare template context for crypto reports
    ///
    /// Builds complete template context with all necessary data for rendering.
    /// Enhanced to accept pre-loaded `chart_modules_content` for optimal performance.
    /// Now includes sandbox token generation for iframe security.
    ///
    /// # Memory Optimization
    /// Takes Report by value (move) and wraps in Arc without cloning.
    ///
    /// # Errors
    ///
    /// Returns error if chart modules content retrieval fails
    pub async fn prepare_crypto_report_context(
        &self,
        report: Report,
        template_type: &str,
        chart_modules_content: Option<Arc<String>>,
        additional_context: Option<HashMap<String, serde_json::Value>>,
    ) -> Layer5Result<TemplateContext> {
        info!(
            "TemplateOrchestrator: Preparing context for template type: {}",
            template_type
        );

        // Use provided chart_modules_content or fetch from ReportCreator
        let chart_modules_content = if let Some(content) = chart_modules_content {
            debug!("TemplateOrchestrator: Using pre-loaded chart modules (Arc - zero clone)");
            content
        } else {
            info!("TemplateOrchestrator: Fallback - reading chart modules from file");
            Arc::new(self.report_creator.get_chart_modules_content().await)
        };

        // Generate sandbox token for iframe security
        let sandboxed_report = self
            .report_creator
            .create_sandboxed_report(&report, Some(&chart_modules_content));

        // Prepare basic context
        let current_time = chrono::Utc::now()
            .format("%Y-%m-%d %H:%M:%S UTC")
            .to_string();
        let pdf_url = format!("/crypto_report/{}/pdf", report.id);

        let mut context = TemplateContext {
            report: Arc::new(report),
            chart_modules_content,
            current_route: "dashboard".to_string(),
            current_lang: "vi".to_string(),
            current_time,
            pdf_url,
            additional_context,
        };

        // Add sandbox token and WebSocket URL to additional context
        let mut extra_context = context.additional_context.take().unwrap_or_default();
        extra_context.insert(
            "sandbox_token".to_string(),
            serde_json::Value::String(sandboxed_report.sandbox_token),
        );
        extra_context.insert(
            "websocket_url".to_string(),
            serde_json::Value::String(get_websocket_url()),
        );

        context.additional_context = Some(extra_context);

        info!("TemplateOrchestrator: Context prepared successfully with sandbox token");
        Ok(context)
    }

    /// Render crypto template with prepared context
    ///
    /// Core template rendering method using Tera engine with proper error handling.
    /// Uses `spawn_blocking` with timeout to prevent hanging on CPU-intensive renders.
    ///
    /// # Performance
    /// `TemplateContext` uses Arc internally, so clone is lightweight (only pointers cloned).
    ///
    /// # Errors
    ///
    /// Returns error if template rendering fails, task join fails, or operation times out
    pub async fn render_template(
        &self,
        tera: &tera::Tera,
        template_path: &str,
        context: TemplateContext,
    ) -> Layer5Result<String> {
        info!(
            "TemplateOrchestrator: Rendering template: {}",
            template_path
        );

        // Clone for spawn_blocking - lightweight due to Arc usage
        let tera_clone = tera.clone();
        let template_str = template_path.to_string();
        let context_clone = context.clone();

        // Run CPU-intensive template rendering in blocking thread with timeout
        let render_task = tokio::task::spawn_blocking(move || {
            let mut tera_context = Context::new();

            // Insert core template data (dereference Arc)
            tera_context.insert("report", context_clone.report.as_ref());
            tera_context.insert(
                "chart_modules_content",
                context_clone.chart_modules_content.as_ref(),
            );
            tera_context.insert("current_route", &context_clone.current_route);
            tera_context.insert("current_lang", &context_clone.current_lang);
            tera_context.insert("current_time", &context_clone.current_time);
            tera_context.insert("pdf_url", &context_clone.pdf_url);

            // Add additional context if provided
            if let Some(extra) = context_clone.additional_context {
                for (key, value) in extra {
                    tera_context.insert(&key, &value);
                }
            }

            // Template-specific context adjustments
            if template_str.contains("pdf.html") {
                let created_display = (context_clone.report.created_at
                    + chrono::Duration::hours(7))
                .format("%d-%m-%Y %H:%M")
                .to_string();
                tera_context.insert("created_at_display", &created_display);
            }

            tera_clone.render(&template_str, &tera_context)
        });

        // Flatten the nested Result with helper function
        self.await_render_task(render_task).await
    }

    /// Helper to flatten nested Result from `spawn_blocking` + timeout
    ///
    /// Converts `Result<Result<Result<T, E1>, E2>, E3>` into `Layer5Result<T>`
    #[inline]
    async fn await_render_task(
        &self,
        task: tokio::task::JoinHandle<Result<String, tera::Error>>,
    ) -> Layer5Result<String> {
        const RENDER_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);

        match tokio::time::timeout(RENDER_TIMEOUT, task).await {
            Ok(Ok(Ok(html))) => {
                info!("TemplateOrchestrator: Template rendered successfully");
                Ok(html)
            }
            Ok(Ok(Err(e))) => {
                error!("TemplateOrchestrator: Template render error: {:#?}", e);
                // Log error chain
                let mut src = std::error::Error::source(&e);
                while let Some(s) = src {
                    error!("Template render error source: {:#?}", s);
                    src = std::error::Error::source(s);
                }
                Err(Layer5Error::TemplateRender(e.to_string()))
            }
            Ok(Err(e)) => {
                error!("TemplateOrchestrator: Task join error: {:#?}", e);
                Err(Layer5Error::TaskJoin(e.to_string()))
            }
            Err(_) => {
                error!("TemplateOrchestrator: Template rendering timeout after 30s");
                Err(Layer5Error::Timeout(
                    "Template rendering took longer than 30 seconds".to_string(),
                ))
            }
        }
    }

    /// Render crypto report view template - High level method
    ///
    /// Combines context preparation and template rendering for crypto report views.
    /// Returns compressed HTML data for optimal file size and transfer speed.
    ///
    /// # Memory Optimization
    /// Takes Report by value (move) to avoid cloning.
    ///
    /// # Errors
    ///
    /// Returns error if context preparation, template rendering, or HTML compression fails
    pub async fn render_crypto_report_view(
        &self,
        tera: &tera::Tera,
        report: Report,
        chart_modules_content: Option<Arc<String>>,
        additional_context: Option<HashMap<String, serde_json::Value>>,
    ) -> Layer5Result<Vec<u8>> {
        debug!("TemplateOrchestrator: Rendering crypto report view with compression");

        // Step 1: Prepare template context (moves report ownership)
        let context = self
            .prepare_crypto_report_context(
                report,
                "view",
                chart_modules_content,
                additional_context,
            )
            .await?;

        // Step 2: Render template
        let html = self
            .render_template(tera, "crypto/routes/reports/view.html", context)
            .await?;

        // Step 3: Compress HTML
        let compressed_data = self.compress_html(&html)?;

        info!("TemplateOrchestrator: HTML compression completed successfully");
        Ok(compressed_data)
    }

    /// Render empty template for no reports case
    ///
    /// Handles the case when no reports are found in database.
    ///
    /// # Errors
    ///
    /// Returns error if context preparation or template rendering fails
    pub async fn render_empty_template(&self, tera: &tera::Tera) -> Layer5Result<String> {
        warn!("TemplateOrchestrator: Rendering empty template");

        // Create empty report for template
        let empty_report = Report {
            id: 0,
            html_content: String::new(),
            css_content: None,
            js_content: None,
            html_content_en: None,
            js_content_en: None,
            created_at: chrono::Utc::now(),
        };

        // Prepare context
        let mut context = self
            .prepare_crypto_report_context(empty_report, "empty", None, None)
            .await?;

        // Override PDF URL for empty case
        context.pdf_url = "#".to_string();

        // Render template
        self.render_template(tera, "crypto/routes/reports/view.html", context)
            .await
    }

    /// Render 404 not found template
    ///
    /// Handles the case when a specific report ID is not found.
    ///
    /// # Errors
    ///
    /// Returns error if context preparation or template rendering fails
    #[allow(dead_code)]
    pub async fn render_not_found_template(
        &self,
        tera: &tera::Tera,
        report_id: i32,
    ) -> Layer5Result<String> {
        debug!(
            "TemplateOrchestrator: Rendering 404 template for report ID: {}",
            report_id
        );

        // Create not found report with error messages
        let not_found_report = Report {
            id: report_id,
            html_content: format!(
                "<div class='text-center py-16'>\
                <h2 class='text-2xl font-bold text-red-600'>Report #{report_id} not found</h2>\
                <p class='text-gray-500 mt-4'>This report may have been deleted or you don't have access.</p>\
                <a href='/crypto_reports_list' class='mt-6 inline-block px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700'>Back to reports list</a>\
                </div>"
            ),
            css_content: None,
            js_content: None,
            html_content_en: None,
            js_content_en: None,
            created_at: chrono::Utc::now(),
        };

        // Prepare context
        let mut context = self
            .prepare_crypto_report_context(not_found_report, "404", None, None)
            .await?;

        // Override PDF URL for 404 case
        context.pdf_url = "#".to_string();

        // Render template
        self.render_template(tera, "crypto/routes/reports/view.html", context)
            .await
    }
}

impl Default for TemplateOrchestrator {
    fn default() -> Self {
        Self::new(ReportCreator::default())
    }
}
