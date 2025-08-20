//! Application State - Minimal Compatibility Layer
//! 
//! This provides a minimal AppState implementation to maintain compatibility
//! with existing handlers during the transition to Service Islands Architecture.

use std::sync::atomic::{AtomicU64, AtomicI32};
use tera::Tera;
use sqlx::PgPool;

/// Minimal AppState for compatibility with existing handlers
/// This is a temporary bridge while we transition to full Service Islands Architecture
pub struct AppState {
    // Database connection pool
    pub db: PgPool,
    // Minimal fields to prevent compilation errors
    pub request_counter: AtomicU64,
    pub cached_latest_id: AtomicI32,
    // Tera template engine for crypto dashboard
    pub tera: Tera,
}

// Placeholder structs for components that handlers expect
pub struct ReportCache;
pub struct CacheManager; 
pub struct Metrics;
pub struct DbConnection;

impl ReportCache {
    pub async fn stats(&self) -> CacheStats {
        CacheStats { 
            entries: 0,
            l1_entry_count: 0,
            l1_hit_count: 0,
            l1_miss_count: 0,
            l1_hit_rate: 0.0,
        }
    }
    
    pub fn hit_rate(&self) -> f64 {
        0.0
    }
}

impl CacheManager {
    pub async fn stats(&self) -> CacheStats {
        CacheStats { 
            entries: 0,
            l1_entry_count: 0,
            l1_hit_count: 0,
            l1_miss_count: 0,
            l1_hit_rate: 0.0,
        }
    }
    
    pub async fn health_check(&self) -> CacheHealth {
        CacheHealth {
            l2_healthy: true,
            overall_healthy: true,
        }
    }
}

impl Metrics {
    pub fn record_request(&self, _response_time: u64) {
        // Placeholder
    }
    
    pub fn avg_response_time(&self) -> f64 {
        0.0
    }
}

pub struct CacheStats {
    pub entries: u64,
    pub l1_entry_count: u64,
    pub l1_hit_count: u64,
    pub l1_miss_count: u64,
    pub l1_hit_rate: f64,
}

pub struct CacheHealth {
    pub l2_healthy: bool,
    pub overall_healthy: bool,
}

impl AppState {
    /// Create a new AppState instance
    pub async fn new() -> Result<Self, anyhow::Error> {
        // Initialize database connection
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://localhost/crypto_reports".to_string());
        let db = PgPool::connect(&database_url).await?;
        
        // Initialize Tera template engine following archive pattern
        let mut tera = match Tera::new("dashboards/**/*.html") {
            Ok(t) => t,
            Err(e) => {
                println!("Warning: Template parsing error: {}", e);
                Tera::default()
            }
        };
        
        // Register crypto dashboard templates with logical names used across codebase
        if let Err(e) = tera.add_template_file("dashboards/crypto_dashboard/routes/reports/view.html", Some("crypto/routes/reports/view.html")) {
            println!("Warning: Failed to load crypto reports view template: {}", e);
        }
        if let Err(e) = tera.add_template_file("dashboards/crypto_dashboard/routes/reports/pdf.html", Some("crypto/routes/reports/pdf.html")) {
            println!("Warning: Failed to load crypto reports pdf template: {}", e);
        }
        if let Err(e) = tera.add_template_file("dashboards/crypto_dashboard/routes/reports/list.html", Some("crypto/routes/reports/list.html")) {
            println!("Warning: Failed to load crypto reports list template: {}", e);
        }
        
        // Register shared components for backward compatibility  
        if let Err(e) = tera.add_template_file("shared_components/theme_toggle.html", Some("crypto/components/theme_toggle.html")) {
            println!("Warning: Failed to load crypto theme toggle template: {}", e);
        }
        if let Err(e) = tera.add_template_file("shared_components/language_toggle.html", Some("crypto/components/language_toggle.html")) {
            println!("Warning: Failed to load crypto language toggle template: {}", e);
        }
        
        tera.autoescape_on(vec![]); // Disable auto-escaping for safe content

        Ok(Self {
            db,
            request_counter: AtomicU64::new(0),
            cached_latest_id: AtomicI32::new(0),
            tera,
        })
    }
    
    /// Prime cache - placeholder method
    pub async fn prime_cache(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Placeholder implementation
        Ok(())
    }
    
    // Add placeholder fields that health checker expects
    pub fn report_cache(&self) -> ReportCache {
        ReportCache
    }
    
    pub fn cache_manager(&self) -> CacheManager {
        CacheManager
    }
    
    pub fn metrics(&self) -> Metrics {
        Metrics
    }
    
    pub fn db(&self) -> DbConnection {
        DbConnection
    }
}
