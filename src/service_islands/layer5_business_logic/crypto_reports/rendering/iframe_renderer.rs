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

use std::{sync::Arc, error::Error as StdError};
use axum::{
    response::{Response, IntoResponse},
    http::StatusCode,
    body::Body
};
use lazy_static::lazy_static;
use tracing::{info, warn, error, debug};

use crate::service_islands::layer1_infrastructure::AppState;
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
    /// ✅ OPTIMIZED: Minimizes unnecessary clones and string allocations
    pub fn create_sandboxed_report(&self, report: &Report, chart_modules_content: Option<&str>) -> SandboxedReport {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        // Generate security token based on report ID and creation time
        let mut hasher = DefaultHasher::new();
        report.id.hash(&mut hasher);
        report.created_at.hash(&mut hasher);
        let sandbox_token = format!("sb_{:x}", hasher.finish());

        info!("🔒 IframeRenderer: Generated sandbox token for report {}: {}", report.id, sandbox_token);

        // Create sandboxed report with sanitized content
        let mut sandboxed_report = SandboxedReport {
            id: report.id,
            html_content: sanitize_html_content(&report.html_content),
            css_content: report.css_content.as_ref().map(|css| sanitize_css_content(css)),
            js_content: report.js_content.as_ref().map(|js| sanitize_js_content(js)),
            html_content_en: report.html_content_en.as_ref().map(|html| sanitize_html_content(html)),
            js_content_en: report.js_content_en.as_ref().map(|js| sanitize_js_content(js)),
            created_at: report.created_at,
            sandbox_token,  // ✅ Move instead of clone - no unnecessary allocation
            chart_modules_content: chart_modules_content.map(str::to_owned),  // ✅ Necessary conversion from &str to String
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
    /// Creates a self-contained HTML document for iframe embedding with isolated CSS
    /// Now includes both languages, dynamic switching capability, and chart modules
    /// Uses external HTML template file for better maintainability
    pub fn generate_sandboxed_html_document(&self, sandboxed_report: &SandboxedReport, language: Option<&str>, chart_modules_content: Option<&str>) -> String {
        let default_lang = language.unwrap_or("vi");

        // Create owned strings to avoid borrow checker issues
        let empty_string = String::new();
        let default_html_vi = &sandboxed_report.html_content;
        let default_html_en = sandboxed_report.html_content_en.as_ref().unwrap_or(default_html_vi);
        let default_js_vi = sandboxed_report.js_content.as_ref().unwrap_or(&empty_string);
        let default_js_en = sandboxed_report.js_content_en.as_ref().unwrap_or(default_js_vi);
        let default_css = sandboxed_report.css_content.as_ref().unwrap_or(&empty_string);

        // Use chart modules from SandboxedReport if available, otherwise use parameter, otherwise empty
        let chart_modules = sandboxed_report.chart_modules_content
            .as_ref()
            .map(|s| s.as_str())
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
    /// Returns sanitized HTML content for secure iframe embedding
    /// This method handles report fetching, token verification, and response generation
    pub async fn serve_sandboxed_report(
        &self,
        _state: &Arc<AppState>,
        report: &Report,
        sandbox_token: &str,
        language: Option<&str>,
        chart_modules_content: Option<&str>
    ) -> Result<Response, Box<dyn StdError + Send + Sync>> {
        info!("🔒 IframeRenderer: Serving sandboxed content for report {} with token {}", report.id, sandbox_token);

        // Create sandboxed version with complete HTML document
        let sandboxed_report = self.create_sandboxed_report(report, chart_modules_content);

        // Verify sandbox token
        if sandboxed_report.sandbox_token != sandbox_token {
            error!("❌ IframeRenderer: Invalid sandbox token for report {}", report.id);
            return Ok(Response::builder()
                .status(StatusCode::FORBIDDEN)
                .header("content-type", "text/plain")
                .body(Body::from("Invalid sandbox token"))
                .unwrap_or_else(|e| {
                    warn!("⚠️ Failed to build forbidden response: {}", e);
                    Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Body::from("Response build error"))
                        .unwrap()  // This is guaranteed safe
                })
                .into_response()
            );
        }

        // Use the pre-generated complete HTML document from cache
        // If a specific language is requested and it's different from default (vi), regenerate
        let sandboxed_html = match language {
            Some(lang) if lang != "vi" => {
                // Regenerate with specific language
                self.regenerate_html_document(&sandboxed_report, Some(lang))
            }
            _ => {
                // ✅ IDIOMATIC: Move ownership instead of cloning 100-500KB HTML
                // sandboxed_report is not used after this point
                sandboxed_report.complete_html_document
            }
        };

        info!("✅ IframeRenderer: Serving HTML document for report {} with language {:?} ({} bytes)",
                report.id, language.unwrap_or("vi"), sandboxed_html.len());

        // Return response with security headers
        Ok(Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "text/html; charset=utf-8")
            .header("x-frame-options", "SAMEORIGIN")
            .header("content-security-policy", "default-src 'self' 'unsafe-inline'; script-src 'self' 'unsafe-inline' https://cdnjs.cloudflare.com https://fonts.googleapis.com https://cdn.tailwindcss.com; style-src 'self' 'unsafe-inline' https://cdnjs.cloudflare.com https://fonts.googleapis.com; font-src 'self' https://cdnjs.cloudflare.com https://fonts.gstatic.com; img-src 'self' data: https:; connect-src 'self'")
            .header("x-content-type-options", "nosniff")
            .header("cache-control", "private, max-age=3600")
            .header("access-control-allow-origin", "*")
            .header("access-control-allow-methods", "GET, POST, OPTIONS")
            .header("access-control-allow-headers", "Content-Type")
            .body(Body::from(sandboxed_html))
            .unwrap_or_else(|e| {
                warn!("⚠️ Failed to build sandboxed response: {}", e);
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::from("Response build error"))
                    .unwrap()  // This is guaranteed safe
            })
            .into_response()
        )
    }
}

impl Default for IframeRenderer {
    fn default() -> Self {
        Self::new()
    }
}
