use sqlx::PgPool;
use std::sync::{Arc, atomic::AtomicUsize};
use tokio::sync::RwLock;
// dashmap removed; MultiLevelCache is used for reports instead
use tera::Tera;
use crate::models::Report;
use crate::data_service::DataService;
use crate::performance::{MultiLevelCache, PerformanceMetrics};
use crate::cache::CacheManager;
use std::time::{Duration, Instant};

pub struct AppState {
    pub db: PgPool,
    // Unified cache manager for all caching operations
    pub cache_manager: Arc<CacheManager>,
    // Data service with cache integration
    pub data_service: DataService,
    // Optimized multi-level cache for reports
    pub report_cache: MultiLevelCache<i32, Report>,
    // Thread-safe cache cho chart modules
    pub chart_modules_cache: RwLock<Option<String>>,
    // Atomic cho latest report ID
    pub cached_latest_id: AtomicUsize, // Sử dụng AtomicUsize thay vì RwLock
    // Tera template engine - thread-safe
    pub tera: Tera,
    // WebSocket service for real-time updates
    pub websocket_service: Arc<crate::websocket_service::WebSocketService>,
    // Performance metrics
    pub metrics: Arc<PerformanceMetrics>,
    // Request counter cho monitoring
    pub request_counter: AtomicUsize,
    // Server start time for uptime monitoring
    pub start_time: Instant,
}

impl AppState {
    pub async fn new(database_url: &str, redis_url: &str, taapi_secret: String) -> Result<Self, anyhow::Error> {
        // Initialize database connection
        let db = PgPool::connect(database_url).await?;
        
        // Initialize unified cache manager
        let cache_manager = Arc::new(CacheManager::new(redis_url).await?);
        
        // Initialize data service with cache manager integration
        let data_service = DataService::with_cache_manager(taapi_secret, cache_manager.clone());
        
        // Initialize WebSocket service with CacheManager
        let websocket_service = Arc::new(crate::websocket_service::WebSocketService::new(cache_manager.clone(), data_service.clone())?);

        // Initialize Tera template engine
        let mut tera = match Tera::new("dashboards/**/*.html") {
            Ok(t) => t,
            Err(e) => {
                println!("Warning: Template parsing error: {}", e);
                Tera::default()
            }
        };
        
        // Register shared components for backward compatibility
        tera.add_template_file("shared_components/theme_toggle.html", Some("crypto/components/theme_toggle.html"))
            .expect("Failed to load legacy crypto theme_toggle.html");
        tera.add_template_file("shared_components/language_toggle.html", Some("crypto/components/language_toggle.html"))
            .expect("Failed to load legacy crypto language_toggle.html");
        
        // Explicitly register crypto dashboard templates with logical names used across the codebase
        tera.add_template_file("dashboards/crypto_dashboard/routes/reports/view.html", Some("crypto/routes/reports/view.html"))
            .expect("Failed to load crypto reports view template");
        tera.add_template_file("dashboards/crypto_dashboard/routes/reports/pdf.html", Some("crypto/routes/reports/pdf.html"))
            .expect("Failed to load crypto reports pdf template");
        tera.add_template_file("dashboards/crypto_dashboard/routes/reports/list.html", Some("crypto/routes/reports/list.html"))
            .expect("Failed to load crypto reports list template");
        
        tera.autoescape_on(vec![]); // Disable auto-escaping for safe content

        Ok(Self {
            db,
            cache_manager,
            data_service,
            report_cache: MultiLevelCache::new(1000, Duration::from_secs(3600)), // 1000 reports, 1 hour TTL
            chart_modules_cache: RwLock::new(None),
            // cached_reports removed: replaced by `report_cache` (MultiLevelCache)
            cached_latest_id: AtomicUsize::new(0), // Atomic counter
            tera,
            websocket_service,
            metrics: Arc::new(PerformanceMetrics::default()),
            request_counter: AtomicUsize::new(0),
            start_time: Instant::now(),
        })
    }

    pub async fn prime_cache(&self) {
        // Prime the latest-report cache once at startup to reduce first-request latency
        // (best-effort; failure won't stop the server)
        if let Ok(Some(report)) = sqlx::query_as::<_, Report>(
            "SELECT id, html_content, css_content, js_content, html_content_en, js_content_en, created_at FROM report ORDER BY created_at DESC LIMIT 1",
        )
        .fetch_optional(&self.db)
        .await
        {
            // Insert into L1 cache (report_cache)
            self.report_cache.insert(report.id, report.clone()).await;
            // Update latest id with atomic
            self.cached_latest_id.store(report.id as usize, std::sync::atomic::Ordering::Relaxed);
        }
    }
}
