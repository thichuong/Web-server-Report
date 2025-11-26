//! Shared Rendering Utilities
//!
//! This module contains common code used by both iframe and Shadow DOM renderers:
//! - Data models (Report, `SandboxedReport`)
//! - Pre-compiled regex patterns for sanitization
//! - Sanitization functions for HTML, CSS, and JavaScript
//! - From trait implementations for type conversions

use std::sync::LazyLock;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::borrow::Cow;

// Import Layer 3 data model for From trait
use crate::service_islands::layer3_communication::data_communication::ReportData;

// ✅ PERFORMANCE OPTIMIZATION: Pre-compiled regex patterns for sanitization
// These regexes are compiled once at startup instead of on every request,
// eliminating ~386,867 regex compilations/second at 16,829+ RPS

/// Pre-compiled HTML sanitization patterns
#[allow(clippy::expect_used)] // Safe: Regex patterns are hardcoded and verified
pub static HTML_SANITIZE_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| vec![
    Regex::new(r"<script[^>]*>.*?</script>").expect("Invalid regex"),  // Remove inline scripts
    Regex::new(r"<iframe[^>]*>.*?</iframe>").expect("Invalid regex"),  // Remove nested iframes
    Regex::new(r"<object[^>]*>.*?</object>").expect("Invalid regex"),  // Remove objects
    Regex::new(r"<embed[^>]*>.*?</embed>").expect("Invalid regex"),    // Remove embeds
    Regex::new(r"<applet[^>]*>.*?</applet>").expect("Invalid regex"),  // Remove applets
    Regex::new(r"javascript:").expect("Invalid regex"),                 // Remove javascript: URLs
    Regex::new(r"on\w+\s*=").expect("Invalid regex"),                   // Remove event handlers
]);

/// Pre-compiled CSS sanitization patterns
#[allow(clippy::expect_used)] // Safe: Regex patterns are hardcoded and verified
pub static CSS_SANITIZE_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| vec![
    Regex::new(r"expression\s*\(").expect("Invalid regex"),            // Remove CSS expressions
    Regex::new(r"javascript\s*:").expect("Invalid regex"),             // Remove javascript URLs in CSS
    Regex::new(r"@import").expect("Invalid regex"),                    // Remove imports
    Regex::new(r"behavior\s*:").expect("Invalid regex"),               // Remove IE behaviors
    Regex::new(r"position\s*:\s*fixed").expect("Invalid regex"),       // Prevent fixed positioning
    Regex::new(r"position\s*:\s*absolute").expect("Invalid regex"),    // Be careful with absolute
    Regex::new(r"z-index\s*:\s*[0-9]{4,}").expect("Invalid regex"),    // Prevent high z-index
    Regex::new(r"!important\s*;").expect("Invalid regex"),             // Remove !important
]);

/// Pre-compiled JavaScript sanitization patterns
#[allow(clippy::expect_used)] // Safe: Regex patterns are hardcoded and verified
pub static JS_SANITIZE_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| vec![
    Regex::new(r"eval\s*\(").expect("Invalid regex"),                  // Remove eval calls
    Regex::new(r"Function\s*\(").expect("Invalid regex"),              // Remove Function constructor
    Regex::new(r"setTimeout\s*\(").expect("Invalid regex"),            // Remove setTimeout
    Regex::new(r"setInterval\s*\(").expect("Invalid regex"),           // Remove setInterval
    Regex::new(r"document\.write").expect("Invalid regex"),            // Remove document.write
    Regex::new(r"window\.location").expect("Invalid regex"),           // Remove location changes
    Regex::new(r"parent\.").expect("Invalid regex"),                   // Remove parent access
    Regex::new(r"top\.").expect("Invalid regex"),                      // Remove top access
]);

/// Report model - exactly from `archive_old_code/models.rs` with iframe sandboxing support
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

/// Implement From trait for automatic conversion from Layer 3 `ReportData`
/// This eliminates manual field-by-field copying in `report_creator.rs`
impl From<ReportData> for Report {
    #[inline]
    fn from(data: ReportData) -> Self {
        Self {
            id: data.id,
            html_content: data.html_content,
            css_content: data.css_content,
            js_content: data.js_content,
            html_content_en: data.html_content_en,
            js_content_en: data.js_content_en,
            created_at: data.created_at,
        }
    }
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
/// Removes potentially dangerous HTML elements and attributes.
///
/// # Performance Optimizations
/// - Uses pre-compiled regex patterns (zero compilation overhead)
/// - Single-pass detection with Cow to avoid allocation when not needed
/// - Only allocates when sanitization is actually required
#[inline]
pub fn sanitize_html_content(html: &str) -> Cow<'_, str> {
    // Single-pass: apply all patterns, Cow avoids allocation if no changes
    let mut result: Cow<'_, str> = Cow::Borrowed(html);

    for re in HTML_SANITIZE_PATTERNS.iter() {
        match result {
            Cow::Borrowed(s) => {
                let replaced = re.replace_all(s, "");
                if let Cow::Owned(owned) = replaced {
                    result = Cow::Owned(owned);
                }
                // If Borrowed, no change was made, keep result as-is
            }
            Cow::Owned(ref s) => {
                let replaced = re.replace_all(s, "");
                if let Cow::Owned(owned) = replaced {
                    result = Cow::Owned(owned);
                }
            }
        }
    }

    result
}

/// CSS wrapper prefix for isolation
const CSS_WRAPPER_PREFIX: &str =
    "/* CSS isolated within iframe sandbox */\n.sandboxed-report-container {\n";
const CSS_WRAPPER_SUFFIX: &str = "\n}";

/// Sanitize CSS content for sandbox
///
/// Removes potentially dangerous CSS properties and expressions.
/// Enhanced to prevent CSS from affecting parent page.
///
/// # Performance Optimizations
/// - Uses pre-compiled regex patterns from `LazyLock`
/// - Single-pass with Cow to minimize allocations
/// - Wrapping done only once at the end
#[inline]
pub fn sanitize_css_content(css: &str) -> String {
    // Single-pass sanitization with Cow
    let mut result: Cow<'_, str> = Cow::Borrowed(css);

    for re in CSS_SANITIZE_PATTERNS.iter() {
        match result {
            Cow::Borrowed(s) => {
                let replaced = re.replace_all(s, "");
                if let Cow::Owned(owned) = replaced {
                    result = Cow::Owned(owned);
                }
            }
            Cow::Owned(ref s) => {
                let replaced = re.replace_all(s, "");
                if let Cow::Owned(owned) = replaced {
                    result = Cow::Owned(owned);
                }
            }
        }
    }

    // Wrap CSS for isolation - single allocation for final result
    let mut wrapped =
        String::with_capacity(CSS_WRAPPER_PREFIX.len() + result.len() + CSS_WRAPPER_SUFFIX.len());
    wrapped.push_str(CSS_WRAPPER_PREFIX);
    wrapped.push_str(&result);
    wrapped.push_str(CSS_WRAPPER_SUFFIX);
    wrapped
}

/// Sanitize JavaScript content for sandbox
///
/// Applies basic JavaScript sanitization for sandbox environment.
///
/// # Performance Optimizations
/// - Uses pre-compiled regex patterns from `LazyLock`
/// - Single-pass with Cow to minimize allocations
#[inline]
pub fn sanitize_js_content(js: &str) -> Cow<'_, str> {
    // Single-pass sanitization with Cow
    let mut result: Cow<'_, str> = Cow::Borrowed(js);

    for re in JS_SANITIZE_PATTERNS.iter() {
        match result {
            Cow::Borrowed(s) => {
                let replaced = re.replace_all(s, "/* SANITIZED */");
                if let Cow::Owned(owned) = replaced {
                    result = Cow::Owned(owned);
                }
            }
            Cow::Owned(ref s) => {
                let replaced = re.replace_all(s, "/* SANITIZED */");
                if let Cow::Owned(owned) = replaced {
                    result = Cow::Owned(owned);
                }
            }
        }
    }

    result
}
