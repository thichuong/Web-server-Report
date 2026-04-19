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
    CacheManager, CacheSystemBuilder, RedisStreams, backends::moka_cache::MokaCacheConfig,
    backends::redis_cache::RedisCache,
};
use std::time::Duration;

use crate::assets::load_chart_modules;
/// Core Application State
///
/// Replaces the complex `ServiceIslands` architecture with a standard Axum state that holds:
/// - Database pool
/// - Tera templates
/// - Multi-tier Cache Manager
/// - Shared static components (Chart modules)
/// - Application counters
pub struct AppState {
    pub db: PgPool,
    pub tera: Arc<Tera>,
    pub cache_manager: Arc<CacheManager>,
    pub chart_modules_content: Arc<String>,
    pub request_counter: AtomicU64,
    pub cached_latest_id: AtomicI32,
    pub crypto_handlers: crate::services::crypto_reports::handlers::CryptoHandlers,
    pub dashboard_handlers: crate::services::dashboard::DashboardHandlers,
    pub redis_stream_reader: crate::stream::RedisStreamReader,
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
            time_to_live: Duration::from_secs(30 * 60), // 30 mins
            time_to_idle: Duration::from_secs(2 * 60),  // 2 mins
        };

        let redis_url =
            std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
        let redis_backend = Arc::new(
            RedisCache::with_url(&redis_url)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to initialize Redis cache backend: {}", e))?,
        );

        let redis_streams = Arc::new(
            RedisStreams::new(&redis_url)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to initialize Redis streams backend: {e}"))?,
        );

        let cache_system = CacheSystemBuilder::new()
            .with_moka_config(moka_config)
            .with_l2(redis_backend)
            .with_streams(redis_streams)
            .build()
            .await?;
        let cache_manager: Arc<CacheManager> = cache_system.cache_manager.clone();

        // 4. Initialize Chart Modules
        let chart_modules_content = Arc::new(load_chart_modules()?);

        info!("✅ Application State initialized successfully");

        Ok(Self {
            db,
            tera,
            cache_manager: cache_manager.clone(),
            chart_modules_content,
            request_counter: AtomicU64::new(0),
            cached_latest_id: AtomicI32::new(0),
            crypto_handlers: crate::services::crypto_reports::handlers::CryptoHandlers::new(),
            dashboard_handlers: crate::services::dashboard::DashboardHandlers::new(),
            redis_stream_reader: crate::stream::RedisStreamReader::new(Arc::clone(&cache_manager)),
        })
    }

    /// Health check
    pub async fn health_check(&self) -> bool {
        // Just return true or add more checks
        self.redis_stream_reader
            .health_check()
            .await
            .unwrap_or(false)
    }

    fn initialize_template_engine() -> Tera {
        debug!("📝 Initializing Tera template engine...");

        let mut tera = match Tera::new("dashboards/**/*.html") {
            Ok(t) => t,
            Err(e) => {
                warn!("Template parsing error: {}", e);
                Tera::default()
            }
        };

        // Register templates
        let templates = vec![
            (
                "dashboards/crypto_dashboard/routes/reports/view.html",
                "crypto/routes/reports/view.html",
            ),
            (
                "dashboards/crypto_dashboard/routes/reports/view_dsd.html",
                "crypto/routes/reports/view_dsd.html",
            ),
            (
                "dashboards/crypto_dashboard/routes/reports/list.html",
                "crypto/routes/reports/list.html",
            ),
            (
                "shared_components/theme_toggle.html",
                "crypto/components/theme_toggle.html",
            ),
            (
                "shared_components/language_toggle.html",
                "crypto/components/language_toggle.html",
            ),
            (
                "shared_components/market-indicators/market-indicators.html",
                "shared/components/market-indicators.html",
            ),
            ("dashboards/home.html", "home.html"),
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
