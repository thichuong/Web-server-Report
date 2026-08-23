//! Template Orchestrator Component
//!
//! This component handles template rendering operations for crypto reports,
//! including context preparation and Tera integration.
//! Follows Service Islands Architecture Layer 5 patterns.

use std::{collections::HashMap, sync::Arc};
use tera::Context;
use tracing::{error, info, warn};

// Import from our specialized components
use super::report_creator::{Report, ReportCreator};

// Import shared utilities
use super::super::shared::{Layer5Error, Layer5Result, get_websocket_url};

/// Template Context Data
///
/// Structured container for all template rendering context data
/// Optimized to use Arc for heavy data to avoid expensive clones
#[derive(Debug, Clone)]
pub struct TemplateContext {
    pub report: Arc<Report>,
    pub current_route: String,
    pub current_lang: String,
    pub current_time: String,
    pub pdf_url: String,
    pub additional_context: Option<HashMap<String, serde_json::Value>>,
}

/// Template Orchestrator
///
/// Manages template rendering operations for crypto reports.
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
    #[must_use]
    pub fn health_check(&self) -> bool {
        self.report_creator.health_check()
    }

    /// Prepare template context for crypto reports
    ///
    /// Builds complete template context with all necessary data for rendering.
    ///
    /// # Memory Optimization
    /// Takes Report by value (move) and wraps in Arc without cloning.
    ///
    /// # Errors
    ///
    /// Returns error if context preparation fails
    pub fn prepare_crypto_report_context(
        &self,
        report: Report,
        template_type: &str,
        additional_context: Option<HashMap<String, serde_json::Value>>,
    ) -> Layer5Result<TemplateContext> {
        info!(
            "TemplateOrchestrator: Preparing context for template type: {}",
            template_type
        );

        let sandboxed_report = self.report_creator.create_sandboxed_report(&report);

        // Prepare basic context
        let current_time = chrono::Utc::now()
            .format("%Y-%m-%d %H:%M:%S UTC")
            .to_string();
        let pdf_url = format!("/crypto_report/{}/pdf", report.id);

        let mut context = TemplateContext {
            report: Arc::new(report),
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

        info!("TemplateOrchestrator: Context prepared successfully");
        Ok(context)
    }

    /// Render crypto template with prepared context
    ///
    /// Core template rendering method using Tera engine with proper error handling.
    ///
    /// # Performance
    /// `TemplateContext` uses Arc internally, so clone is lightweight (only pointers cloned).
    ///
    /// # Errors
    ///
    /// Returns error if template rendering fails
    pub fn render_template(
        &self,
        tera: &tera::Tera,
        template_path: &str,
        context: TemplateContext,
    ) -> Layer5Result<String> {
        info!(
            "TemplateOrchestrator: Rendering template: {}",
            template_path
        );

        let mut tera_context = Context::new();

        // Insert core template data (dereference Arc)
        tera_context.insert("report", context.report.as_ref());
        tera_context.insert("current_route", &context.current_route);
        tera_context.insert("current_lang", &context.current_lang);
        tera_context.insert("current_time", &context.current_time);
        tera_context.insert("pdf_url", &context.pdf_url);

        // Add additional context if provided
        if let Some(extra) = context.additional_context {
            for (key, value) in extra {
                tera_context.insert(&key, &value);
            }
        }

        // Template-specific context adjustments
        if template_path.contains("pdf.html") {
            let created_display = (context.report.created_at + chrono::Duration::hours(7))
                .format("%d-%m-%Y %H:%M")
                .to_string();
            tera_context.insert("created_at_display", &created_display);
        }

        // Render template synchronously
        match tera.render(template_path, &tera_context) {
            Ok(html) => {
                info!("TemplateOrchestrator: Template rendered successfully");
                Ok(html)
            }
            Err(e) => {
                error!("TemplateOrchestrator: Template render error: {:#?}", e);
                let mut src = std::error::Error::source(&e);
                while let Some(s) = src {
                    error!("Template render error source: {:#?}", s);
                    src = std::error::Error::source(s);
                }
                Err(Layer5Error::TemplateRender(e.to_string()))
            }
        }
    }

    /// Render empty template for no reports case
    ///
    /// Handles the case when no reports are found in database.
    ///
    /// # Errors
    ///
    /// Returns error if context preparation or template rendering fails
    pub fn render_empty_template(&self, tera: &tera::Tera) -> Layer5Result<String> {
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

        let mut context = self.prepare_crypto_report_context(empty_report, "empty", None)?;
        context.pdf_url = "#".to_string();
        self.render_template(tera, "crypto/routes/reports/view.html", context)
    }

    /// Render 404 not found template
    ///
    /// Handles the case when a specific report ID is not found.
    ///
    /// # Errors
    ///
    /// Returns error if context preparation or template rendering fails
    #[allow(dead_code)]
    pub fn render_not_found_template(
        &self,
        tera: &tera::Tera,
        report_id: i32,
    ) -> Layer5Result<String> {
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

        let mut context =
            self.prepare_crypto_report_context(not_found_report, "404", None)?;
        context.pdf_url = "#".to_string();
        self.render_template(tera, "crypto/routes/reports/view.html", context)
    }
}

impl Default for TemplateOrchestrator {
    fn default() -> Self {
        Self::new(ReportCreator::default())
    }
}
