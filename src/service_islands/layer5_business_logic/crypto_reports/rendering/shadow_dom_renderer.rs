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

use std::{sync::Arc, error::Error as StdError};
use axum::{
    response::{Response, IntoResponse},
    http::StatusCode,
    body::Body
};
use lazy_static::lazy_static;
use tracing::{info, warn, error};

use crate::service_islands::layer1_infrastructure::AppState;
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

    /// Create sandboxed report (reuses iframe logic for token generation)
    ///
    /// Generates security token and sanitizes content for Shadow DOM delivery
    pub fn create_sandboxed_report(&self, report: &Report, chart_modules_content: Option<&str>) -> SandboxedReport {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        // Generate security token based on report ID and creation time
        let mut hasher = DefaultHasher::new();
        report.id.hash(&mut hasher);
        report.created_at.hash(&mut hasher);
        let sandbox_token = format!("sb_{:x}", hasher.finish());

        info!("🌓 ShadowDomRenderer: Generated shadow DOM token for report {}: {}", report.id, sandbox_token);

        // Create sandboxed report with sanitized content
        SandboxedReport {
            id: report.id,
            html_content: sanitize_html_content(&report.html_content),
            css_content: report.css_content.as_ref().map(|css| sanitize_css_content(css)),
            js_content: report.js_content.as_ref().map(|js| sanitize_js_content(js)),
            html_content_en: report.html_content_en.as_ref().map(|html| sanitize_html_content(html)),
            js_content_en: report.js_content_en.as_ref().map(|js| sanitize_js_content(js)),
            created_at: report.created_at,
            sandbox_token,
            chart_modules_content: chart_modules_content.map(str::to_owned),
            complete_html_document: String::new(), // Not used for Shadow DOM
        }
    }

    /// Generate Shadow DOM content for Declarative Shadow DOM architecture
    ///
    /// Creates HTML fragment to be embedded within <template shadowrootmode="open">
    /// This is a modern replacement for the iframe-based approach with better performance
    pub fn generate_shadow_dom_content(&self, sandboxed_report: &SandboxedReport, language: Option<&str>, chart_modules_content: Option<&str>) -> String {
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
    /// Returns HTML fragment for embedding within <template shadowrootmode="open">
    /// This is a modern replacement for serve_sandboxed_report with better performance
    pub async fn serve_shadow_dom_content(
        &self,
        _state: &Arc<AppState>,
        report: &Report,
        shadow_dom_token: &str,
        language: Option<&str>,
        chart_modules_content: Option<&str>
    ) -> Result<Response, Box<dyn StdError + Send + Sync>> {
        info!("🌓 ShadowDomRenderer: Serving Shadow DOM content for report {} with token {}", report.id, shadow_dom_token);

        // Create sandboxed version (reuse existing logic)
        let sandboxed_report = self.create_sandboxed_report(report, chart_modules_content);

        // Verify shadow DOM token (same as sandbox token)
        if sandboxed_report.sandbox_token != shadow_dom_token {
            error!("❌ ShadowDomRenderer: Invalid shadow DOM token for report {}", report.id);
            return Ok(Response::builder()
                .status(StatusCode::FORBIDDEN)
                .header("content-type", "text/plain")
                .body(Body::from("Invalid shadow DOM token"))
                .unwrap_or_else(|e| {
                    warn!("⚠️ Failed to build forbidden response: {}", e);
                    Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Body::from("Response build error"))
                        .unwrap()
                })
                .into_response()
            );
        }

        // Generate Shadow DOM content
        let shadow_dom_html = self.generate_shadow_dom_content(&sandboxed_report, language, chart_modules_content);

        info!("✅ ShadowDomRenderer: Serving Shadow DOM content for report {} with language {:?} ({} bytes)",
              report.id, language.unwrap_or("vi"), shadow_dom_html.len());

        // Return response with appropriate headers
        Ok(Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "text/html; charset=utf-8")
            .header("x-content-type-options", "nosniff")
            .header("cache-control", "private, max-age=3600")
            .header("access-control-allow-origin", "*")
            .header("access-control-allow-methods", "GET, POST, OPTIONS")
            .header("access-control-allow-headers", "Content-Type")
            .body(Body::from(shadow_dom_html))
            .unwrap_or_else(|e| {
                warn!("⚠️ Failed to build Shadow DOM response: {}", e);
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::from("Response build error"))
                    .unwrap()
            })
            .into_response()
        )
    }
}

impl Default for ShadowDomRenderer {
    fn default() -> Self {
        Self::new()
    }
}
