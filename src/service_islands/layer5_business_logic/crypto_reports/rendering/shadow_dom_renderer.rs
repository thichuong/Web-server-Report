//! Shadow DOM Renderer
//!
//! Handles Declarative Shadow DOM rendering for crypto reports.
//! Modern approach replacing iframe-based rendering.
//!
//! Features:
//! - Declarative Shadow DOM for better performance
//! - Multi-language support (Vietnamese and English)
//! - Pre-loaded templates for optimal performance
//! - Content sanitization for security
//! - Better SEO and accessibility compared to iframe

use std::sync::{Arc, LazyLock};

use axum::response::Response;
use tracing::{info, warn};

use crate::service_islands::layer1_infrastructure::AppState;
use crate::service_islands::layer5_business_logic::shared::{
    build_forbidden_response, build_shadow_dom_response, generate_sandbox_token,
    verify_sandbox_token, Layer5Result,
};

use super::shared::{
    sanitize_css_content, sanitize_html_content, sanitize_js_content, Report, SandboxedReport,
};

/// Pre-loaded Shadow DOM template for modern DSD architecture.
/// Replaces iframe-based approach with Declarative Shadow DOM.
static VIEW_SHADOW_DOM_TEMPLATE: LazyLock<String> = LazyLock::new(|| {
    std::fs::read_to_string("shared_components/view_shadow_dom.html").unwrap_or_else(|_| {
        include_str!("../../../../../shared_components/view_shadow_dom.html").to_owned()
    })
});

/// Shadow DOM Renderer
///
/// Handles all Declarative Shadow DOM rendering logic for crypto reports.
#[derive(Clone, Copy, Default)]
pub struct ShadowDomRenderer;

impl ShadowDomRenderer {
    /// Create a new Shadow DOM renderer.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Create sandboxed report with secure token generation.
    ///
    /// Generates cryptographically secure token and sanitizes content for Shadow DOM delivery.
    ///
    /// # Security
    /// Uses blake3 for cryptographically secure token generation.
    #[must_use]
    pub fn create_sandboxed_report(
        &self,
        report: &Report,
        chart_modules_content: Option<&str>,
    ) -> SandboxedReport {
        let sandbox_token = generate_sandbox_token(report.id, &report.created_at);

        info!(
            report_id = report.id,
            token = %sandbox_token,
            "ShadowDomRenderer: Generated secure shadow DOM token"
        );

        SandboxedReport {
            id: report.id,
            html_content: sanitize_html_content(&report.html_content).into_owned(),
            css_content: report.css_content.as_deref().map(sanitize_css_content),
            js_content: report
                .js_content
                .as_deref()
                .map(|js| sanitize_js_content(js).into_owned()),
            html_content_en: report
                .html_content_en
                .as_deref()
                .map(|h| sanitize_html_content(h).into_owned()),
            js_content_en: report
                .js_content_en
                .as_deref()
                .map(|js| sanitize_js_content(js).into_owned()),
            created_at: report.created_at,
            sandbox_token,
            chart_modules_content: chart_modules_content.map(ToOwned::to_owned),
            complete_html_document: String::new(),
        }
    }

    /// Generate Shadow DOM content for Declarative Shadow DOM architecture.
    ///
    /// Creates HTML fragment to be embedded within `<template shadowrootmode="open">`.
    /// Modern replacement for iframe-based approach with better performance.
    ///
    /// # Performance
    /// Uses `as_deref()` pattern for zero-allocation Option handling.
    #[must_use]
    pub fn generate_shadow_dom_content(
        &self,
        sandboxed_report: &SandboxedReport,
        language: Option<&str>,
        chart_modules_content: Option<&str>,
    ) -> String {
        let lang = language.unwrap_or("vi");

        // Zero-allocation content resolution via as_deref()
        let html_vi = &sandboxed_report.html_content;
        let html_en = sandboxed_report
            .html_content_en
            .as_deref()
            .unwrap_or(html_vi);
        let js_vi = sandboxed_report.js_content.as_deref().unwrap_or_default();
        let js_en = sandboxed_report.js_content_en.as_deref().unwrap_or(js_vi);
        let css = sandboxed_report.css_content.as_deref().unwrap_or_default();

        // Prefer report's chart modules, fall back to parameter
        let chart_modules = sandboxed_report
            .chart_modules_content
            .as_deref()
            .or(chart_modules_content)
            .unwrap_or_default();

        // Language-based active class assignment
        let (vi_active, en_active) = if lang == "en" {
            ("", "active")
        } else {
            ("active", "")
        };

        // Template substitution using pre-loaded template
        VIEW_SHADOW_DOM_TEMPLATE
            .replace("{{default_lang}}", lang)
            .replace("{{report_id}}", &sandboxed_report.id.to_string())
            .replace("{{vi_active_class}}", vi_active)
            .replace("{{en_active_class}}", en_active)
            .replace("{{css_content}}", css)
            .replace("{{html_content_vi}}", html_vi)
            .replace("{{html_content_en}}", html_en)
            .replace("{{chart_modules}}", chart_modules)
            .replace("{{js_content_vi}}", js_vi)
            .replace("{{js_content_en}}", js_en)
    }

    /// Serve Shadow DOM content for Declarative Shadow DOM architecture.
    ///
    /// Returns HTML fragment for embedding within `<template shadowrootmode="open">`.
    /// Modern replacement for `serve_sandboxed_report` with better performance.
    ///
    /// # Security
    /// Uses constant-time token comparison to prevent timing attacks.
    pub async fn serve_shadow_dom_content(
        &self,
        _state: &Arc<AppState>,
        report: &Report,
        shadow_dom_token: &str,
        language: Option<&str>,
        chart_modules_content: Option<&str>,
    ) -> Layer5Result<Response> {
        info!(
            report_id = report.id,
            token = %shadow_dom_token,
            "ShadowDomRenderer: Serving Shadow DOM content"
        );

        // Verify token with constant-time comparison
        if !verify_sandbox_token(shadow_dom_token, report.id, &report.created_at) {
            warn!(
                report_id = report.id,
                "ShadowDomRenderer: Invalid shadow DOM token"
            );
            return Ok(build_forbidden_response("Invalid shadow DOM token"));
        }

        let sandboxed_report = self.create_sandboxed_report(report, chart_modules_content);
        let shadow_dom_html =
            self.generate_shadow_dom_content(&sandboxed_report, language, chart_modules_content);

        info!(
            report_id = report.id,
            language = language.unwrap_or("vi"),
            size_bytes = shadow_dom_html.len(),
            "ShadowDomRenderer: Shadow DOM content generated"
        );

        Ok(build_shadow_dom_response(shadow_dom_html))
    }
}
