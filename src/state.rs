use sqlx::PgPool;
use std::sync::{Arc, atomic::AtomicUsize};
use tokio::sync::RwLock;
use dashmap::DashMap;
use tera::Tera;
use crate::models::Report;
use crate::data_service::DataService;
use crate::performance::{MultiLevelCache, PerformanceMetrics};
use std::time::Duration;

pub struct AppState {
    pub db: PgPool,
    // Optimized multi-level cache
    pub report_cache: MultiLevelCache<i32, Report>,
    // Thread-safe cache cho chart modules
    pub chart_modules_cache: RwLock<Option<String>>,
    // DashMap thay thế RwLock<HashMap> để tránh lock contention
    pub cached_reports: DashMap<i32, Report>,
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
}

impl AppState {
    pub async fn new(
        db: PgPool,
        taapi_secret: String,
        redis_url: &str,
    ) -> Result<Self, anyhow::Error> {
        // Initialize data service
        let data_service = DataService::new(taapi_secret);
        
        // Initialize WebSocket service
        let websocket_service = Arc::new(
            crate::websocket_service::WebSocketService::new(redis_url, data_service)?
        );
        
        // Start background data updates
        websocket_service.start_background_updates().await;

        // Initialize Tera template engine with new architecture
        let mut tera = Tera::default();
        
        // Load shared components (global)
        tera.add_template_file("shared_components/theme_toggle.html", Some("shared/components/theme_toggle.html"))
            .expect("Failed to load shared theme_toggle.html");
        tera.add_template_file("shared_components/language_toggle.html", Some("shared/components/language_toggle.html"))
            .expect("Failed to load shared language_toggle.html");
        
        // Load route-specific templates for crypto_dashboard
        tera.add_template_file("dashboards/crypto_dashboard/routes/reports/view.html", Some("crypto/routes/reports/view.html"))
            .expect("Failed to load crypto reports view template");
        tera.add_template_file("dashboards/crypto_dashboard/routes/reports/pdf.html", Some("crypto/routes/reports/pdf.html"))
            .expect("Failed to load crypto reports pdf template");
        tera.add_template_file("dashboards/crypto_dashboard/routes/reports/list.html", Some("crypto/routes/reports/list.html"))
            .expect("Failed to load crypto reports list template");
        
        // Load legacy templates for backwards compatibility (keeping for fallback)
        // Add legacy components as well for backwards compatibility
        tera.add_template_file("shared_components/theme_toggle.html", Some("crypto/components/theme_toggle.html"))
            .expect("Failed to load legacy crypto theme_toggle.html");
        tera.add_template_file("shared_components/language_toggle.html", Some("crypto/components/language_toggle.html"))
            .expect("Failed to load legacy crypto language_toggle.html");
        
        tera.autoescape_on(vec![]); // Disable auto-escaping for safe content

        Ok(Self {
            db,
            report_cache: MultiLevelCache::new(1000, Duration::from_secs(3600)), // 1000 reports, 1 hour TTL
            chart_modules_cache: RwLock::new(None),
            cached_reports: DashMap::new(), // Thread-safe HashMap
            cached_latest_id: AtomicUsize::new(0), // Atomic counter
            tera,
            websocket_service,
            metrics: Arc::new(PerformanceMetrics::default()),
            request_counter: AtomicUsize::new(0),
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
            // Insert vào DashMap (thread-safe)
            self.cached_reports.insert(report.id, report.clone());
            // Cập nhật latest id với atomic
            self.cached_latest_id.store(report.id as usize, std::sync::atomic::Ordering::Relaxed);
        }
    }
}
