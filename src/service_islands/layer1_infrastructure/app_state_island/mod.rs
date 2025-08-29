//! App State Island - Layer 1 Infrastructure
//! 
//! Centralized application state management with Service Islands Architecture integration.
//! Provides database connections, template engine (Tera), and core application counters.

use std::sync::{Arc, atomic::{AtomicU64, AtomicI32}};
use tera::Tera;
use sqlx::PgPool;
use anyhow::Result;

/// App State Island - Centralized Application State
/// 
/// This island manages core application state including:
/// - Database connection pool (PostgreSQL)
/// - Template engine (Tera) for rendering HTML templates
/// - Request counters and application metrics
/// - Integration points for Cache System Island
pub struct AppStateIsland {
    /// Database connection pool (backup storage)
    pub db: PgPool,
    /// Tera template engine for crypto dashboard and shared components
    pub tera: Tera,
    /// Request counter for application metrics
    pub request_counter: AtomicU64,
    /// Cached latest ID for optimization
    pub cached_latest_id: AtomicI32,
}

/// Legacy AppState struct for backward compatibility
/// 
/// This struct maintains compatibility with existing code while
/// providing integration with Service Islands Architecture.
pub struct AppState {
    /// Database connection pool (backup storage)  
    pub db: PgPool,
    /// Cache System Island reference (injected during ServiceIslands initialization)
    pub cache_system: Option<Arc<crate::service_islands::layer1_infrastructure::CacheSystemIsland>>,
    /// Request counter for application metrics
    pub request_counter: AtomicU64,
    /// Cached latest ID for optimization
    pub cached_latest_id: AtomicI32,
    /// Tera template engine for crypto dashboard
    pub tera: Tera,
}

impl AppStateIsland {
    /// Initialize the App State Island
    pub async fn new() -> Result<Self> {
        println!("🏗️ Initializing App State Island...");
        
        // Initialize database connection
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://localhost/crypto_reports".to_string());
        let db = PgPool::connect(&database_url).await?;
        
        // Initialize Tera template engine
        let tera = Self::initialize_template_engine().await?;
        
        println!("✅ App State Island initialized successfully");
        
        Ok(Self {
            db,
            tera,
            request_counter: AtomicU64::new(0),
            cached_latest_id: AtomicI32::new(0),
        })
    }
    
    /// Initialize Tera template engine with all required templates
    async fn initialize_template_engine() -> Result<Tera> {
        println!("📝 Initializing Tera template engine...");
        
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
        
        // Register market indicators component for homepage
        if let Err(e) = tera.add_template_file("shared_components/market-indicators/market-indicators.html", Some("shared/components/market-indicators.html")) {
            println!("Warning: Failed to load market indicators component: {}", e);
        }
        
        // Register home.html template 
        if let Err(e) = tera.add_template_file("dashboards/home.html", Some("home.html")) {
            println!("Warning: Failed to load homepage template: {}", e);
        }
        
        tera.autoescape_on(vec![]); // Disable auto-escaping for safe content
        
        println!("✅ Tera template engine initialized with all templates");
        Ok(tera)
    }
    
    /// Health check for the App State Island
    pub async fn health_check(&self) -> bool {
        // Check database connection
        let db_healthy = match sqlx::query("SELECT 1").fetch_one(&self.db).await {
            Ok(_) => true,
            Err(e) => {
                println!("  ⚠️ Database health check failed: {}", e);
                false
            }
        };
        
        // Check template engine
        let tera_healthy = !self.tera.get_template_names().collect::<Vec<_>>().is_empty();
        
        if db_healthy && tera_healthy {
            println!("  ✅ App State Island health check passed");
            true
        } else {
            println!("  ⚠️ App State Island health check failed - DB: {}, Templates: {}", db_healthy, tera_healthy);
            false
        }
    }
    
    /// Create legacy AppState for backward compatibility
    /// 
    /// This method creates an AppState instance that can be injected with
    /// a CacheSystemIsland reference during ServiceIslands initialization.
    pub fn create_legacy_app_state(&self, cache_system: Option<Arc<crate::service_islands::layer1_infrastructure::CacheSystemIsland>>) -> AppState {
        AppState {
            db: self.db.clone(),
            cache_system,
            request_counter: AtomicU64::new(self.request_counter.load(std::sync::atomic::Ordering::Relaxed)),
            cached_latest_id: AtomicI32::new(self.cached_latest_id.load(std::sync::atomic::Ordering::Relaxed)),
            tera: self.tera.clone(),
        }
    }
    
}

impl AppState {
    /// Create a new AppState instance with Service Islands Integration
    /// 
    /// **DEPRECATED**: Use `AppStateIsland::new()` and `ServiceIslands::initialize()` instead.
    /// This method is kept for backward compatibility only.
    #[allow(dead_code)]
    pub async fn new() -> Result<Self, anyhow::Error> {
        println!("⚠️ Using deprecated AppState::new() - consider migrating to ServiceIslands architecture");
        
        // Initialize database connection
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://localhost/crypto_reports".to_string());
        let db = PgPool::connect(&database_url).await?;
        
        // Initialize Cache System Island with Redis Streams
        let cache_system = match crate::service_islands::layer1_infrastructure::CacheSystemIsland::new().await {
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
        
        // Initialize Tera template engine using the same logic as AppStateIsland
        let tera = AppStateIsland::initialize_template_engine().await?;
        
        Ok(Self {
            db,
            cache_system,
            request_counter: AtomicU64::new(0),
            cached_latest_id: AtomicI32::new(0),
            tera,
        })
    }
}
