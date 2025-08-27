//! Report Creator Component
//! 
//! This component handles report creation business logic for crypto reports,
//! including report data fetching, processing, and chart modules management.

use serde::{Serialize, Deserialize};
use sqlx::{FromRow};
use std::sync::Arc;

// Import from current state - will be refactored when lower layers are implemented
use crate::state::AppState;
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
}

/// Report summary for listing - from archive_old_code/models.rs
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ReportSummary {
    pub id: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Report list item with formatted dates - from archive_old_code/models.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportListItem {
    pub id: i32,
    pub created_date: String,
    pub created_time: String,
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
    /// This method sanitizes content and generates a security token.
    pub fn create_sandboxed_report(&self, report: &Report) -> SandboxedReport {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        // Generate security token based on report ID and creation time
        let mut hasher = DefaultHasher::new();
        report.id.hash(&mut hasher);
        report.created_at.hash(&mut hasher);
        let sandbox_token = format!("sb_{:x}", hasher.finish());
        
        println!("🔒 ReportCreator: Generated sandbox token for report {}: {}", report.id, sandbox_token);
        
        SandboxedReport {
            id: report.id,
            html_content: self.sanitize_html_content(&report.html_content),
            css_content: report.css_content.as_ref().map(|css| self.sanitize_css_content(css)),
            js_content: report.js_content.as_ref().map(|js| self.sanitize_js_content(js)),
            html_content_en: report.html_content_en.as_ref().map(|html| self.sanitize_html_content(html)),
            js_content_en: report.js_content_en.as_ref().map(|js| self.sanitize_js_content(js)),
            created_at: report.created_at,
            sandbox_token,
        }
    }

    /// Sanitize HTML content for sandbox
    /// 
    /// Removes potentially dangerous HTML elements and attributes
    fn sanitize_html_content(&self, html: &str) -> String {
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
        
        let mut sanitized = html.to_string();
        for pattern in dangerous_patterns.iter() {
            let re = regex::Regex::new(pattern).unwrap_or_else(|_| {
                regex::Regex::new("").unwrap() // Fallback empty regex
            });
            sanitized = re.replace_all(&sanitized, "").to_string();
        }
        
        sanitized
    }

    /// Sanitize CSS content for sandbox
    /// 
    /// Removes potentially dangerous CSS properties and expressions
    /// Enhanced to prevent CSS from affecting parent page
    fn sanitize_css_content(&self, css: &str) -> String {
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
            "/* CSS isolated within iframe sandbox */\n.sandboxed-report-content {{\n{}\n}}",
            sanitized
        );
        
        wrapped_css
    }

    /// Sanitize JavaScript content for sandbox
    /// 
    /// Applies basic JavaScript sanitization for sandbox environment
    fn sanitize_js_content(&self, js: &str) -> String {
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
    pub fn generate_sandboxed_html_document(&self, sandboxed_report: &SandboxedReport, language: Option<&str>) -> String {
        let lang = language.unwrap_or("vi");
        
        // Create owned strings to avoid borrow checker issues
        let empty_string = String::new();
        let default_html = &sandboxed_report.html_content;
        let default_js = &empty_string;
        let default_css = &empty_string;
        
        let (html_content, js_content) = if lang == "en" {
            let html_en = sandboxed_report.html_content_en.as_ref().unwrap_or(default_html);
            let js_en = sandboxed_report.js_content_en.as_ref()
                .unwrap_or_else(|| sandboxed_report.js_content.as_ref().unwrap_or(default_js));
            (html_en, js_en)
        } else {
            let js_vi = sandboxed_report.js_content.as_ref().unwrap_or(default_js);
            (default_html, js_vi)
        };

        // Get CSS content - this is now isolated inside the iframe
        let css_content = sandboxed_report.css_content.as_ref().unwrap_or(default_css);

        format!(r#"<!DOCTYPE html>
<html lang="{lang}">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Sandboxed Report #{}</title>
    
    <!-- Base styling for sandboxed content -->
    <style>
        /* Reset and base styles for iframe content */
        body {{
            margin: 0;
            padding: 20px;
            font-family: 'Inter', -apple-system, BlinkMacSystemFont, sans-serif;
            line-height: 1.6;
            color: #333;
            background: #fff;
        }}
        
        /* Common elements styling */
        h1, h2, h3, h4, h5, h6 {{
            margin-top: 0;
            margin-bottom: 1rem;
            font-weight: 600;
        }}
        
        p {{
            margin-bottom: 1rem;
        }}
        
        img {{
            max-width: 100%;
            height: auto;
        }}
        
        table {{
            width: 100%;
            border-collapse: collapse;
            margin-bottom: 1rem;
        }}
        
        th, td {{
            padding: 0.5rem;
            text-align: left;
            border-bottom: 1px solid #ddd;
        }}
        
        /* Report-specific CSS (isolated from main page) */
        {css_content}
    </style>
</head>
<body>
    <div id="report-content" class="sandboxed-report-content">
        {html_content}
    </div>
    
    <script>
        // Sandboxed environment - limited API access
        (function() {{
            'use strict';
            
            // Disable dangerous APIs
            if (typeof eval !== 'undefined') eval = undefined;
            if (typeof Function !== 'undefined') Function = undefined;
            
            // Prevent access to parent window
            try {{
                if (window.parent && window.parent !== window) {{
                    // Only allow safe communication
                    window.parent.postMessage = undefined;
                }}
            }} catch(e) {{
                // Cross-origin restriction - this is expected and good
            }}
            
            // Report-specific JavaScript in sandboxed context
            try {{
                {js_content}
            }} catch(e) {{
                console.warn('Sandboxed script error:', e);
            }}
            
            // Auto-notify parent about content height changes
            function notifyParentAboutHeight() {{
                try {{
                    const height = Math.max(
                        document.body.scrollHeight,
                        document.body.offsetHeight,
                        document.documentElement.clientHeight,
                        document.documentElement.scrollHeight,
                        document.documentElement.offsetHeight
                    );
                    
                    // Safe postMessage to parent for height adjustment
                    if (window.parent && window.parent !== window) {{
                        window.parent.postMessage({{
                            type: 'iframe-height',
                            height: height + 50 // Add some padding
                        }}, '*');
                    }}
                }} catch(e) {{
                    // Cross-origin restrictions - expected
                }}
            }}
            
            // Initial height notification
            setTimeout(notifyParentAboutHeight, 100);
            
            // Listen for content changes
            if (typeof MutationObserver !== 'undefined') {{
                const observer = new MutationObserver(notifyParentAboutHeight);
                observer.observe(document.body, {{
                    childList: true,
                    subtree: true,
                    attributes: true
                }});
            }}
            
            // Fallback periodic height check
            setInterval(notifyParentAboutHeight, 2000);
        }})();
    </script>
</body>
</html>"#, 
            sandboxed_report.id,
            css_content = css_content,
            html_content = html_content,
            js_content = js_content
        )
    }
}
