//! Shared Rendering Utilities
//!
//! This module contains common code used by both iframe and Shadow DOM renderers:
//! - Data models (Report, SandboxedReport)
//! - Pre-compiled regex patterns for sanitization
//! - Sanitization functions for HTML, CSS, and JavaScript

use serde::{Serialize, Deserialize};
use sqlx::FromRow;
use lazy_static::lazy_static;
use regex::Regex;

// ✅ PERFORMANCE OPTIMIZATION: Pre-compiled regex patterns for sanitization
// These regexes are compiled once at startup instead of on every request,
// eliminating ~386,867 regex compilations/second at 16,829+ RPS
lazy_static! {
    /// Pre-compiled HTML sanitization patterns
    pub static ref HTML_SANITIZE_PATTERNS: Vec<Regex> = vec![
        Regex::new(r"<script[^>]*>.*?</script>").unwrap(),  // Remove inline scripts
        Regex::new(r"<iframe[^>]*>.*?</iframe>").unwrap(),  // Remove nested iframes
        Regex::new(r"<object[^>]*>.*?</object>").unwrap(),  // Remove objects
        Regex::new(r"<embed[^>]*>.*?</embed>").unwrap(),    // Remove embeds
        Regex::new(r"<applet[^>]*>.*?</applet>").unwrap(),  // Remove applets
        Regex::new(r"javascript:").unwrap(),                 // Remove javascript: URLs
        Regex::new(r"on\w+\s*=").unwrap(),                   // Remove event handlers
    ];

    /// Pre-compiled CSS sanitization patterns
    pub static ref CSS_SANITIZE_PATTERNS: Vec<Regex> = vec![
        Regex::new(r"expression\s*\(").unwrap(),            // Remove CSS expressions
        Regex::new(r"javascript\s*:").unwrap(),             // Remove javascript URLs in CSS
        Regex::new(r"@import").unwrap(),                    // Remove imports
        Regex::new(r"behavior\s*:").unwrap(),               // Remove IE behaviors
        Regex::new(r"position\s*:\s*fixed").unwrap(),       // Prevent fixed positioning
        Regex::new(r"position\s*:\s*absolute").unwrap(),    // Be careful with absolute
        Regex::new(r"z-index\s*:\s*[0-9]{4,}").unwrap(),    // Prevent high z-index
        Regex::new(r"!important\s*;").unwrap(),             // Remove !important
    ];

    /// Pre-compiled JavaScript sanitization patterns
    pub static ref JS_SANITIZE_PATTERNS: Vec<Regex> = vec![
        Regex::new(r"eval\s*\(").unwrap(),                  // Remove eval calls
        Regex::new(r"Function\s*\(").unwrap(),              // Remove Function constructor
        Regex::new(r"setTimeout\s*\(").unwrap(),            // Remove setTimeout
        Regex::new(r"setInterval\s*\(").unwrap(),           // Remove setInterval
        Regex::new(r"document\.write").unwrap(),            // Remove document.write
        Regex::new(r"window\.location").unwrap(),           // Remove location changes
        Regex::new(r"parent\.").unwrap(),                   // Remove parent access
        Regex::new(r"top\.").unwrap(),                      // Remove top access
    ];
}

/// Report model - exactly from archive_old_code/models.rs with iframe sandboxing support
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Report {
    pub id: i32,
    pub html_content: String,
    pub css_content: Option<String>,
    pub js_content: Option<String>,
    pub html_content_en: Option<String>,
    pub js_content_en: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Sandboxed report content for secure iframe delivery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxedReport {
    pub id: i32,
    pub html_content: String,
    pub css_content: Option<String>,
    pub js_content: Option<String>,
    pub html_content_en: Option<String>,
    pub js_content_en: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub sandbox_token: String, // Security token for iframe access
    pub chart_modules_content: Option<String>, // Chart modules content for iframe
    pub complete_html_document: String, // Complete HTML document ready for iframe
}

/// Sanitize HTML content for sandbox
///
/// Removes potentially dangerous HTML elements and attributes
/// ✅ PERFORMANCE OPTIMIZED: Uses pre-compiled regex patterns (zero compilation overhead)
pub fn sanitize_html_content(html: &str) -> String {
    // Check if sanitization is needed using pre-compiled patterns
    let needs_sanitization = HTML_SANITIZE_PATTERNS.iter()
        .any(|re| re.is_match(html));

    if !needs_sanitization {
        // No dangerous content, return as-is
        return html.to_string();
    }

    // Sanitize using pre-compiled regex patterns
    let mut sanitized = html.to_string();
    for re in HTML_SANITIZE_PATTERNS.iter() {
        sanitized = re.replace_all(&sanitized, "").into_owned();
    }

    sanitized
}

/// Sanitize CSS content for sandbox
///
/// Removes potentially dangerous CSS properties and expressions
/// Enhanced to prevent CSS from affecting parent page
/// ✅ PERFORMANCE: Uses pre-compiled regex patterns from lazy_static
pub fn sanitize_css_content(css: &str) -> String {
    // Check if sanitization is needed
    let needs_sanitization = CSS_SANITIZE_PATTERNS.iter()
        .any(|re| re.is_match(css));

    if !needs_sanitization {
        // No dangerous patterns found - wrap and return
        return format!(
            "/* CSS isolated within iframe sandbox */\n.sandboxed-report-container {{\n{}\n}}",
            css
        );
    }

    // Apply sanitization using pre-compiled patterns
    let mut sanitized = css.to_string();
    for re in CSS_SANITIZE_PATTERNS.iter() {
        sanitized = re.replace_all(&sanitized, "").into_owned();
    }

    // Additional safety: wrap all CSS rules to ensure they only apply within iframe
    let wrapped_css = format!(
        "/* CSS isolated within iframe sandbox */\n.sandboxed-report-container {{\n{}\n}}",
        sanitized
    );

    wrapped_css
}

/// Sanitize JavaScript content for sandbox
///
/// Applies basic JavaScript sanitization for sandbox environment
/// ✅ PERFORMANCE: Uses pre-compiled regex patterns from lazy_static
pub fn sanitize_js_content(js: &str) -> String {
    // Check if sanitization is needed
    let needs_sanitization = JS_SANITIZE_PATTERNS.iter()
        .any(|re| re.is_match(js));

    if !needs_sanitization {
        return js.to_string();
    }

    // Apply sanitization using pre-compiled patterns
    let mut sanitized = js.to_string();
    for re in JS_SANITIZE_PATTERNS.iter() {
        sanitized = re.replace_all(&sanitized, "/* SANITIZED */").into_owned();
    }

    sanitized
}
