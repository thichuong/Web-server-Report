use anyhow::Result;
use sqlx::PgPool;
use std::sync::{
    Arc,
    atomic::{AtomicI32, AtomicU64},
};
use tera::Tera;
use tracing::{debug, info, warn};

// Import cache system from library
use multi_tier_cache::{
    CacheManager, CacheSystemBuilder, backends::moka_cache::MokaCacheConfig,
    backends::redis_cache::RedisCache,
};
use std::time::Duration;

/// Core Application State
///
/// Holds application resources:
/// - Database pool
/// - Tera templates
/// - Multi-tier Cache Manager
/// - Application counters
pub struct AppState {
    pub db: PgPool,
    pub tera: Arc<Tera>,
    pub cache_manager: Arc<CacheManager>,
    pub request_counter: AtomicU64,
    pub cached_latest_id: AtomicI32,
    pub crypto_handlers: crate::services::crypto_reports::handlers::CryptoHandlers,
    pub dashboard_handlers: crate::services::dashboard::DashboardHandlers,
}

impl AppState {
    /// Initialize the application state
    ///
    /// # Errors
    /// Returns an error if database connection or cache system initialization fails.
    pub async fn new() -> Result<Self> {
        info!("🏗️ Initializing Application State...");

        // 1. Initialize DB
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://localhost/crypto_reports".to_string());
        let db = PgPool::connect(&database_url).await?;

        // 2. Initialize Templates
        let tera = Arc::new(Self::initialize_template_engine());

        // 3. Initialize Cache System
        let moka_config = MokaCacheConfig {
            max_capacity: 1000,
            time_to_live: Duration::from_mins(30), // 30 mins
            time_to_idle: Duration::from_mins(2),  // 2 mins
        };

        let redis_url =
            std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
        let redis_backend = Arc::new(
            RedisCache::with_url(&redis_url)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to initialize Redis cache backend: {e}"))?,
        );

        let cache_system = CacheSystemBuilder::new()
            .with_moka_config(moka_config)
            .with_l2(redis_backend)
            .build()
            .await?;
        let cache_manager: Arc<CacheManager> = cache_system.cache_manager.clone();

        info!("✅ Application State initialized successfully");

        Ok(Self {
            db,
            tera,
            cache_manager: cache_manager.clone(),
            request_counter: AtomicU64::new(0),
            cached_latest_id: AtomicI32::new(0),
            crypto_handlers: crate::services::crypto_reports::handlers::CryptoHandlers::new(),
            dashboard_handlers: crate::services::dashboard::DashboardHandlers::new(),
        })
    }

    /// Health check
    pub async fn health_check(&self) -> bool {
        self.cache_manager.get("_health_check").await.is_ok()
    }

    fn initialize_template_engine() -> Tera {
        debug!("📝 Initializing Tera template engine...");

        let mut tera = match Tera::new("frontend/**/*.html") {
            Ok(t) => t,
            Err(e) => {
                warn!("Template parsing error: {}", e);
                Tera::default()
            }
        };

        // Register templates
        let templates = vec![
            (
                "frontend/pages/report_view/view.html",
                "crypto/routes/reports/view.html",
            ),
            (
                "frontend/pages/reports_list/list.html",
                "crypto/routes/reports/list.html",
            ),
            (
                "frontend/pages/market_analytics/market_analytics.html",
                "crypto/routes/analytics/market_analytics.html",
            ),
            (
                "frontend/shared/components/theme_toggle.html",
                "crypto/components/theme_toggle.html",
            ),
            (
                "frontend/shared/components/language_toggle.html",
                "crypto/components/language_toggle.html",
            ),
            (
                "frontend/pages/home/components/market-indicators/market-indicators.html",
                "shared/components/market-indicators.html",
            ),
            ("frontend/pages/home/home.html", "home.html"),
        ];

        for (path, name) in templates {
            if let Err(e) = tera.add_template_file(path, Some(name)) {
                warn!("Failed to load template {path}: {e}");
            }
        }

        tera.autoescape_on(vec![]);
        info!("✅ Tera template engine initialized");
        tera
    }
}
