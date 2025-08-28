//! PDF Generator Component
//! 
//! This component handles PDF generation operations for crypto reports,
//! including A4 optimization and print-ready formatting with iframe sandboxing support.

use std::{error::Error as StdError, sync::Arc};
use tera::Context;
use axum::{
    response::{Response, IntoResponse},
    http::StatusCode,
    body::Body
};

// Import from current state - will be refactored when lower layers are implemented
use crate::state::AppState;

// Import models from report_creator
use super::report_creator::{ReportCreator, SandboxedReport};

/// PDF Generator
/// 
/// Manages PDF generation operations for crypto reports with A4 optimization.
pub struct PdfGenerator {
    report_creator: ReportCreator,
}

impl PdfGenerator {
    /// Create a new PdfGenerator
    pub fn new() -> Self {
        Self {
            report_creator: ReportCreator::new(),
        }
    }
    
    /// Health check for PDF generator
    pub async fn health_check(&self) -> bool {
        // Verify PDF generation is working
        true // Will implement actual health check
    }

    /// Generate PDF template for a specific crypto report by ID
    /// 
    /// This method fetches a report by ID and renders it using the PDF-optimized template
    /// "crypto/routes/reports/pdf.html" with proper context for print-friendly output
    pub async fn crypto_report_pdf_with_tera(
        &self, 
        app_state: &Arc<AppState>, 
        report_id: i32
    ) -> Result<String, Box<dyn StdError + Send + Sync>> {
        println!("📄 PdfGenerator::crypto_report_pdf_with_tera - Report ID: {}", report_id);
        
        // TODO: L1/L2 Cache logic (similar to crypto_report_by_id_with_tera but with different cache key)
        let _cache_key = format!("crypto_report_pdf_{}", report_id);
        println!("Processed 0 requests to crypto_report_pdf");
        println!("🔍 L1 Cache miss for PDF report ID: {} - checking L2 cache (Redis)", report_id);
        println!("🔍 L1+L2 Cache miss for PDF report ID: {} - fetching from DB", report_id);
        
        // Fetch and cache report by ID using ReportCreator
        let report_option = self.report_creator.fetch_and_cache_report_by_id(app_state, report_id).await?;
        
        let report = match report_option {
            Some(report) => report,
            None => {
                return Err(format!("Report #{} not found", report_id).into());
            }
        };
        
        // Format creation date for display (using simple formatting without timezone)
        let created_at_display = report.created_at.format("%d/%m/%Y lúc %H:%M:%S").to_string();
        
        // Fetch chart modules content 
        let chart_modules_content = self.report_creator.get_chart_modules_content().await;

        // Setup template context for PDF rendering
        let mut context = Context::new();
        context.insert("report", &report);
        context.insert("created_at_display", &created_at_display);
        context.insert("chart_modules_content", &chart_modules_content);
        
        // Add sandbox token for PDF iframe access
        let sandboxed_report = self.create_pdf_sandboxed_report(&report, Some(&chart_modules_content));
        context.insert("sandbox_token", &sandboxed_report.sandbox_token);
        
        // Add additional PDF-specific context
        context.insert("template_type", "pdf");
        context.insert("print_optimized", &true);
        
        // Use PDF-specific template path
        let template_name = "crypto/routes/reports/pdf.html";
        
        let render_result = tokio::task::spawn({
            let context = context.clone();
            let tera = app_state.tera.clone();
            let template_name = template_name.to_string();
            
            async move {
                tera.render(&template_name, &context)
            }
        }).await;

        match render_result {
            Ok(Ok(html)) => {
                println!("✅ PDF template rendered from DB for report ID: {} - crypto_report_pdf complete", report_id);
                
                // TODO: Cache the rendered PDF HTML in L1/L2 with longer TTL (PDFs change less frequently)
                // self.cache_manager.set_l1(&cache_key, &html, Duration::from_secs(1800)).await; // 30min cache
                
                Ok(html)
            }
            Ok(Err(e)) => {
                eprintln!("❌ PDF template render error for report ID {}: {:#?}", report_id, e);
                let mut src = e.source();
                while let Some(s) = src {
                    eprintln!("❌ PDF template render error source: {:#?}", s);
                    src = s.source();
                }
                Err(format!("PDF template render error: {}", e).into())
            }
            Err(e) => {
                eprintln!("❌ PDF template task join error: {:#?}", e);
                Err(format!("PDF template task join error: {}", e).into())
            }
        }
    }

    /// Create sandboxed PDF report with navigation removed
    /// 
    /// Creates a sandboxed version specifically for PDF generation without navigation sidebar
    pub fn create_pdf_sandboxed_report(&self, report: &super::report_creator::Report, chart_modules_content: Option<&str>) -> SandboxedReport {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        // Generate security token based on report ID and creation time
        let mut hasher = DefaultHasher::new();
        report.id.hash(&mut hasher);
        report.created_at.hash(&mut hasher);
        "pdf".hash(&mut hasher); // Add PDF identifier to token
        let sandbox_token = format!("pdf_sb_{:x}", hasher.finish());
        
        println!("🔒 PdfGenerator: Generated PDF sandbox token for report {}: {}", report.id, sandbox_token);
        
        // Create sandboxed report with sanitized content using ReportCreator's sanitization methods
        let mut sandboxed_report = SandboxedReport {
            id: report.id,
            html_content: self.report_creator.sanitize_html_content(&report.html_content),
            css_content: report.css_content.as_ref().map(|css| self.report_creator.sanitize_css_content(css)),
            js_content: report.js_content.as_ref().map(|js| self.report_creator.sanitize_js_content(js)),
            html_content_en: report.html_content_en.as_ref().map(|html| self.report_creator.sanitize_html_content(html)),
            js_content_en: report.js_content_en.as_ref().map(|js| self.report_creator.sanitize_js_content(js)),
            created_at: report.created_at,
            sandbox_token: sandbox_token.clone(),
            chart_modules_content: chart_modules_content.map(|s| s.to_string()),
            complete_html_document: String::new(), // Will be populated below
        };
        
        // Generate complete HTML document using PDF template and store it
        sandboxed_report.complete_html_document = self.generate_pdf_sandboxed_html_document(&sandboxed_report, None, chart_modules_content);
        
        println!("📄 PdfGenerator: Complete PDF HTML document generated for report {} ({} bytes)", 
                report.id, sandboxed_report.complete_html_document.len());
        
        sandboxed_report
    }

    /// Generate complete PDF sandboxed HTML document
    /// 
    /// Creates a self-contained HTML document for PDF iframe embedding without navigation
    /// Uses pdf_iframe.html template for navigation-free PDF generation
    pub fn generate_pdf_sandboxed_html_document(&self, sandboxed_report: &SandboxedReport, language: Option<&str>, chart_modules_content: Option<&str>) -> String {
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

        // Load PDF iframe template from external file (navigation-free version)
        let template_path = "shared_components/pdf_iframe.html";
        let template_content = match std::fs::read_to_string(template_path) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("⚠️ PdfGenerator: Failed to load PDF template {}: {}. Using fallback.", template_path, e);
                // Fallback to regular iframe template if PDF template is not available
                match std::fs::read_to_string("shared_components/view_iframe.html") {
                    Ok(content) => content,
                    Err(e2) => {
                        eprintln!("⚠️ PdfGenerator: Failed to load fallback template: {}. Using minimal template.", e2);
                        // Minimal fallback template
                        r#"<!DOCTYPE html>
<html lang="{{default_lang}}">
<head>
    <meta charset="UTF-8">
    <title>PDF Report #{{report_id}}</title>
</head>
<body>
    <div id="content-vi" class="{{vi_active_class}}">{{html_content_vi}}</div>
    <div id="content-en" class="{{en_active_class}}">{{html_content_en}}</div>
    <script>{{chart_modules}}</script>
    <script>{{js_content_vi}}</script>
    <script>{{js_content_en}}</script>
</body>
</html>"#.to_string()
                    }
                }
            }
        };

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

    /// Serve PDF sandboxed report content for iframe
    /// 
    /// Returns sanitized HTML content for secure PDF iframe embedding without navigation
    pub async fn serve_pdf_sandboxed_report(
        &self,
        state: &Arc<AppState>,
        report_id: i32,
        sandbox_token: &str,
        language: Option<&str>,
        chart_modules_content: Option<&str>
    ) -> Result<Response, Box<dyn StdError + Send + Sync>> {
        println!("🔒 PdfGenerator: Serving PDF sandboxed content for report {} with token {}", report_id, sandbox_token);
        
        // Fetch report from database
        let report_result = if report_id == -1 {
            self.report_creator.fetch_and_cache_latest_report(state).await
        } else {
            self.report_creator.fetch_and_cache_report_by_id(state, report_id).await
        };

        match report_result {
            Ok(Some(report)) => {
                // Create PDF sandboxed version with complete HTML document
                let sandboxed_report = self.create_pdf_sandboxed_report(&report, chart_modules_content);
                
                // Verify sandbox token
                if sandboxed_report.sandbox_token != sandbox_token {
                    println!("❌ PdfGenerator: Invalid PDF sandbox token for report {}", report_id);
                    return Ok(Response::builder()
                        .status(StatusCode::FORBIDDEN)
                        .header("content-type", "text/plain")
                        .body(Body::from("Invalid PDF sandbox token"))
                        .unwrap()
                        .into_response()
                    );
                }
                
                // Use the pre-generated complete HTML document from cache
                // If a specific language is requested and it's different from default (vi), regenerate
                let sandboxed_html = if let Some(lang) = language {
                    if lang != "vi" {
                        // Regenerate with specific language
                        self.generate_pdf_sandboxed_html_document(&sandboxed_report, Some(lang), chart_modules_content)
                    } else {
                        // Use cached default document
                        sandboxed_report.complete_html_document.clone()
                    }
                } else {
                    // Use cached default document
                    sandboxed_report.complete_html_document.clone()
                };
                
                println!("✅ PdfGenerator: Serving PDF HTML document for report {} with language {:?} ({} bytes)", 
                        report_id, language.unwrap_or("vi"), sandboxed_html.len());
                
                // Return response with security headers optimized for PDF generation
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
                    .unwrap()
                    .into_response()
                )
            }
            Ok(None) => {
                println!("❌ PdfGenerator: Report {} not found for PDF sandboxing", report_id);
                Ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .header("content-type", "text/plain")
                    .body(Body::from("PDF Report not found"))
                    .unwrap()
                    .into_response()
                )
            }
            Err(e) => {
                eprintln!("❌ PdfGenerator: Database error serving PDF sandboxed report {}: {}", report_id, e);
                Ok(Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header("content-type", "text/plain")
                    .body(Body::from("Database error"))
                    .unwrap()
                    .into_response()
                )
            }
        }
    }
}
