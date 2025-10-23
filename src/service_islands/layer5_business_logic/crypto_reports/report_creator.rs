//! Report Creator Component
//! 
//! This component handles report creation business logic for crypto reports,
//! including report data fetching, processing, and chart modules management.

use serde::{Serialize, Deserialize};
use sqlx::{FromRow};
use std::{sync::Arc, error::Error as StdError};
use axum::{
    response::{Response, IntoResponse},
    http::StatusCode,
    body::Body
};

// Import from current state - will be refactored when lower layers are implemented
use crate::service_islands::layer1_infrastructure::AppState;
// Import Layer 3 data communication service - proper architecture
use crate::service_islands::layer3_communication::data_communication::CryptoDataService;
// Import Layer 1 infrastructure services
use crate::service_islands::layer1_infrastructure::ChartModulesIsland;

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

/// Report Creator
/// 
/// Manages report creation business logic with market analysis capabilities.
/// Uses Layer 3 data services and Layer 1 infrastructure services for proper architectural separation.
#[derive(Clone)]
pub struct ReportCreator {
    pub data_service: CryptoDataService,
    pub chart_modules_island: ChartModulesIsland,
}

impl ReportCreator {
    /// Create a new ReportCreator
    pub fn new() -> Self {
        Self {
            data_service: CryptoDataService::new(),
            chart_modules_island: ChartModulesIsland::new(),
        }
    }
    
    /// Health check for report creator
    pub async fn health_check(&self) -> bool {
        // Verify report creation is working and chart modules are accessible
        self.chart_modules_island.health_check().await
    }

    /// Fetch and cache latest report from database
    /// 
    /// Retrieves the most recent crypto report with full content using Layer 3 data service
    pub async fn fetch_and_cache_latest_report(
        &self,
        state: &Arc<AppState>,
    ) -> Result<Option<Report>, sqlx::Error> {
        println!("🔍 ReportCreator: Fetching latest crypto report from database via data service");
        
        // Use Layer 3 data service instead of direct database access
        let report_data = self.data_service.fetch_latest_report(state).await?;
        
        if let Some(data) = report_data {
            // Convert data layer model to business layer model
            let report = Report {
                id: data.id,
                html_content: data.html_content,
                css_content: data.css_content,
                js_content: data.js_content,
                html_content_en: data.html_content_en,
                js_content_en: data.js_content_en,
                created_at: data.created_at,
            };
            
            // Update latest id cache (business logic concern)
            state.cached_latest_id.store(report.id, std::sync::atomic::Ordering::Relaxed);
            println!("💾 ReportCreator: Cached latest crypto report {} from data service", report.id);
            
            // TODO: Implement L1/L2 caching logic when cache layers are ready
            
            Ok(Some(report))
        } else {
            println!("📭 ReportCreator: No latest crypto report available");
            Ok(None)
        }
    }

    /// Fetch and cache specific report by ID
    /// 
    /// Retrieves a crypto report by its ID with full content using Layer 3 data service
    pub async fn fetch_and_cache_report_by_id(
        &self,
        state: &Arc<AppState>,
        report_id: i32,
    ) -> Result<Option<Report>, sqlx::Error> {
        println!("🔍 ReportCreator: Fetching crypto report {} via data service", report_id);
        
        // Use Layer 3 data service instead of direct database access
        let report_data = self.data_service.fetch_report_by_id(state, report_id).await?;
        
        if let Some(data) = report_data {
            // Convert data layer model to business layer model
            let report = Report {
                id: data.id,
                html_content: data.html_content,
                css_content: data.css_content,
                js_content: data.js_content,
                html_content_en: data.html_content_en,
                js_content_en: data.js_content_en,
                created_at: data.created_at,
            };
            
            println!("💾 ReportCreator: Successfully processed crypto report {} from data service", report.id);
            
            // TODO: Implement L1/L2 caching logic when cache layers are ready
            
            Ok(Some(report))
        } else {
            println!("📭 ReportCreator: Crypto report {} not found via data service", report_id);
            Ok(None)
        }
    }

    /// Get chart modules content
    /// 
    /// Delegates to Layer 1 ChartModulesIsland for proper architectural separation.
    /// This method provides a business logic wrapper around the infrastructure service.
    pub async fn get_chart_modules_content(&self) -> String {
        println!("📊 ReportCreator: Requesting chart modules from Layer 1 Infrastructure");
        
        // Delegate to Layer 1 infrastructure service
        self.chart_modules_island.get_cached_chart_modules_content().await
    }

    /// Get available chart modules
    /// 
    /// Returns a list of available chart module names via Layer 1 infrastructure.
    #[allow(dead_code)]
    pub async fn get_available_chart_modules(&self) -> Vec<String> {
        self.chart_modules_island.get_available_modules().await
    }

    /// Generate sandboxed report content
    /// 
    /// Creates a secure sandboxed version of the report for iframe delivery.
    /// This method sanitizes content, generates a security token, and creates the complete HTML document.
    pub fn create_sandboxed_report(&self, report: &Report, chart_modules_content: Option<&str>) -> SandboxedReport {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        // Generate security token based on report ID and creation time
        let mut hasher = DefaultHasher::new();
        report.id.hash(&mut hasher);
        report.created_at.hash(&mut hasher);
        let sandbox_token = format!("sb_{:x}", hasher.finish());
        
        println!("🔒 ReportCreator: Generated sandbox token for report {}: {}", report.id, sandbox_token);
        
        // Create sandboxed report with sanitized content
        let mut sandboxed_report = SandboxedReport {
            id: report.id,
            html_content: self.sanitize_html_content(&report.html_content),
            css_content: report.css_content.as_ref().map(|css| self.sanitize_css_content(css)),
            js_content: report.js_content.as_ref().map(|js| self.sanitize_js_content(js)),
            html_content_en: report.html_content_en.as_ref().map(|html| self.sanitize_html_content(html)),
            js_content_en: report.js_content_en.as_ref().map(|js| self.sanitize_js_content(js)),
            created_at: report.created_at,
            sandbox_token: sandbox_token.clone(),
            chart_modules_content: chart_modules_content.map(String::from), // Sử dụng String::from thay vì to_string()
            complete_html_document: String::new(), // Will be populated below
        };
        
        // Generate complete HTML document and store it
        sandboxed_report.complete_html_document = self.generate_sandboxed_html_document(&sandboxed_report, None, chart_modules_content);
        
        println!("📄 ReportCreator: Complete HTML document generated for report {} ({} bytes)", 
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
        
        println!("🔄 ReportCreator: Regenerated HTML document for report {} with language {:?} ({} bytes)",
                sandboxed_report.id, language.unwrap_or("vi"), html_doc.len());
        
        html_doc
    }

    /// Sanitize HTML content for sandbox
    /// 
    /// Removes potentially dangerous HTML elements and attributes
    pub fn sanitize_html_content(&self, html: &str) -> String {
        // Basic HTML sanitization - remove dangerous elements
        let dangerous_patterns = [
            r"<script[^>]*>.*?</script>", // Remove inline scripts
            r"<iframe[^>]*>.*?</iframe>", // Remove nested iframes
            r"<object[^>]*>.*?</object>", // Remove objects
            r"<embed[^>]*>.*?</embed>",   // Remove embeds
            r"<applet[^>]*>.*?</applet>", // Remove applets
            r"javascript:", // Remove javascript: URLs
            r"on\w+\s*=", // Remove event handlers
        ];
        
        // Tối ưu: chỉ clone 1 lần nếu cần thiết
        let mut needs_sanitization = false;
        
        // Kiểm tra xem có cần sanitize không
        for pattern in dangerous_patterns.iter() {
            if let Ok(re) = regex::Regex::new(pattern) {
                if re.is_match(html) {
                    needs_sanitization = true;
                    break;
                }
            }
        }
        
        if !needs_sanitization {
            // Không có gì nguy hiểm, trả về clone đơn giản
            return html.to_string();
        }
        
        // Cần sanitize - clone và xử lý
        let mut sanitized = html.to_string();
        for pattern in dangerous_patterns.iter() {
            if let Ok(re) = regex::Regex::new(pattern) {
                // Dùng into_owned() để tránh to_string() thêm
                sanitized = re.replace_all(&sanitized, "").into_owned();
            }
        }
        
        sanitized
    }

    /// Sanitize CSS content for sandbox
    /// 
    /// Removes potentially dangerous CSS properties and expressions
    /// Enhanced to prevent CSS from affecting parent page
    pub fn sanitize_css_content(&self, css: &str) -> String {
        // Basic CSS sanitization - remove dangerous properties
        let dangerous_patterns = [
            r"expression\s*\(", // Remove CSS expressions
            r"javascript\s*:", // Remove javascript URLs in CSS
            r"@import", // Remove imports
            r"behavior\s*:", // Remove IE behaviors
            r"position\s*:\s*fixed", // Prevent fixed positioning that could escape iframe
            r"position\s*:\s*absolute", // Be careful with absolute positioning
            r"z-index\s*:\s*[0-9]{4,}", // Prevent extremely high z-index values
            r"!important\s*;", // Remove !important declarations that could override parent styles
        ];
        
        let mut sanitized = css.to_string();
        for pattern in dangerous_patterns.iter() {
            let re = regex::Regex::new(pattern).unwrap_or_else(|_| {
                regex::Regex::new("").unwrap() // Fallback empty regex
            });
            sanitized = re.replace_all(&sanitized, "").to_string();
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
    pub fn sanitize_js_content(&self, js: &str) -> String {
        // Basic JS sanitization - remove dangerous functions
        let dangerous_patterns = [
            r"eval\s*\(", // Remove eval calls
            r"Function\s*\(", // Remove Function constructor
            r"setTimeout\s*\(", // Remove setTimeout
            r"setInterval\s*\(", // Remove setInterval
            r"document\.write", // Remove document.write
            r"window\.location", // Remove location changes
            r"parent\.", // Remove parent access
            r"top\.", // Remove top access
        ];
        
        let mut sanitized = js.to_string();
        for pattern in dangerous_patterns.iter() {
            let re = regex::Regex::new(pattern).unwrap_or_else(|_| {
                regex::Regex::new("").unwrap() // Fallback empty regex
            });
            sanitized = re.replace_all(&sanitized, "/* SANITIZED */").to_string();
        }
        
        sanitized
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

        // Load HTML template from external file
        let template_path = "shared_components/view_iframe.html";
        let template_content = match std::fs::read_to_string(template_path) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("⚠️ ReportCreator: Failed to load template {}: {}. Using fallback.", template_path, e);
                // Fallback to minimal template if file loading fails
                include_str!("../../../../shared_components/view_iframe.html").to_string()
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

    /// Serve sandboxed report content for iframe
    /// 
    /// Returns sanitized HTML content for secure iframe embedding
    /// This method belongs to ReportCreator as it handles report content generation
    pub async fn serve_sandboxed_report(
        &self,
        state: &Arc<AppState>,
        report_id: i32,
        sandbox_token: &str,
        language: Option<&str>,
        chart_modules_content: Option<&str>
    ) -> Result<Response, Box<dyn StdError + Send + Sync>> {
        println!("🔒 ReportCreator: Serving sandboxed content for report {} with token {}", report_id, sandbox_token);
        
        // Fetch report from database
        let report_result = if report_id == -1 {
            self.fetch_and_cache_latest_report(state).await
        } else {
            self.fetch_and_cache_report_by_id(state, report_id).await
        };

        match report_result {
            Ok(Some(report)) => {
                // Create sandboxed version with complete HTML document
                let sandboxed_report = self.create_sandboxed_report(&report, chart_modules_content);
                
                // Verify sandbox token
                if sandboxed_report.sandbox_token != sandbox_token {
                    println!("❌ ReportCreator: Invalid sandbox token for report {}", report_id);
                    return Ok(Response::builder()
                        .status(StatusCode::FORBIDDEN)
                        .header("content-type", "text/plain")
                        .body(Body::from("Invalid sandbox token"))
                        .unwrap()
                        .into_response()
                    );
                }
                
                // Use the pre-generated complete HTML document from cache
                // If a specific language is requested and it's different from default (vi), regenerate
                let sandboxed_html = if let Some(lang) = language {
                    if lang != "vi" {
                        // Regenerate with specific language
                        self.regenerate_html_document(&sandboxed_report, Some(lang))
                    } else {
                        // Use cached default document
                        sandboxed_report.complete_html_document.clone()
                    }
                } else {
                    // Use cached default document
                    sandboxed_report.complete_html_document.clone()
                };
                
                println!("✅ ReportCreator: Serving HTML document for report {} with language {:?} ({} bytes)", 
                        report_id, language.unwrap_or("vi"), sandboxed_html.len());
                
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
                    .unwrap()
                    .into_response()
                )
            }
            Ok(None) => {
                println!("❌ ReportCreator: Report {} not found for sandboxing", report_id);
                Ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .header("content-type", "text/plain")
                    .body(Body::from("Report not found"))
                    .unwrap()
                    .into_response()
                )
            }
            Err(e) => {
                eprintln!("❌ ReportCreator: Database error serving sandboxed report {}: {}", report_id, e);
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
