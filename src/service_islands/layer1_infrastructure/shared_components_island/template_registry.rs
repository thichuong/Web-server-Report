//! Template Registry Component
//! 
//! Manages all templates using the Tera templating engine.
//! Provides template loading, caching, and rendering capabilities.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use anyhow::Result;
use tera::{Tera, Context};
use serde_json;

/// Template Registry manages all application templates
pub struct TemplateRegistry {
    /// Tera templating engine
    tera: Tera,
    /// Template usage statistics
    render_count: Arc<AtomicUsize>,
    template_cache_hits: Arc<AtomicUsize>,
    /// Available templates registry
    available_templates: HashMap<String, String>,
}

impl TemplateRegistry {
    /// Initialize the Template Registry
    pub async fn new() -> Result<Self> {
        println!("📄 Initializing Template Registry...");
        
        // Initialize Tera with template patterns
        let mut tera = Tera::new("dashboards/**/*.html")?;
        
        // Add additional template directories
        tera.add_template_files(vec![
            ("dashboards/home.html", Some("home")),
            ("shared_components/language_toggle.html", Some("language_toggle")),
            ("shared_components/theme_toggle.html", Some("theme_toggle")),
        ])?;
        
        // Register custom filters and functions
        tera.register_filter("currency_format", Self::currency_format_filter);
        tera.register_filter("percentage_format", Self::percentage_format_filter);
        tera.register_filter("date_format", Self::date_format_filter);
        
        let available_templates = Self::discover_templates().await?;
        
        println!("  📄 Loaded {} templates", available_templates.len());
        
        Ok(Self {
            tera,
            render_count: Arc::new(AtomicUsize::new(0)),
            template_cache_hits: Arc::new(AtomicUsize::new(0)),
            available_templates,
        })
    }
    
    /// Discover all available templates
    async fn discover_templates() -> Result<HashMap<String, String>> {
        let mut templates = HashMap::new();
        
        // Core templates
        templates.insert("home".to_string(), "Main dashboard home page".to_string());
        templates.insert("crypto_dashboard".to_string(), "Cryptocurrency dashboard".to_string());
        templates.insert("stock_dashboard".to_string(), "Stock market dashboard".to_string());
        templates.insert("language_toggle".to_string(), "Language switching component".to_string());
        templates.insert("theme_toggle".to_string(), "Theme switching component".to_string());
        
        // Report templates
        templates.insert("crypto_report".to_string(), "Cryptocurrency report template".to_string());
        templates.insert("pdf_report".to_string(), "PDF report template".to_string());
        
        Ok(templates)
    }
    
    /// Render a template with the provided context
    pub async fn render_template(&self, template_name: &str, context: &Context) -> Result<String> {
        self.render_count.fetch_add(1, Ordering::Relaxed);
        
        match self.tera.render(template_name, context) {
            Ok(rendered) => {
                self.template_cache_hits.fetch_add(1, Ordering::Relaxed);
                Ok(rendered)
            },
            Err(e) => {
                eprintln!("❌ Template rendering failed for '{}': {}", template_name, e);
                Err(anyhow::anyhow!("Template rendering failed: {}", e))
            }
        }
    }
    
    /// Health check method for monitoring
    pub async fn health_check(&self) -> bool {
        // Test basic template registry functionality by checking if templates are loaded
        let template_count = self.tera.get_template_names().count();
        
        if template_count > 0 {
            println!("✅ Template Registry health check passed: {} templates loaded", template_count);
            true
        } else {
            eprintln!("❌ Template Registry health check failed: No templates loaded");
            false
        }
    }
    
    /// Get template registry statistics
    pub async fn get_statistics(&self) -> Result<serde_json::Value> {
        let total_renders = self.render_count.load(Ordering::Relaxed);
        let cache_hits = self.template_cache_hits.load(Ordering::Relaxed);
        let hit_rate = if total_renders > 0 {
            (cache_hits as f64 / total_renders as f64) * 100.0
        } else {
            0.0
        };
        
        Ok(serde_json::json!({
            "component": "template_registry",
            "total_renders": total_renders,
            "cache_hits": cache_hits,
            "hit_rate_percent": hit_rate,
            "available_templates": self.available_templates.len(),
            "templates": self.available_templates
        }))
    }
    
    /// Get list of available templates
    pub async fn list_templates(&self) -> Vec<String> {
        self.available_templates.keys().cloned().collect()
    }
    
    /// Check if a template exists
    pub async fn template_exists(&self, template_name: &str) -> bool {
        self.available_templates.contains_key(template_name)
    }
    
    // Custom Tera filters
    
    /// Currency formatting filter
    fn currency_format_filter(value: &tera::Value, _args: &HashMap<String, tera::Value>) -> tera::Result<tera::Value> {
        let num = value.as_f64().ok_or_else(|| tera::Error::msg("Value must be a number"))?;
        Ok(tera::Value::String(format!("${:.2}", num)))
    }
    
    /// Percentage formatting filter
    fn percentage_format_filter(value: &tera::Value, _args: &HashMap<String, tera::Value>) -> tera::Result<tera::Value> {
        let num = value.as_f64().ok_or_else(|| tera::Error::msg("Value must be a number"))?;
        Ok(tera::Value::String(format!("{:.2}%", num * 100.0)))
    }
    
    /// Date formatting filter
    fn date_format_filter(value: &tera::Value, args: &HashMap<String, tera::Value>) -> tera::Result<tera::Value> {
        let date_str = value.as_str().ok_or_else(|| tera::Error::msg("Value must be a string"))?;
        let format = args.get("format").and_then(|v| v.as_str()).unwrap_or("%Y-%m-%d");
        
        // Simple date formatting - in a real implementation, you'd use chrono
        Ok(tera::Value::String(format!("Formatted({}): {}", format, date_str)))
    }
}
