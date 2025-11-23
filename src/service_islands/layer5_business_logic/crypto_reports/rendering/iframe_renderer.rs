//! Iframe Renderer
//!
//! This module handles iframe-based rendering for crypto reports.
//! This is the legacy approach that will be replaced by Shadow DOM rendering.
//!
//! Features:
//! - Secure iframe sandboxing with token-based authentication
//! - Multi-language support (Vietnamese and English)
//! - Pre-loaded templates for optimal performance
//! - Content sanitization for security

use std::sync::Arc;
use axum::response::Response;
use lazy_static::lazy_static;
use tracing::{info, warn, debug};

use crate::service_islands::layer1_infrastructure::AppState;
use crate::service_islands::layer5_business_logic::shared::{
    generate_sandbox_token,
    verify_sandbox_token,
    build_sandboxed_response,
    build_forbidden_response,
    Layer5Result,
};
use super::shared::{Report, SandboxedReport, sanitize_html_content, sanitize_css_content, sanitize_js_content};

lazy_static! {
    /// Pre-loaded iframe template to avoid file I/O on every request
    /// Eliminates thousands of syscalls/second at high RPS
    static ref VIEW_IFRAME_TEMPLATE: String = {
        // Try to load from file first, fall back to compile-time include
        std::fs::read_to_string("shared_components/view_iframe.html")
            .unwrap_or_else(|_| {
                // Compile-time fallback ensures template is always available
                include_str!("../../../../../shared_components/view_iframe.html").to_string()
            })
    };
}

/// Iframe Renderer
///
/// Handles all iframe-based rendering logic for crypto reports
#[derive(Clone)]
pub struct IframeRenderer;

impl IframeRenderer {
    /// Create a new iframe renderer
    pub fn new() -> Self {
        Self
    }

    /// Generate sandboxed report content
    ///
    /// Creates a secure sandboxed version of the report for iframe delivery.
    /// This method sanitizes content, generates a security token, and creates the complete HTML document.
    ///
    /// # Security
    /// Uses cryptographically secure blake3 hash for token generation.
    /// Tokens cannot be predicted or forged.
    pub fn create_sandboxed_report(&self, report: &Report, chart_modules_content: Option<&str>) -> SandboxedReport {
        // Generate cryptographically secure token
        let sandbox_token = generate_sandbox_token(report.id, &report.created_at);

        info!("IframeRenderer: Generated secure sandbox token for report {}: {}", report.id, sandbox_token);

        // Create sandboxed report with sanitized content
        // Using .into_owned() on Cow only allocates when content was actually modified
        let mut sandboxed_report = SandboxedReport {
            id: report.id,
            html_content: sanitize_html_content(&report.html_content).into_owned(),
            css_content: report.css_content.as_deref().map(sanitize_css_content),
            js_content: report.js_content.as_deref().map(|js| sanitize_js_content(js).into_owned()),
            html_content_en: report.html_content_en.as_deref().map(|html| sanitize_html_content(html).into_owned()),
            js_content_en: report.js_content_en.as_deref().map(|js| sanitize_js_content(js).into_owned()),
            created_at: report.created_at,
            sandbox_token,
            chart_modules_content: chart_modules_content.map(str::to_owned),
            complete_html_document: String::new(), // Will be populated below
        };

        // Generate complete HTML document and store it
        sandboxed_report.complete_html_document = self.generate_sandboxed_html_document(&sandboxed_report, None, chart_modules_content);

        debug!("📄 IframeRenderer: Complete HTML document generated for report {} ({} bytes)",
                report.id, sandboxed_report.complete_html_document.len());

        sandboxed_report
    }

    /// Regenerate HTML document for a specific language if needed
    ///
    /// This method allows generating a new HTML document with a specific language
    /// without recreating the entire SandboxedReport.
    pub fn regenerate_html_document(&self, sandboxed_report: &SandboxedReport, language: Option<&str>) -> String {
        let html_doc = self.generate_sandboxed_html_document(sandboxed_report, language,
            sandboxed_report.chart_modules_content.as_deref());

        info!("🔄 IframeRenderer: Regenerated HTML document for report {} with language {:?} ({} bytes)",
                sandboxed_report.id, language.unwrap_or("vi"), html_doc.len());

        html_doc
    }

    /// Generate complete sandboxed HTML document
    ///
    /// Creates a self-contained HTML document for iframe embedding with isolated CSS.
    /// Now includes both languages, dynamic switching capability, and chart modules.
    /// Uses external HTML template file for better maintainability.
    ///
    /// # Performance
    /// Uses as_deref() pattern for efficient Option<String> handling.
    pub fn generate_sandboxed_html_document(&self, sandboxed_report: &SandboxedReport, language: Option<&str>, chart_modules_content: Option<&str>) -> String {
        let default_lang = language.unwrap_or("vi");

        // Use as_deref() pattern - zero allocations for defaults
        let default_html_vi = &sandboxed_report.html_content;
        let default_html_en = sandboxed_report.html_content_en.as_deref().unwrap_or(default_html_vi);
        let default_js_vi = sandboxed_report.js_content.as_deref().unwrap_or("");
        let default_js_en = sandboxed_report.js_content_en.as_deref().unwrap_or(default_js_vi);
        let default_css = sandboxed_report.css_content.as_deref().unwrap_or("");

        // Use chart modules from SandboxedReport if available, otherwise use parameter
        let chart_modules = sandboxed_report.chart_modules_content
            .as_deref()
            .or(chart_modules_content)
            .unwrap_or("");

        // Determine active classes based on default language
        let (vi_active_class, en_active_class) = if default_lang == "en" {
            ("", "active")
        } else {
            ("active", "")
        };

        // ✅ PERFORMANCE OPTIMIZATION: Use pre-loaded template (zero file I/O)
        // At high RPS, this eliminates thousands of syscalls/second
        let template_content = &*VIEW_IFRAME_TEMPLATE;

        // Replace template variables
        template_content
            .replace("{{default_lang}}", default_lang)
            .replace("{{report_id}}", &sandboxed_report.id.to_string())
            .replace("{{vi_active_class}}", vi_active_class)
            .replace("{{en_active_class}}", en_active_class)
            .replace("{{css_content}}", default_css)
            .replace("{{html_content_vi}}", default_html_vi)
            .replace("{{html_content_en}}", default_html_en)
            .replace("{{chart_modules}}", chart_modules)
            .replace("{{js_content_vi}}", default_js_vi)
            .replace("{{js_content_en}}", default_js_en)
    }

    /// Serve sandboxed report content for iframe
    ///
    /// Returns sanitized HTML content for secure iframe embedding.
    /// This method handles report fetching, token verification, and response generation.
    ///
    /// # Security
    /// Uses constant-time token comparison to prevent timing attacks.
    pub async fn serve_sandboxed_report(
        &self,
        _state: &Arc<AppState>,
        report: &Report,
        sandbox_token: &str,
        language: Option<&str>,
        chart_modules_content: Option<&str>
    ) -> Layer5Result<Response> {
        info!("IframeRenderer: Serving sandboxed content for report {} with token {}", report.id, sandbox_token);

        // Verify sandbox token using constant-time comparison
        if !verify_sandbox_token(sandbox_token, report.id, &report.created_at) {
            warn!("IframeRenderer: Invalid sandbox token for report {}", report.id);
            return Ok(build_forbidden_response("Invalid sandbox token"));
        }

        // Create sandboxed version with complete HTML document
        let sandboxed_report = self.create_sandboxed_report(report, chart_modules_content);

        // Use the pre-generated complete HTML document
        // If a specific language is requested and it's different from default (vi), regenerate
        let sandboxed_html = match language {
            Some(lang) if lang != "vi" => {
                // Regenerate with specific language
                self.regenerate_html_document(&sandboxed_report, Some(lang))
            }
            _ => {
                // Move ownership instead of cloning
                sandboxed_report.complete_html_document
            }
        };

        info!("IframeRenderer: Serving HTML document for report {} with language {:?} ({} bytes)",
                report.id, language.unwrap_or("vi"), sandboxed_html.len());

        // Return response with security headers using safe builder
        Ok(build_sandboxed_response(sandboxed_html))
    }
}

impl Default for IframeRenderer {
    fn default() -> Self {
        Self::new()
    }
}
