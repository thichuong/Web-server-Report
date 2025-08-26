//! Application State - Enhanced with Service Islands Integration
//! 
//! Provides AppState with full Service Islands Architecture integration
//! including Redis Streams as primary storage and multi-tier caching.

use std::sync::{Arc, atomic::{AtomicU64, AtomicI32}};
use tera::Tera;
use sqlx::PgPool;
use crate::service_islands::layer1_infrastructure::cache_system_island::CacheSystemIsland;

/// Enhanced AppState with Service Islands Architecture Integration
/// 
/// Provides full access to Redis Streams as primary storage and
/// multi-tier caching system for optimal performance.
pub struct AppState {
    // Database connection pool (backup storage)
    pub db: PgPool,
    // Cache System Island with Redis Streams (primary storage)
    pub cache_system: Option<Arc<CacheSystemIsland>>,
    // Minimal fields to prevent compilation errors
    pub request_counter: AtomicU64,
    pub cached_latest_id: AtomicI32,
    // Tera template engine for crypto dashboard
    pub tera: Tera,
}

#[allow(dead_code)]
pub struct CacheStats {
    pub entries: u64,
    pub l1_entry_count: u64,
    pub l1_hit_count: u64,
    pub l1_miss_count: u64,
    pub l1_hit_rate: f64,
}

#[allow(dead_code)]
pub struct CacheHealth {
    pub l2_healthy: bool,
    pub overall_healthy: bool,
}

impl AppState {
    /// Create a new AppState instance with Service Islands Integration
    pub async fn new() -> Result<Self, anyhow::Error> {
        // Initialize database connection
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://localhost/crypto_reports".to_string());
        let db = PgPool::connect(&database_url).await?;
        
        // Initialize Cache System Island with Redis Streams
        let cache_system = match CacheSystemIsland::new().await {
            Ok(cache) => {
                println!("✅ Cache System Island with Redis Streams initialized successfully");
                Some(Arc::new(cache))
            },
            Err(e) => {
                println!("⚠️ Cache System Island initialization failed: {}", e);
                println!("   Continuing with minimal compatibility mode...");
                None
            }
        };
        
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
            cache_system,
            request_counter: AtomicU64::new(0),
            cached_latest_id: AtomicI32::new(0),
            tera,
        })
    }
    
    /// Get cache system for Layer 3 → Layer 1 communication
    pub fn get_cache_system(&self) -> Option<Arc<CacheSystemIsland>> {
        self.cache_system.clone()
    }
}
