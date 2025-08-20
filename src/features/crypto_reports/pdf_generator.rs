//! PDF Generator - PDF report generation and template management
//!
//! Handles PDF template rendering, print optimization,
//! and report formatting for crypto reports.

use crate::features::shared_components::SharedComponents;
use crate::features::cache_system::CacheSystem;
use crate::models::Report;
use serde_json::json;
use std::collections::HashMap;
use std::error::Error as StdError;
use std::sync::Arc;

/// PDF generation component
pub struct PdfGenerator {
    shared_components: Option<Arc<SharedComponents>>,
    cache_system: Option<Arc<CacheSystem>>,
}

impl PdfGenerator {
    pub fn new(shared_components: &SharedComponents, cache_system: &CacheSystem) -> Self {
        Self {
            shared_components: Some(Arc::new(shared_components.clone())),
            cache_system: Some(Arc::new(cache_system.clone())),
        }
    }

    /// Initialize PDF generator
    pub async fn initialize(&self) -> Result<(), Box<dyn StdError + Send + Sync>> {
        println!("📄 Initializing PDF Generator component");
        Ok(())
    }

    /// Generate PDF template HTML
    pub async fn generate_pdf_template(&self, report_id: i32) -> Result<String, Box<dyn StdError + Send + Sync>> {
        // Try L1 cache first
        if let Some(cache) = &self.cache_system {
            let cache_key = format!("pdf_template:{}", report_id);
            if let Ok(Some(cached_html)) = cache.get::<String>(&cache_key).await {
                println!("🔥 PDF Template Cache HIT for report {}", report_id);
                return Ok(cached_html);
            }
        }

        // Cache miss - generate PDF template
        let report = self.fetch_report_for_pdf(report_id).await?;
        let html = self.render_pdf_template(&report).await?;

        // Cache the rendered HTML
        if let Some(cache) = &self.cache_system {
            let cache_key = format!("pdf_template:{}", report_id);
            if let Err(e) = cache.set_with_ttl(&cache_key, &html, 1800).await { // 30 minutes TTL
                eprintln!("⚠️ Failed to cache PDF template: {}", e);
            } else {
                println!("💾 Cached PDF template for report {} (TTL: 30min)", report_id);
            }
        }

        Ok(html)
    }

    /// Render PDF template with report data
    async fn render_pdf_template(&self, report: &Report) -> Result<String, Box<dyn StdError + Send + Sync>> {
        // Create PDF-specific context
        let mut context = self.create_pdf_context(report);
        
        // Add PDF-specific metadata
        context.insert("print_optimized".to_string(), json!(true));
        context.insert("page_break_enabled".to_string(), json!(true));
        context.insert("high_resolution".to_string(), json!(true));

        // Generate PDF template HTML
        let html = self.build_pdf_html(report, &context).await?;
        
        Ok(html)
    }

    /// Create PDF-specific template context
    fn create_pdf_context(&self, report: &Report) -> HashMap<String, serde_json::Value> {
        let mut context = HashMap::new();
        
        // Format date for PDF display  
        let created_display = (report.created_at + chrono::Duration::hours(7))
            .format("%d-%m-%Y %H:%M")
            .to_string();
            
        context.insert("report".to_string(), json!(report));
        context.insert("created_at_display".to_string(), json!(created_display));
        context.insert("current_lang".to_string(), json!("vi"));
        context.insert("pdf_mode".to_string(), json!(true));
        context.insert("report_id".to_string(), json!(report.id));
        
        // PDF-specific styling
        context.insert("print_styles".to_string(), json!({
            "page_size": "A4",
            "margins": "1cm",
            "font_size": "12pt",
            "line_height": "1.4"
        }));

        context
    }

    /// Build complete PDF HTML document
    async fn build_pdf_html(&self, report: &Report, context: &HashMap<String, serde_json::Value>) -> Result<String, Box<dyn StdError + Send + Sync>> {
        // Get chart modules for PDF
        let chart_modules = self.get_chart_modules_for_pdf().await;
        
        // Build PDF HTML template
        let html = format!(r#"<!DOCTYPE html>
<html lang="vi">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Báo cáo #{} - Dashboard Toàn Cảnh Thị Trường</title>
    <script src="https://cdn.tailwindcss.com"></script>
    <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700;800&display=swap" rel="stylesheet">
    <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.4.2/css/all.min.css">
    
    <!-- PDF-specific styles -->
    <link rel="stylesheet" href="/shared_assets/css/colors.css">
    <link rel="stylesheet" href="/shared_assets/css/style.css">
    <link rel="stylesheet" href="/shared_assets/css/chart.css">
    <link rel="stylesheet" href="/shared_assets/css/pdf-template.css">
    <link rel="stylesheet" href="/shared_assets/css/report.css">

    {}

    <style>
        @media print {{
            .no-print {{ display: none !important; }}
            .print-break {{ page-break-before: always; }}
            body {{ font-size: 12pt; line-height: 1.4; }}
        }}
        .pdf-template {{ background: white; }}
        .a4-container {{ 
            max-width: 21cm; 
            margin: 0 auto; 
            padding: 1cm;
            background: white;
        }}
    </style>
</head>
<body class="pdf-template antialiased">
    
    <!-- Language Toggle (hidden in print) -->
    <div class="no-print mb-4 text-center">
        <button class="lang-btn bg-blue-600 text-white px-3 py-1 rounded mr-2" data-lang="vi">VI</button>
        <button class="lang-btn bg-gray-400 text-white px-3 py-1 rounded" data-lang="en">EN</button>
    </div>

    <!-- Print Controls (hidden in print) -->
    <div class="print-controls no-print text-center mb-4">
        <button onclick="window.print()" class="bg-blue-600 text-white px-4 py-2 rounded-lg hover:bg-blue-700 transition-colors mr-2">
            <i class="fas fa-print mr-2"></i>In Báo cáo
        </button>
        <button onclick="window.close()" class="bg-gray-600 text-white px-4 py-2 rounded-lg hover:bg-gray-700 transition-colors">
            <i class="fas fa-times mr-2"></i>Đóng
        </button>
    </div>
    
    <div class="mobile-wrapper">
        <div class="a4-container">
            <header class="text-center mb-8">
                <h1 class="font-extrabold mb-4 bg-clip-text text-transparent bg-gradient-to-r from-blue-600 to-green-500">
                    <span id="title-vi" style="display: block;">Toàn Cảnh Thị Trường Tiền Mã Hóa</span>
                    <span id="title-en" style="display: none;">Comprehensive Cryptocurrency Market Overview</span>
                </h1>
                <p class="text-base text-gray-600 max-w-full mx-auto">
                    <span id="subtitle-vi" style="display: block;">
                        Báo cáo #{} (tạo lúc {})
                    </span>
                    <span id="subtitle-en" style="display: none;">
                        Report #{} (created at {})
                    </span>
                </p>
            </header>

            <main id="report-container" class="max-w-none" data-report-id="{}">
                <!-- Vietnamese report content -->
                <div id="report-content-vi" style="display: block;">
                    {}
                </div>

                <!-- English report content (if available) -->
                {}
            </main>
        </div>
    </div>
    
    <!-- Chart modules injected from server -->
    <script>
        {}
    </script>
    
    <!-- Report-specific JavaScript -->
    {}
    
    <!-- PDF-specific scripts -->
    <script src="/shared_components/report-initializer.js" defer></script>
    <script src="/shared_components/pdf-template.js" defer></script>
    <script src="/shared_components/core/language-toggle.js" defer></script>

    <script>
        // PDF-specific JavaScript
        (function() {{
            const STORAGE_KEY = 'app_lang';
            function getLang() {{ return localStorage.getItem(STORAGE_KEY) || 'vi'; }}
            function setLang(lang) {{ localStorage.setItem(STORAGE_KEY, lang); applyLang(lang); }}
            function applyLang(lang) {{
                // Toggle language content visibility
                const elements = [
                    ['report-content-en', 'report-content-vi'],
                    ['title-en', 'title-vi'],
                    ['subtitle-en', 'subtitle-vi']
                ];
                
                elements.forEach(([en, vi]) => {{
                    const enEl = document.getElementById(en);
                    const viEl = document.getElementById(vi);
                    if (enEl) enEl.style.display = (lang === 'en') ? 'block' : 'none';
                    if (viEl) viEl.style.display = (lang === 'vi') ? 'block' : 'none';
                }});
            }}

            // Language toggle event handlers
            document.addEventListener('click', function(e) {{
                const btn = e.target.closest && e.target.closest('.lang-btn');
                if (!btn) return;
                const lang = btn.getAttribute('data-lang');
                if (lang) setLang(lang);
            }});

            document.addEventListener('DOMContentLoaded', function() {{
                applyLang(getLang());
            }});
        }})();
    </script>
</body>
</html>"#,
            report.id,
            // CSS content from report
            report.css_content.as_deref().map(|css| format!("<style>{}</style>", css)).unwrap_or_default(),
            report.id,
            context.get("created_at_display").and_then(|v| v.as_str()).unwrap_or("N/A"),
            report.id,
            context.get("created_at_display").and_then(|v| v.as_str()).unwrap_or("N/A"),
            report.id,
            // Main HTML content
            report.html_content,
            // English content if available
            report.html_content_en.as_deref().map(|content| {
                format!(r#"<div id="report-content-en" style="display: none;">{}</div>"#, content)
            }).unwrap_or_default(),
            // Chart modules
            chart_modules,
            // Report-specific JavaScript
            report.js_content.as_deref().map(|js| format!("<script>{}</script>", js)).unwrap_or_default()
        );

        Ok(html)
    }

    /// Get chart modules optimized for PDF
    async fn get_chart_modules_for_pdf(&self) -> String {
        // PDF-optimized chart modules
        r#"
// PDF-optimized chart modules
console.log('PDF Chart modules loaded');

// Chart.js configuration for PDF printing
if (typeof Chart !== 'undefined') {
    Chart.defaults.animation = false; // Disable animations for PDF
    Chart.defaults.responsive = true;
    Chart.defaults.maintainAspectRatio = false;
}

// Initialize charts for PDF
function initializePdfCharts() {
    console.log('Initializing charts for PDF generation');
    // Chart initialization logic would go here
}

// PDF-specific chart utilities
window.pdfCharts = {
    initialize: initializePdfCharts,
    optimizeForPrint: function() {
        // Optimize charts for printing
        console.log('Optimizing charts for print');
    }
};

// Auto-initialize when DOM is ready
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', initializePdfCharts);
} else {
    initializePdfCharts();
}
"#.to_string()
    }

    /// Fetch report data for PDF generation
    async fn fetch_report_for_pdf(&self, report_id: i32) -> Result<Report, Box<dyn StdError + Send + Sync>> {
        // Try cache first if available
        if let Some(cache) = &self.cache_system {
            let cache_key = format!("crypto_report:{}", report_id);
            if let Ok(Some(cached_report)) = cache.get::<Report>(&cache_key).await {
                println!("🔥 Report Cache HIT for PDF generation (report {})", report_id);
                return Ok(cached_report);
            }
        }

        // Mock report for now - TODO: Replace with actual database integration
        let report = self.create_mock_report(report_id);
        
        // Cache the report if cache is available
        if let Some(cache) = &self.cache_system {
            let cache_key = format!("crypto_report:{}", report_id);
            if let Err(e) = cache.set(&cache_key, &report).await {
                eprintln!("⚠️ Failed to cache report for PDF: {}", e);
            }
        }
        
        Ok(report)
    }

    /// Create mock report for testing
    fn create_mock_report(&self, report_id: i32) -> Report {
        Report {
            id: report_id,
            html_content: format!(r#"
                <section class="report-section">
                    <h2>Báo Cáo Thị Trường Crypto #{}</h2>
                    <div class="market-overview">
                        <p>Thị trường tiền mã hóa hôm nay cho thấy những biến động tích cực với Bitcoin duy trì mức giá ổn định.</p>
                    </div>
                    <div class="key-metrics">
                        <div class="metric-item">
                            <h3>Bitcoin (BTC)</h3>
                            <p class="price">$45,230</p>
                            <p class="change positive">+2.34%</p>
                        </div>
                        <div class="metric-item">
                            <h3>Ethereum (ETH)</h3>
                            <p class="price">$3,123</p>
                            <p class="change positive">+1.87%</p>
                        </div>
                    </div>
                </section>
            "#, report_id),
            html_content_en: Some(format!(r#"
                <section class="report-section">
                    <h2>Crypto Market Report #{}</h2>
                    <div class="market-overview">
                        <p>Today's cryptocurrency market shows positive movements with Bitcoin maintaining stable price levels.</p>
                    </div>
                    <div class="key-metrics">
                        <div class="metric-item">
                            <h3>Bitcoin (BTC)</h3>
                            <p class="price">$45,230</p>
                            <p class="change positive">+2.34%</p>
                        </div>
                        <div class="metric-item">
                            <h3>Ethereum (ETH)</h3>
                            <p class="price">$3,123</p>
                            <p class="change positive">+1.87%</p>
                        </div>
                    </div>
                </section>
            "#, report_id)),
            css_content: Some(r#"
                .report-section { margin-bottom: 2rem; }
                .market-overview { padding: 1rem; background: #f8f9fa; border-radius: 8px; }
                .key-metrics { display: grid; grid-template-columns: repeat(auto-fit, minmax(250px, 1fr)); gap: 1rem; margin-top: 1rem; }
                .metric-item { padding: 1rem; border: 1px solid #dee2e6; border-radius: 8px; text-align: center; }
                .price { font-size: 1.5rem; font-weight: bold; color: #333; }
                .change.positive { color: #28a745; }
                .change.negative { color: #dc3545; }
            "#.to_string()),
            js_content: Some(r#"
                console.log('Report JavaScript loaded');
                document.addEventListener('DOMContentLoaded', function() {
                    console.log('Report content initialized');
                });
            "#.to_string()),
            js_content_en: None,
            created_at: chrono::Utc::now() - chrono::Duration::days(1),
        }
    }

    /// Check if PDF generator is healthy
    pub async fn is_healthy(&self) -> bool {
        // Simple health check - could be expanded
        true
    }
}

impl Default for PdfGenerator {
    fn default() -> Self {
        Self {
            shared_components: None,
            cache_system: None,
        }
    }
}
