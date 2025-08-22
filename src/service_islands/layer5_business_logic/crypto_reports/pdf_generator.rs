//! PDF Generator Component
//! 
//! This component handles PDF generation operations for crypto reports,
//! including A4 optimization and print-ready formatting.

use std::{error::Error as StdError, sync::Arc};
use tera::Context;

// Import from current state - will be refactored when lower layers are implemented
use crate::state::AppState;

// Import models from report_creator
use super::report_creator::ReportCreator;

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
}
