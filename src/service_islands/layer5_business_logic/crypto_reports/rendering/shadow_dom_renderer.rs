//! Shadow DOM Renderer
//!
//! This module handles Declarative Shadow DOM rendering for crypto reports.
//! This is the modern approach that replaces iframe-based rendering.
//!
//! Features:
//! - Declarative Shadow DOM for better performance
//! - Multi-language support (Vietnamese and English)
//! - Pre-loaded templates for optimal performance
//! - Content sanitization for security
//! - Better SEO and accessibility compared to iframe

use std::sync::Arc;
use axum::response::Response;
use lazy_static::lazy_static;
use tracing::{info, warn};

use crate::service_islands::layer1_infrastructure::AppState;
use crate::service_islands::layer5_business_logic::shared::{
    generate_sandbox_token,
    verify_sandbox_token,
    build_shadow_dom_response,
    build_forbidden_response,
    Layer5Result,
};
use super::shared::{Report, SandboxedReport, sanitize_html_content, sanitize_css_content, sanitize_js_content};

lazy_static! {
    /// Pre-loaded Shadow DOM template for modern DSD architecture
    /// Replaces iframe-based approach with Declarative Shadow DOM
    static ref VIEW_SHADOW_DOM_TEMPLATE: String = {
        std::fs::read_to_string("shared_components/view_shadow_dom.html")
            .unwrap_or_else(|_| {
                include_str!("../../../../../shared_components/view_shadow_dom.html").to_string()
            })
    };
}

/// Shadow DOM Renderer
///
/// Handles all Declarative Shadow DOM rendering logic for crypto reports
#[derive(Clone)]
pub struct ShadowDomRenderer;

impl ShadowDomRenderer {
    /// Create a new Shadow DOM renderer
    pub fn new() -> Self {
        Self
    }

    /// Create sandboxed report with secure token generation
    ///
    /// Generates cryptographically secure token and sanitizes content for Shadow DOM delivery.
    ///
    /// # Security
    /// Uses blake3 for cryptographically secure token generation.
    pub fn create_sandboxed_report(&self, report: &Report, chart_modules_content: Option<&str>) -> SandboxedReport {
        // Generate cryptographically secure token
        let sandbox_token = generate_sandbox_token(report.id, &report.created_at);

        info!("ShadowDomRenderer: Generated secure shadow DOM token for report {}: {}", report.id, sandbox_token);

        // Create sandboxed report with sanitized content using as_deref() pattern
        SandboxedReport {
            id: report.id,
            html_content: sanitize_html_content(&report.html_content).into_owned(),
            css_content: report.css_content.as_deref().map(sanitize_css_content),
            js_content: report.js_content.as_deref().map(|js| sanitize_js_content(js).into_owned()),
            html_content_en: report.html_content_en.as_deref().map(|html| sanitize_html_content(html).into_owned()),
            js_content_en: report.js_content_en.as_deref().map(|js| sanitize_js_content(js).into_owned()),
            created_at: report.created_at,
            sandbox_token,
            chart_modules_content: chart_modules_content.map(str::to_owned),
            complete_html_document: String::new(), // Not used for Shadow DOM
        }
    }

    /// Generate Shadow DOM content for Declarative Shadow DOM architecture
    ///
    /// Creates HTML fragment to be embedded within <template shadowrootmode="open">.
    /// This is a modern replacement for the iframe-based approach with better performance.
    ///
    /// # Performance
    /// Uses as_deref() pattern for zero-allocation Option handling.
    pub fn generate_shadow_dom_content(&self, sandboxed_report: &SandboxedReport, language: Option<&str>, chart_modules_content: Option<&str>) -> String {
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

        // Use pre-loaded Shadow DOM template
        let template_content = &*VIEW_SHADOW_DOM_TEMPLATE;

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

    /// Serve Shadow DOM content for Declarative Shadow DOM architecture
    ///
    /// Returns HTML fragment for embedding within <template shadowrootmode="open">.
    /// This is a modern replacement for serve_sandboxed_report with better performance.
    ///
    /// # Security
    /// Uses constant-time token comparison to prevent timing attacks.
    pub async fn serve_shadow_dom_content(
        &self,
        _state: &Arc<AppState>,
        report: &Report,
        shadow_dom_token: &str,
        language: Option<&str>,
        chart_modules_content: Option<&str>
    ) -> Layer5Result<Response> {
        info!("ShadowDomRenderer: Serving Shadow DOM content for report {} with token {}", report.id, shadow_dom_token);

        // Verify shadow DOM token using constant-time comparison
        if !verify_sandbox_token(shadow_dom_token, report.id, &report.created_at) {
            warn!("ShadowDomRenderer: Invalid shadow DOM token for report {}", report.id);
            return Ok(build_forbidden_response("Invalid shadow DOM token"));
        }

        // Create sandboxed version
        let sandboxed_report = self.create_sandboxed_report(report, chart_modules_content);

        // Generate Shadow DOM content
        let shadow_dom_html = self.generate_shadow_dom_content(&sandboxed_report, language, chart_modules_content);

        info!("ShadowDomRenderer: Serving Shadow DOM content for report {} with language {:?} ({} bytes)",
              report.id, language.unwrap_or("vi"), shadow_dom_html.len());

        // Return response using safe builder
        Ok(build_shadow_dom_response(shadow_dom_html))
    }
}

impl Default for ShadowDomRenderer {
    fn default() -> Self {
        Self::new()
    }
}
