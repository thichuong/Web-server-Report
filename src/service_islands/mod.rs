//! Service Islands Architecture Registry
//! Central registry for all service islands
//!
//! This module provides the main entry point for the Service Islands Architecture,
//! managing the initialization and health checking of all islands across all layers.

pub mod layer1_infrastructure;
// Layer 2 external services moved to Web-server-Report-websocket service
// pub mod layer2_external_services;
pub mod layer3_communication;
pub mod layer4_observability;
pub mod layer5_business_logic;

use std::sync::Arc;
use tracing::{debug, info, warn};

use layer1_infrastructure::{AppStateIsland, CacheSystemIsland, SharedComponentsIsland};
use layer3_communication::redis_stream_reader::RedisStreamReader;
use layer4_observability::health_system::HealthSystemIsland;
use layer5_business_logic::{crypto_reports::CryptoReportsIsland, dashboard::DashboardIsland};

/// Main Service Islands Registry
///
/// This struct holds references to all service islands and provides
/// unified initialization and health checking capabilities.
pub struct ServiceIslands {
    // Layer 1: Infrastructure Islands
    pub app_state: Arc<AppStateIsland>,
    // Legacy AppState for backward compatibility (lazily initialized)
    legacy_app_state: std::sync::OnceLock<Arc<layer1_infrastructure::AppState>>,
    pub shared_components: Arc<SharedComponentsIsland>,
    pub cache_system: Arc<CacheSystemIsland>,

    // Layer 3: Communication Islands
    pub redis_stream_reader: Arc<RedisStreamReader>,

    // Layer 4: Observability Islands
    pub health_system: Arc<HealthSystemIsland>,

    // Layer 5: Business Logic Islands
    pub dashboard: Arc<DashboardIsland>,
    pub crypto_reports: Arc<CryptoReportsIsland>,
}

impl ServiceIslands {
    /// Initialize all Service Islands
    ///
    /// This method initializes all service islands in the proper dependency order.
    /// Layer 1 (Infrastructure) first, then Layer 2 (External Services), then Layer 4 (Observability), then Layer 3 (Communication), then Layer 5 (Business Logic).
    ///
    /// # Errors
    ///
    /// Returns error if any island initialization fails (database, cache, templates, etc.)
    pub fn initialize() -> Result<Self, anyhow::Error> {
        info!("🏝️ Initializing Service Islands Architecture...");

        // Initialize Layer 1: Infrastructure (foundation layer)
        info!("🏗️ Initializing Layer 1: Infrastructure Islands...");
        // Layer 1 is now synchronous
        let app_state = Arc::new(AppStateIsland::new()?);
        let shared_components = Arc::new(SharedComponentsIsland::new()?);
        let cache_system = Arc::new(CacheSystemIsland::new()?);

        // Initialize Layer 3: Communication (Redis Stream Reader for data from websocket service)
        info!("📡 Initializing Layer 3: Communication Islands (Redis Stream Reader)...");
        let redis_stream_reader = Arc::new(RedisStreamReader::new(Arc::clone(&cache_system)));
        info!("✅ Redis Stream Reader initialized!");

        // Initialize Layer 4: Observability
        info!("🔍 Initializing Layer 4: Observability Islands...");
        let health_system = Arc::new(HealthSystemIsland::new()?);

        // Initialize Layer 5: Business Logic
        // Note: Layer 5 now reads from cache/streams instead of calling external APIs directly
        info!("📊 Initializing Layer 5: Business Logic Islands...");
        let dashboard = Arc::new(DashboardIsland::new()?);
        let crypto_reports = Arc::new(CryptoReportsIsland::new()?);

        info!("✅ Layer 1 Infrastructure Islands initialized!");
        info!("✅ Layer 3 Communication Islands initialized!");
        info!("✅ Layer 4 Observability Islands initialized!");
        info!("✅ Layer 5 Business Logic Islands initialized!");
        info!("✅ Service Islands Architecture initialized (Main Service)!");
        info!("📡 Note: External APIs and WebSocket are handled by separate websocket service");

        info!("📊 Architecture Status:");
        info!("  🏝️ Total Islands: 6/6 islands (Main Service)");
        info!("  🏗️ Layer 1 - Infrastructure: 3/3 islands");
        info!("  📡 Layer 3 - Communication: 1/1 islands (Redis Stream Reader)");
        info!("  🔍 Layer 4 - Observability: 1/1 islands");
        info!("  📊 Layer 5 - Business Logic: 2/2 islands");

        Ok(Self {
            app_state,
            legacy_app_state: std::sync::OnceLock::new(),
            shared_components,
            cache_system,
            redis_stream_reader,
            health_system,
            dashboard,
            crypto_reports,
        })
    }

    // Note: initialize_unified_streaming and initialize_stream_integration methods
    // have been removed as WebSocket functionality is now in a separate service

    /// Perform health check on all Service Islands
    ///
    /// Returns true if all islands are healthy, false otherwise.
    pub async fn health_check(&self) -> bool {
        debug!("🔍 Performing Service Islands health check (Main Service)...");

        let shared_components_healthy = self.shared_components.health_check();
        let app_state_healthy = self.app_state.health_check();
        let cache_system_healthy = self.cache_system.health_check();
        let redis_stream_reader_healthy = self
            .redis_stream_reader
            .health_check()
            .await
            .unwrap_or(false);
        let health_system_healthy = self.health_system.health_check();
        let dashboard_healthy = self.dashboard.health_check();
        let crypto_reports_healthy = self.crypto_reports.health_check();

        let all_healthy = shared_components_healthy
            && app_state_healthy
            && cache_system_healthy
            && redis_stream_reader_healthy
            && health_system_healthy
            && dashboard_healthy
            && crypto_reports_healthy;

        if all_healthy {
            info!("✅ All Service Islands are healthy!");
        } else {
            warn!("❌ Some Service Islands are unhealthy!");
            warn!(
                "   Shared Components Island: {}",
                if shared_components_healthy {
                    "✅"
                } else {
                    "❌"
                }
            );
            warn!(
                "   App State Island: {}",
                if app_state_healthy { "✅" } else { "❌" }
            );
            warn!(
                "   Cache System Island: {}",
                if cache_system_healthy { "✅" } else { "❌" }
            );
            warn!(
                "   Redis Stream Reader: {}",
                if redis_stream_reader_healthy {
                    "✅"
                } else {
                    "❌"
                }
            );
            warn!(
                "   Health System Island: {}",
                if health_system_healthy { "✅" } else { "❌" }
            );
            warn!(
                "   Dashboard Island: {}",
                if dashboard_healthy { "✅" } else { "❌" }
            );
            warn!(
                "   Crypto Reports Island: {}",
                if crypto_reports_healthy { "✅" } else { "❌" }
            );
        }

        all_healthy
    }

    /// Get pre-loaded chart modules content for optimal performance
    /// Direct access to Layer 1 `SharedComponentsIsland`
    pub fn get_chart_modules_content(&self) -> Arc<String> {
        Arc::clone(&self.shared_components.chart_modules_content)
    }

    /// Get legacy `AppState` for backward compatibility
    ///
    /// This method creates a legacy `AppState` instance with cache system integration
    /// for components that haven't been fully migrated to Service Islands.
    /// Get legacy `AppState` for backward compatibility
    ///
    /// This method creates a legacy `AppState` instance with cache system integration
    /// for components that haven't been fully migrated to Service Islands.
    pub fn get_legacy_app_state(&self) -> Arc<layer1_infrastructure::AppState> {
        Arc::clone(self.legacy_app_state.get_or_init(|| {
            // ✅ Cache re-enabled after spawn_blocking memory leak fix
            Arc::new(
                self.app_state
                    .create_legacy_app_state(Some(Arc::clone(&self.cache_system))),
            )
        }))
    }

    /// Graceful shutdown of all Service Islands
    ///
    /// ✅ PRODUCTION-READY: Properly closes all resources in reverse dependency order
    /// Ensures database connections and Redis connections are cleanly closed.
    ///
    /// # Errors
    ///
    /// Returns error if resource cleanup fails (database close, cache cleanup, etc.)
    pub async fn shutdown(&self) -> Result<(), anyhow::Error> {
        info!("🛑 Initiating graceful shutdown of Service Islands...");

        // Shutdown in reverse dependency order (Layer 5 → Layer 4 → Layer 3 → Layer 1)

        // Layer 5: Business Logic Islands (no resources to cleanup)
        info!("📊 Layer 5: Business Logic Islands - no cleanup needed");

        // Layer 4: Observability Islands (no resources to cleanup)
        info!("🔍 Layer 4: Observability Islands - no cleanup needed");

        // Layer 3: Communication Islands
        info!("📡 Layer 3: Cleaning up Communication Islands...");
        if let Err(e) = self.redis_stream_reader.cleanup() {
            warn!("   ⚠️  Redis Stream Reader cleanup error: {}", e);
        }

        // Layer 1: Infrastructure Islands
        info!("🏗️  Layer 1: Cleaning up Infrastructure Islands...");

        // Close database connections
        info!("   🗄️  Closing database connection pool...");
        let legacy_state = self.get_legacy_app_state();
        legacy_state.db.close().await;
        info!("   ✅ Database connections closed");

        // Cache system cleanup (Redis connections handled by multi-tier-cache library)
        info!("   💾 Cache system cleanup - Redis connections handled by library");

        info!("✅ Service Islands shutdown complete");
        Ok(())
    }

    // Note: active_connections() method removed as WebSocket tracking is in separate service
}
