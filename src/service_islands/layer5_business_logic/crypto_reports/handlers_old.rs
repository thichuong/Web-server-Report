//! Crypto Reports HTTP Request Handlers
//! 
//! This module contains all HTTP request handlers related to crypto reports functionality.
//! Based on archive_old_code/handlers/crypto.rs

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Serialize, Deserialize};
use std::{collections::HashMap, error::Error as StdError, sync::Arc, sync::atomic::Ordering};
use tera::Context;

// Import from current state - will be refactored when lower layers are implemented
use crate::state::AppState;

/// Report model - exactly from archive_old_code/models.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub id: i32,
    pub html_content: String,
    pub css_content: Option<String>,
    pub js_content: Option<String>,
    pub html_content_en: Option<String>,
    pub js_content_en: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Crypto Handlers
/// 
/// Contains all HTTP request handlers for crypto reports-related operations.
/// These handlers manage crypto report generation, PDF creation, and API interactions.
pub struct CryptoHandlers {
    // Component state will be added here as we implement lower layers
}

impl CryptoHandlers {
    /// Create a new CryptoHandlers instance
    pub fn new() -> Self {
        Self {
            // Initialize component state
        }
    }
    
    /// Health check for crypto handlers
    pub async fn health_check(&self) -> bool {
        // Verify handlers are functioning properly
        true // Will implement actual health checks
    }

    /// Helper function for template rendering
    /// 
    /// Directly from archive_old_code/handlers/crypto.rs::render_crypto_template
    pub async fn render_crypto_template(
        &self,
        tera: &tera::Tera, 
        template: &str,
        report: &Report,
        chart_modules_content: &str,
        additional_context: Option<HashMap<String, serde_json::Value>>
    ) -> Result<String, Box<dyn StdError + Send + Sync>> {
        let tera_clone = tera.clone();
        let template_str = template.to_string(); // Clone to owned string
        let report_clone = report.clone();
        let chart_content_clone = chart_modules_content.to_string();
        let additional_clone = additional_context.clone();
        
        let render_result = tokio::task::spawn_blocking(move || {
            let mut context = Context::new();
            context.insert("report", &report_clone);
            context.insert("chart_modules_content", &chart_content_clone);
            
            // Add additional context for different templates
            if let Some(extra) = additional_clone {
                for (key, value) in extra {
                    context.insert(&key, &value);
                }
            }
            
            // Common context for view templates
            if template_str.contains("view.html") {
                context.insert("current_route", "dashboard");
                context.insert("current_lang", "vi");
                context.insert("current_time", &chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string());
                let pdf_url = format!("/pdf-template/{}", report_clone.id);
                context.insert("pdf_url", &pdf_url);
            }
            
            // PDF template specific context
            if template_str.contains("pdf.html") {
                let created_display = (report_clone.created_at + chrono::Duration::hours(7)).format("%d-%m-%Y %H:%M").to_string();
                context.insert("created_at_display", &created_display);
            }

            tera_clone.render(&template_str, &context)
        }).await;
        
        match render_result {
            Ok(Ok(html)) => Ok(html),
            Ok(Err(e)) => {
                eprintln!("Template render error: {:#?}", e);
                let mut src = e.source();
                while let Some(s) = src {
                    eprintln!("Template render error source: {:#?}", s);
                    src = s.source();
                }
                Err(format!("Template render error: {}", e).into())
            }
            Err(e) => {
                eprintln!("Task join error: {:#?}", e);
                Err(format!("Task join error: {}", e).into())
            }
        }
    }

    /// Create cached response
    /// 
    /// From archive_old_code/handlers/crypto.rs::create_cached_response
    pub fn create_cached_response(&self, html: String, cache_status: &str) -> Response {
        Response::builder()
            .status(StatusCode::OK)
            .header("cache-control", "public, max-age=15")
            .header("content-type", "text/html; charset=utf-8")
            .header("x-cache", cache_status)
            .body(html)
            .unwrap()
            .into_response()
    }

    /// Fetch and cache latest report 
    /// 
    /// From archive_old_code/handlers/crypto.rs::fetch_and_cache_latest_report
    pub async fn fetch_and_cache_latest_report(
        &self,
        state: &Arc<AppState>
    ) -> Result<Option<Report>, Box<dyn StdError + Send + Sync>> {
        // TODO: Implement real database query like archive_old_code
        // let report = sqlx::query_as::<_, Report>(
        //     "SELECT id, html_content, css_content, js_content, html_content_en, js_content_en, created_at FROM crypto_report ORDER BY created_at DESC LIMIT 1",
        // ).fetch_optional(&state.db).await?;
        
        // Simple mock data for Template Engine - data from database only
        let mock_report = Report {
            id: 1,
            html_content: "<!-- HTML content from database -->".to_string(),
            css_content: Some("/* CSS content from database */".to_string()),
            js_content: Some("// JS content from database".to_string()),
            html_content_en: Some("<!-- English HTML from database -->".to_string()),
            js_content_en: Some("// English JS from database".to_string()),
            created_at: chrono::Utc::now(),
        };
        
        Ok(Some(mock_report))
    }

    /// Get chart modules content (mock implementation)
    /// 
    /// Based on archive_old_code/utils.rs::get_chart_modules_content
    pub async fn get_chart_modules_content(&self) -> String {
        // Mock implementation - would normally read from shared_assets/js/chart_modules/
        r#"
        // ==================== Chart Modules ====================
        console.log('📊 Loading Chart Modules via Service Islands');
        
        // Mock Chart.js integration
        window.ChartModules = {
            initializeBitcoinChart: function() {
                console.log('📈 Initializing Bitcoin price chart...');
                // Chart initialization would go here
            },
            
            initializeMarketCapChart: function() {
                console.log('💰 Initializing market cap chart...');
                // Chart initialization would go here  
            },
            
            updateChartData: function() {
                console.log('🔄 Updating chart data...');
                // Real-time data update would go here
            }
        };
        
        // Auto-initialize on DOM ready
        document.addEventListener('DOMContentLoaded', function() {
            if (window.ChartModules) {
                window.ChartModules.initializeBitcoinChart();
                window.ChartModules.initializeMarketCapChart();
                
                // Update charts every 30 seconds
                setInterval(window.ChartModules.updateChartData, 30000);
            }
        });
        
        console.log('✅ Chart modules loaded successfully');
        // ==================== End Chart Modules ====================
        "#.to_string()
    }
    
    /// Crypto Index handler - Main crypto dashboard
    /// 
    /// Originally from src/handlers/crypto.rs::crypto_index
    /// Implements multi-layer caching and template rendering for crypto dashboard
    /// Uses "crypto/routes/reports/view.html" template like the original implementation
    pub async fn crypto_index(&self) -> Result<String, Box<dyn StdError + Send + Sync>> {
        println!("🚀 CryptoHandlers::crypto_index - Processing crypto dashboard request");
        
        // TODO: Implement proper state access to get Tera engine
        // For now, this is a placeholder that shows the correct structure
        // The full implementation will need:
        // 1. Access to Tera engine from Service Islands
        // 2. Proper caching logic from archive code
        // 3. Real database integration
        
        Err("Template engine not yet integrated in Service Islands. Need Tera engine access.".into())
    }
    
    /// Crypto Index handler with proper Tera template engine
    /// 
    /// This implements the correct approach using Tera engine like archive_old_code/handlers/crypto.rs
    pub async fn crypto_index_with_tera(&self, state: &Arc<AppState>) -> Result<String, Box<dyn StdError + Send + Sync>> {
        println!("🚀 CryptoHandlers::crypto_index_with_tera - Processing with real template engine");
        
        // Step 1: Fetch report data (mock for now, would be from cache/DB)
        let report = self.fetch_and_cache_latest_report(state).await?
            .ok_or("No report found")?;
        
        // Step 2: Get chart modules content
        let chart_modules_content = self.get_chart_modules_content().await;
        
        // Step 3: Use real Tera template engine like archive code
        let html = self.render_crypto_template(
            &state.tera,  // THIS is the real Tera engine access we need
            "crypto/routes/reports/view.html",
            &report,
            &chart_modules_content,
            None
        ).await?;
        
        println!("✅ CryptoHandlers::crypto_index_with_tera - Template rendered successfully");
        Ok(html)
    }

    // NOTE: Crypto handlers implementation following archive_old_code/handlers/crypto.rs
    // Key requirements:
    // 1. MUST use Tera template engine - NO manual HTML creation
    // 2. MUST use "crypto/routes/reports/view.html" template 
    // 3. Template variables: {{ report.css_content }}, {{ report.js_content }}, {{ chart_modules_content }}
    // 4. Implement L1/L2 caching logic like original
    // 5. Parallel chart modules fetching
    // 
    // Current status: Template engine access needed from Service Islands architecture

}
            html_content: r#"
                <!-- HTML content từ database - sẽ được inject vào template -->
                <div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
                    <div class="bg-white dark:bg-gray-800 rounded-lg shadow-lg p-6">
                        <h2 class="text-2xl font-bold mb-4 text-gray-800 dark:text-white">
                            <i class="fas fa-chart-line text-green-500 mr-2"></i>
                            Phân Tích Giá Bitcoin
                        </h2>
                        <div class="space-y-4">
                            <div class="flex justify-between items-center">
                                <span class="text-gray-600 dark:text-gray-300">Giá hiện tại:</span>
                                <span class="text-2xl font-bold text-green-600" id="btc-current-price">$43,250</span>
                            </div>
                        </div>
                    </div>
                </div>
            "#.to_string(),
            css_content: Some(r#"
                /* CSS từ database - sẽ được inject vào template */
                .price-up { color: #10b981; animation: pulse 2s infinite; }
                .price-down { color: #ef4444; animation: pulse 2s infinite; }
                @keyframes pulse { 0%, 100% { opacity: 1; } 50% { opacity: 0.7; } }
            "#.to_string()),
            js_content: Some(r#"
                // JS từ database - sẽ được inject vào template
                console.log('🚀 Service Islands Crypto Dashboard Loaded');
                document.addEventListener('DOMContentLoaded', function() {
                    console.log('✅ Crypto dashboard initialized from database JS');
                });
            "#.to_string()),
            html_content_en: Some("<!-- English content from database -->".to_string()),
            js_content_en: Some("// English JS content from database".to_string()),
            created_at: chrono::Utc::now(),
        }
    }
    
    // NOTE: Crypto handlers implementation following archive_old_code/handlers/crypto.rs
    // Key requirements:
    // 1. MUST use Tera template engine - NO manual HTML creation
    // 2. MUST use "crypto/routes/reports/view.html" template 
    // 3. Template variables: {{ report.css_content }}, {{ report.js_content }}, {{ chart_modules_content }}
    // 4. Implement L1/L2 caching logic like original
    // 5. Parallel chart modules fetching
    // 
    // Current status: Template engine access needed from Service Islands architecture

}
            "#.to_string()),
            html_content_en: Some("<!-- English content from database -->".to_string()),
            js_content_en: Some("// English JS content from database".to_string()),
            created_at: chrono::Utc::now(),
        }
    }
    }
    
    /// Get mock chart modules content
    async fn get_mock_chart_modules(&self) -> String {
        r#"
        // Mock Chart Modules for Service Islands Demo
        const ChartModules = {
            initializePriceChart: function() {
                console.log('📈 Initializing price chart...');
            },
            initializeVolumeChart: function() {
                console.log('📊 Initializing volume chart...');
            },
            updateMarketData: function() {
                console.log('🔄 Updating market data...');
            }
        };
        
        // Auto-initialize charts
        document.addEventListener('DOMContentLoaded', function() {
            ChartModules.initializePriceChart();
            ChartModules.initializeVolumeChart();
            
            // Update data every 30 seconds
            setInterval(ChartModules.updateMarketData, 30000);
        });
        "#.to_string()
    }
    
    /// Render crypto template using Tera (mock implementation)
    /// 
    /// In full implementation, this would use actual Tera instance from Service Islands
    async fn render_crypto_template_with_tera(
        &self,
        template: &str,
        report: &CryptoReport,
        chart_modules_content: &str,
        additional_context: Option<HashMap<String, serde_json::Value>>
    ) -> Result<String, Box<dyn StdError + Send + Sync>> {
        println!("🎨 Rendering template: {}", template);
        
        // For now, create a mock rendered HTML that simulates what the real template would produce
        // In full implementation, this would use actual Tera template engine
        let html = format!(r#"
        <!DOCTYPE html>
        <html lang="vi">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>📊 Dashboard Toàn Cảnh Thị Trường - Service Islands</title>
            <script src="https://cdn.tailwindcss.com"></script>
            <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700;800&display=swap" rel="stylesheet">
            <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.4.2/css/all.min.css">
            <link rel="stylesheet" href="/shared_assets/css/colors.css">
            <link rel="stylesheet" href="/shared_assets/css/style.css">
            <link rel="stylesheet" href="/shared_assets/css/chart.css">
            <link rel="stylesheet" href="/shared_assets/css/report.css">
            
            <style>
                /* Report-specific CSS from database */
                {}
                
                /* Service Islands enhancements */
                .service-islands-header {{
                    background: linear-gradient(135deg, #1e3a8a 0%, #3b82f6 100%);
                    color: white;
                    padding: 15px;
                    text-align: center;
                    margin-bottom: 20px;
                    border-radius: 8px;
                }}
                
                .template-info {{
                    background: #f8f9fa;
                    padding: 10px;
                    border-left: 4px solid #007bff;
                    margin: 10px 0;
                    font-size: 0.9em;
                }}
            </style>
        </head>
        <body>
            <div class="service-islands-header">
                <h1>🏝️ Service Islands Architecture</h1>
                <p>Template: {} | Report ID: {} | Generated: {}</p>
            </div>
            
            <div class="template-info">
                <strong>🎨 Template Rendering:</strong> {} được render thành công bởi Crypto Reports Island
            </div>
            
            <!-- Report Content from Database -->
            <div class="report-content">
                {}
            </div>
            
            <div class="chart-section">
                <h3>📊 Interactive Charts</h3>
                <div id="crypto-charts">
                    <p>Chart modules đã được load và sẵn sàng hiển thị dữ liệu real-time</p>
                </div>
            </div>
            
            <div class="actions">
                <a href="/crypto_reports_list" class="btn btn-primary">📋 View All Reports</a>
                <a href="/pdf-template/{}" class="btn btn-secondary">� PDF Template</a>
            </div>
            
            <footer style="margin-top: 40px; text-align: center; color: #666;">
                <p>🏝️ Service Islands | Layer 5: Business Logic | Crypto Reports Island</p>
                <p>Template: {} | Cache: Service Islands Ready</p>
            </footer>
            
            <script>
                // Chart modules content
                {}
                
                // Report-specific JavaScript
                {}
                
                console.log('✅ Service Islands Crypto Dashboard loaded successfully');
                console.log('📊 Template: {}');
                console.log('🔄 Report ID: {}');
            </script>
        </body>
        </html>
        "#, 
            report.css_content.as_deref().unwrap_or(""),
            template,
            report.id,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
            template,
            report.html_content,
            report.id,
            template,
            chart_modules_content,
            report.js_content.as_deref().unwrap_or(""),
            template,
            report.id
        );
        
        Ok(html)
    }

    // NOTE: Crypto handlers implementation following archive_old_code/handlers/crypto.rs
    // Key requirements:
    // 1. MUST use Tera template engine - NO manual HTML creation
    // 2. MUST use "crypto/routes/reports/view.html" template 
    // 3. Template variables: {{ report.css_content }}, {{ report.js_content }}, {{ chart_modules_content }}
    // 4. Implement L1/L2 caching logic like original
    // 5. Parallel chart modules fetching
    // 
    // Current status: Template engine access needed from Service Islands architecture
