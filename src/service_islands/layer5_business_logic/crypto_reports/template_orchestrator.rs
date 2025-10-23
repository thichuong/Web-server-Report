//! Template Orchestrator Component
//! 
//! This component handles all template rendering operations for crypto reports,
//! including context preparation, chart modules injection, and Tera integration.
//! Follows Service Islands Architecture Layer 5 patterns.

use std::{collections::HashMap, error::Error as StdError, io::Write};
use tera::Context;
use flate2::{Compression, write::GzEncoder};

// Import from our specialized components
use super::report_creator::{Report, ReportCreator};

/// Template Context Data
/// 
/// Structured container for all template rendering context data
#[derive(Debug, Clone)]
pub struct TemplateContext {
    pub report: Report,
    pub chart_modules_content: String,
    pub current_route: String,
    pub current_lang: String,
    pub current_time: String,
    pub pdf_url: String,
    pub additional_context: Option<HashMap<String, serde_json::Value>>,
}

/// Template Orchestrator
/// 
/// Manages all template rendering operations for crypto reports.
/// Separates template logic from HTTP handlers following Layer 5 architecture.
pub struct TemplateOrchestrator {
    /// Reference to ReportCreator for data operations
    pub report_creator: ReportCreator,
}

impl TemplateOrchestrator {
    /// Create a new TemplateOrchestrator
    pub fn new(report_creator: ReportCreator) -> Self {
        Self {
            report_creator,
        }
    }
    
    /// Health check for template orchestrator
    pub async fn health_check(&self) -> bool {
        // Verify template orchestrator is functioning properly
        self.report_creator.health_check().await
    }

    /// Compress HTML content using gzip
    /// 
    /// Compresses the rendered HTML content to reduce file size and improve transfer speed
    fn compress_html(&self, html: &str) -> Result<Vec<u8>, Box<dyn StdError + Send + Sync>> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(html.as_bytes())?;
        let compressed_data = encoder.finish()?;
        
        let original_size = html.len();
        let compressed_size = compressed_data.len();
        let compression_ratio = (1.0 - (compressed_size as f64 / original_size as f64)) * 100.0;
        
        println!("🗜️  TemplateOrchestrator: Compression completed - Original: {}KB, Compressed: {}KB, Ratio: {:.1}%", 
                 original_size / 1024, 
                 compressed_size / 1024, 
                 compression_ratio);
        
        Ok(compressed_data)
    }

    /// Prepare template context for crypto reports
    /// 
    /// Builds complete template context with all necessary data for rendering
    /// Enhanced to accept pre-loaded chart_modules_content for optimal performance
    /// Now includes sandbox token generation for iframe security
    pub async fn prepare_crypto_report_context(
        &self,
        report: &Report,
        template_type: &str,
        chart_modules_content: Option<String>, // THÊM THAM SỐ NÀY
        additional_context: Option<HashMap<String, serde_json::Value>>
    ) -> Result<TemplateContext, Box<dyn StdError + Send + Sync>> {
        println!("🎨 TemplateOrchestrator: Preparing context for template type: {}", template_type);
        
        // Sử dụng chart_modules_content được truyền vào, hoặc fetch từ ReportCreator nếu không có
        let chart_modules_content = match chart_modules_content {
            Some(content) => {
                println!("✅ TemplateOrchestrator: Sử dụng chart modules đã được pre-load");
                content
            }
            None => {
                println!("🔄 TemplateOrchestrator: Fallback - đọc chart modules từ file");
                self.report_creator.get_chart_modules_content().await
            }
        };
        
        // Generate sandbox token for iframe security
        let sandboxed_report = self.report_creator.create_sandboxed_report(report, Some(&chart_modules_content));
        
        // Prepare basic context
        let current_time = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();
        let pdf_url = format!("/crypto_report/{}/pdf", report.id);
        
        let mut context = TemplateContext {
            report: report.clone(), // Cần clone vì Report không implement Copy
            chart_modules_content,
            current_route: "dashboard".to_string(),
            current_lang: "vi".to_string(),
            current_time,
            pdf_url,
            additional_context,
        };
        
        // Add sandbox token to additional context - không clone string
        let mut extra_context = context.additional_context.unwrap_or_else(HashMap::new);
        extra_context.insert("sandbox_token".to_string(), serde_json::Value::String(sandboxed_report.sandbox_token));
        context.additional_context = Some(extra_context);
        
        println!("✅ TemplateOrchestrator: Context prepared successfully with sandbox token");
        Ok(context)
    }

    /// Render crypto template with prepared context
    /// 
    /// Core template rendering method using Tera engine with proper error handling
    pub async fn render_template(
        &self,
        tera: &tera::Tera,
        template_path: &str,
        context: TemplateContext
    ) -> Result<String, Box<dyn StdError + Send + Sync>> {
        println!("🎨 TemplateOrchestrator: Rendering template: {}", template_path);
        
        // Clone để đáp ứng yêu cầu 'static của spawn_blocking
        let tera_clone = tera.clone(); // Tera là Arc nên clone nhẹ
        let template_str = template_path.to_string();
        let context_clone = context.clone(); // Cần clone vì spawn_blocking yêu cầu 'static
        
        let render_result = tokio::task::spawn_blocking(move || {
            let mut tera_context = Context::new();
            
            // Insert core template data
            tera_context.insert("report", &context_clone.report);
            tera_context.insert("chart_modules_content", &context_clone.chart_modules_content);
            tera_context.insert("current_route", &context_clone.current_route);
            tera_context.insert("current_lang", &context_clone.current_lang);
            tera_context.insert("current_time", &context_clone.current_time);
            tera_context.insert("pdf_url", &context_clone.pdf_url);
            
            // Add additional context if provided
            if let Some(extra) = context_clone.additional_context {
                for (key, value) in extra {
                    tera_context.insert(&key, &value);
                }
            }
            
            // Template-specific context adjustments
            if template_str.contains("pdf.html") {
                let created_display = (context_clone.report.created_at + chrono::Duration::hours(7))
                    .format("%d-%m-%Y %H:%M").to_string();
                tera_context.insert("created_at_display", &created_display);
            }

            // ONLY use Tera template engine - NO manual HTML
            tera_clone.render(&template_str, &tera_context)
        }).await;
        
        match render_result {
            Ok(Ok(html)) => {
                println!("✅ TemplateOrchestrator: Template rendered successfully");
                Ok(html)
            }
            Ok(Err(e)) => {
                eprintln!("❌ TemplateOrchestrator: Template render error: {:#?}", e);
                let mut src = e.source();
                while let Some(s) = src {
                    eprintln!("❌ Template render error source: {:#?}", s);
                    src = s.source();
                }
                Err(format!("Template render error: {}", e).into())
            }
            Err(e) => {
                eprintln!("❌ TemplateOrchestrator: Task join error: {:#?}", e);
                Err(format!("Task join error: {}", e).into())
            }
        }
    }

    /// Render crypto report view template - High level method
    /// 
    /// Combines context preparation and template rendering for crypto report views
    /// Enhanced to accept pre-loaded chart_modules_content for optimal performance
    /// Now returns compressed HTML data for optimal file size and transfer speed
    pub async fn render_crypto_report_view(
        &self,
        tera: &tera::Tera,
        report: &Report,
        chart_modules_content: Option<String>, // THÊM THAM SỐ NÀY
        additional_context: Option<HashMap<String, serde_json::Value>>
    ) -> Result<Vec<u8>, Box<dyn StdError + Send + Sync>> {
        println!("🚀 TemplateOrchestrator: Rendering crypto report view with compression");
        
        // Step 1: Prepare template context
        let context = self.prepare_crypto_report_context(
            report,
            "view",
            chart_modules_content, // TRUYỀN THAM SỐ
            additional_context
        ).await?;
        
        // Step 2: Render template
        let html = self.render_template(
            tera,
            "crypto/routes/reports/view.html",
            context
        ).await?;
        
        // Step 3: Compress and gather detailed metrics
        println!("🗜️  TemplateOrchestrator: Starting HTML compression with metrics...");
        let original_size = html.len();
        let compressed_data = self.compress_html(&html)?;
        let compressed_size = compressed_data.len();
        let compression_ratio = (1.0 - (compressed_size as f64 / original_size as f64)) * 100.0;
        
        // Log detailed compression information
        println!("📊 TemplateOrchestrator: Compression Metrics:");
        println!("   📏 Original Size: {} bytes ({} KB)", original_size, original_size / 1024);
        println!("   🗜️  Compressed Size: {} bytes ({} KB)", compressed_size, compressed_size / 1024);
        println!("   📈 Compression Ratio: {:.2}%", compression_ratio);
        println!("   💾 Space Saved: {} bytes ({} KB)", original_size - compressed_size, (original_size - compressed_size) / 1024);
        
        println!("✅ TemplateOrchestrator: HTML compression completed successfully");
        
        // Return compressed data instead of original HTML
        Ok(compressed_data)
    }

    /// Render empty template for no reports case
    /// 
    /// Handles the case when no reports are found in database
    pub async fn render_empty_template(
        &self,
        tera: &tera::Tera
    ) -> Result<String, Box<dyn StdError + Send + Sync>> {
        println!("⚠️ TemplateOrchestrator: Rendering empty template");
        
        // Create empty report for template
        let empty_report = Report {
            id: 0,
            html_content: String::new(),
            css_content: Some(String::new()),
            js_content: Some(String::new()),
            html_content_en: Some(String::new()),
            js_content_en: Some(String::new()),
            created_at: chrono::Utc::now(),
        };
        
        // Prepare context
        let context = self.prepare_crypto_report_context(
            &empty_report,
            "empty",
            None, // chart_modules_content
            None  // additional_context
        ).await?;
        
        // Override PDF URL for empty case
        let mut modified_context = context;
        modified_context.pdf_url = "#".to_string();
        
        // Render template
        self.render_template(
            tera,
            "crypto/routes/reports/view.html",
            modified_context
        ).await
    }

    /// Render 404 not found template
    /// 
    /// Handles the case when a specific report ID is not found
    #[allow(dead_code)]
    pub async fn render_not_found_template(
        &self,
        tera: &tera::Tera,
        report_id: i32
    ) -> Result<String, Box<dyn StdError + Send + Sync>> {
        println!("🔍 TemplateOrchestrator: Rendering 404 template for report ID: {}", report_id);
        
        // Create not found report with error messages
        let not_found_report = Report {
            id: report_id,
            html_content: format!(
                "<div class='text-center py-16'><h2 class='text-2xl font-bold text-red-600'>Báo cáo #{} không tồn tại</h2><p class='text-gray-500 mt-4'>Báo cáo này có thể đã bị xóa hoặc không có quyền truy cập.</p><a href='/crypto_reports_list' class='mt-6 inline-block px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700'>Quay lại danh sách báo cáo</a></div>", 
                report_id
            ),
            css_content: Some(String::new()),
            js_content: Some(String::new()),
            html_content_en: Some(format!(
                "<div class='text-center py-16'><h2 class='text-2xl font-bold text-red-600'>Report #{} not found</h2><p class='text-gray-500 mt-4'>This report may have been deleted or you don't have access.</p><a href='/crypto_reports_list' class='mt-6 inline-block px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700'>Back to reports list</a></div>", 
                report_id
            )),
            js_content_en: Some(String::new()),
            created_at: chrono::Utc::now(),
        };
        
        // Prepare context
        let context = self.prepare_crypto_report_context(
            &not_found_report,
            "404",
            None, // chart_modules_content
            None  // additional_context
        ).await?;
        
        // Override PDF URL for 404 case
        let mut modified_context = context;
        modified_context.pdf_url = "#".to_string();
        
        // Render template
        self.render_template(
            tera,
            "crypto/routes/reports/view.html",
            modified_context
        ).await
    }
}
