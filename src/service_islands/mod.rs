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

use layer1_infrastructure::{
    AppStateIsland,
    SharedComponentsIsland,
    CacheSystemIsland,
};
use layer3_communication::{
    redis_stream_reader::RedisStreamReader,
};
use layer4_observability::{
    health_system::HealthSystemIsland,
};
use layer5_business_logic::{
    dashboard::DashboardIsland,
    crypto_reports::CryptoReportsIsland,
};

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
    pub async fn initialize() -> Result<Self, anyhow::Error> {
        println!("🏝️ Initializing Service Islands Architecture...");
        
        // Initialize Layer 1: Infrastructure (foundation layer)
        println!("🏗️ Initializing Layer 1: Infrastructure Islands...");
        let app_state = Arc::new(AppStateIsland::new().await?);
        let shared_components = Arc::new(SharedComponentsIsland::new().await?);
        let cache_system = Arc::new(CacheSystemIsland::new().await?);

        // Initialize Layer 3: Communication (Redis Stream Reader for data from websocket service)
        println!("📡 Initializing Layer 3: Communication Islands (Redis Stream Reader)...");
        let redis_stream_reader = Arc::new(RedisStreamReader::new(cache_system.clone()));
        println!("✅ Redis Stream Reader initialized!");

        // Initialize Layer 4: Observability
        println!("🔍 Initializing Layer 4: Observability Islands...");
        let health_system = Arc::new(HealthSystemIsland::new().await?);

        // Initialize Layer 5: Business Logic
        // Note: Layer 5 now reads from cache/streams instead of calling external APIs directly
        println!("📊 Initializing Layer 5: Business Logic Islands...");
        let dashboard = Arc::new(DashboardIsland::new().await?);
        let crypto_reports = Arc::new(CryptoReportsIsland::new().await?);
        
        println!("✅ Layer 1 Infrastructure Islands initialized!");
        println!("✅ Layer 3 Communication Islands initialized!");
        println!("✅ Layer 4 Observability Islands initialized!");
        println!("✅ Layer 5 Business Logic Islands initialized!");
        println!("✅ Service Islands Architecture initialized (Main Service)!");
        println!("📡 Note: External APIs and WebSocket are handled by separate websocket service");

        println!("📊 Architecture Status:");
        println!("  🏝️ Total Islands: 6/6 islands (Main Service)");
        println!("  🏗️ Layer 1 - Infrastructure: 3/3 islands");
        println!("  📡 Layer 3 - Communication: 1/1 islands (Redis Stream Reader)");
        println!("  🔍 Layer 4 - Observability: 1/1 islands");
        println!("  📊 Layer 5 - Business Logic: 2/2 islands");

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
        println!("🔍 Performing Service Islands health check (Main Service)...");

        let shared_components_healthy = self.shared_components.health_check().await;
        let app_state_healthy = self.app_state.health_check().await;
        let cache_system_healthy = self.cache_system.health_check().await;
        let redis_stream_reader_healthy = self.redis_stream_reader.health_check().await.unwrap_or(false);
        let health_system_healthy = self.health_system.health_check().await;
        let dashboard_healthy = self.dashboard.health_check().await;
        let crypto_reports_healthy = self.crypto_reports.health_check().await;

        let all_healthy = shared_components_healthy && app_state_healthy && cache_system_healthy && redis_stream_reader_healthy && health_system_healthy && dashboard_healthy && crypto_reports_healthy;

        if all_healthy {
            println!("✅ All Service Islands are healthy!");
        } else {
            println!("❌ Some Service Islands are unhealthy!");
            println!("   Shared Components Island: {}", if shared_components_healthy { "✅" } else { "❌" });
            println!("   App State Island: {}", if app_state_healthy { "✅" } else { "❌" });
            println!("   Cache System Island: {}", if cache_system_healthy { "✅" } else { "❌" });
            println!("   Redis Stream Reader: {}", if redis_stream_reader_healthy { "✅" } else { "❌" });
            println!("   Health System Island: {}", if health_system_healthy { "✅" } else { "❌" });
            println!("   Dashboard Island: {}", if dashboard_healthy { "✅" } else { "❌" });
            println!("   Crypto Reports Island: {}", if crypto_reports_healthy { "✅" } else { "❌" });
        }

        all_healthy
    }
    
    /// Get pre-loaded chart modules content for optimal performance
    /// Direct access to Layer 1 SharedComponentsIsland
    pub fn get_chart_modules_content(&self) -> Arc<String> {
        self.shared_components.chart_modules_content.clone()
    }
    
    /// Get legacy AppState for backward compatibility
    ///
    /// This method creates a legacy AppState instance with cache system integration
    /// for components that haven't been fully migrated to Service Islands.
    pub fn get_legacy_app_state(&self) -> Arc<layer1_infrastructure::AppState> {
        self.legacy_app_state.get_or_init(|| {
            Arc::new(self.app_state.create_legacy_app_state(Some(self.cache_system.clone())))
        }).clone()
    }

    /// Graceful shutdown of all Service Islands
    ///
    /// ✅ PRODUCTION-READY: Properly closes all resources in reverse dependency order
    /// Ensures database connections and Redis connections are cleanly closed.
    pub async fn shutdown(&self) -> Result<(), anyhow::Error> {
        println!("🛑 Initiating graceful shutdown of Service Islands...");

        // Shutdown in reverse dependency order (Layer 5 → Layer 4 → Layer 3 → Layer 1)

        // Layer 5: Business Logic Islands (no resources to cleanup)
        println!("📊 Layer 5: Business Logic Islands - no cleanup needed");

        // Layer 4: Observability Islands (no resources to cleanup)
        println!("🔍 Layer 4: Observability Islands - no cleanup needed");

        // Layer 3: Communication Islands
        println!("📡 Layer 3: Cleaning up Communication Islands...");
        if let Err(e) = self.redis_stream_reader.cleanup().await {
            eprintln!("   ⚠️  Redis Stream Reader cleanup error: {}", e);
        }

        // Layer 1: Infrastructure Islands
        println!("🏗️  Layer 1: Cleaning up Infrastructure Islands...");

        // Close database connections
        println!("   🗄️  Closing database connection pool...");
        let legacy_state = self.get_legacy_app_state();
        legacy_state.db.close().await;
        println!("   ✅ Database connections closed");

        // Cache system cleanup (Redis connections handled by multi-tier-cache library)
        println!("   💾 Cache system cleanup - Redis connections handled by library");

        println!("✅ Service Islands shutdown complete");
        Ok(())
    }

    // Note: active_connections() method removed as WebSocket tracking is in separate service
}
