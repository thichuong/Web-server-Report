//! Service Islands Architecture Registry
//! Central registry for all service islands
//! 
//! This module provides the main entry point for the Service Islands Architecture,
//! managing the initialization and health checking of all islands across all layers.

pub mod layer1_infrastructure;
pub mod layer2_external_services;
pub mod layer3_communication;
pub mod layer4_observability;
pub mod layer5_business_logic;

use std::sync::Arc;

use layer1_infrastructure::{
    AppStateIsland,
    SharedComponentsIsland,
    CacheSystemIsland,
};
use layer2_external_services::{
    external_apis_island::ExternalApisIsland,
};
use layer3_communication::{
    websocket_service::WebSocketServiceIsland,
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
    
    // Layer 2: External Services Islands
    pub external_apis: Arc<ExternalApisIsland>,
    
    // Layer 3: Communication Islands
    pub websocket_service: Arc<WebSocketServiceIsland>,
    
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
        
        // Initialize Layer 2: External Services (depends on Layer 1 - Cache System)
        println!("🌐 Initializing Layer 2: External Services Islands with Cache...");
        let taapi_secret = std::env::var("TAAPI_SECRET").unwrap_or_else(|_| "default_secret".to_string());
        let cmc_api_key = std::env::var("CMC_API_KEY").ok(); // Optional CoinMarketCap API key
        let finnhub_api_key = std::env::var("FINNHUB_API_KEY").ok(); // Optional Finnhub API key
        
        if cmc_api_key.is_some() {
            println!("🔑 CoinMarketCap API key found - enabling fallback support");
        } else {
            println!("⚠️ No CoinMarketCap API key - using CoinGecko only");
        }
        
        if finnhub_api_key.is_some() {
            println!("📈 Finnhub API key found - enabling US stock indices");
        } else {
            println!("⚠️ No Finnhub API key - US stock indices will be unavailable");
        }
        
        let external_apis = Arc::new(ExternalApisIsland::with_cache_and_all_keys(
            taapi_secret,
            cmc_api_key,
            finnhub_api_key,
            Some(cache_system.clone())
        ).await?);
        
        // Initialize Layer 4: Observability
        println!("🔍 Initializing Layer 4: Observability Islands...");
        let health_system = Arc::new(HealthSystemIsland::new().await?);
        
        // Initialize Layer 3: Communication (depends on Layer 2 + Cache Optimization)
        println!("📡 Initializing Layer 3: Communication Islands with Layer 2 dependencies and Cache Optimization...");
        let websocket_service = Arc::new(
            WebSocketServiceIsland::with_external_apis_and_cache(
                external_apis.clone(),
                cache_system.clone()
            ).await?
        );
        
        // Initialize Layer 5: Business Logic (depends on Layer 3 ONLY)
        println!("📊 Initializing Layer 5: Business Logic Islands...");
        let dashboard = Arc::new(DashboardIsland::with_dependencies(websocket_service.clone()).await?);
        // ✅ STRICT: Layer 5 only depends on Layer 3 (no direct Layer 2 access)
        let crypto_reports = Arc::new(CryptoReportsIsland::with_dependencies(websocket_service.clone()).await?);
        
        println!("✅ Layer 1 Infrastructure Islands initialized!");
        println!("✅ Layer 2 External Services Islands initialized with Cache!");
        println!("✅ Layer 4 Observability Islands initialized!");
        println!("✅ Layer 3 Communication Islands initialized!");
        println!("✅ Layer 5 Business Logic Islands initialized!");
        println!("✅ Service Islands Architecture initialized with API caching!");
        
        println!("📊 Architecture Status:");
        println!("  🏝️ Total Islands: 8/8 (100% complete)");
        println!("  🏗️ Layer 1 - Infrastructure: 3/3 islands");
        println!("  🌐 Layer 2 - External Services: 1/1 islands");
        println!("  📡 Layer 3 - Communication: 1/2 islands");
        println!("  🔍 Layer 4 - Observability: 1/1 islands");
        println!("  � Layer 5 - Business Logic: 2/2 islands");
        
        Ok(Self {
            app_state,
            legacy_app_state: std::sync::OnceLock::new(),
            shared_components,
            cache_system,
            external_apis,
            websocket_service,
            health_system,
            dashboard,
            crypto_reports,
        })
    }
    
    /// Initialize unified streaming with Layer 5 access
    /// 
    /// This ensures WebSocket streaming uses the same Layer 5 → Layer 3 → Layer 2 flow
    /// as HTTP API and WebSocket initial messages for data consistency.
    pub async fn initialize_unified_streaming(&self) -> Result<(), anyhow::Error> {
        println!("🔄 Initializing unified streaming with Layer 5 access...");
        
        // Configure WebSocket streaming to use ServiceIslands for unified data access
        self.websocket_service.start_streaming_with_service_islands(
            Arc::new(ServiceIslands {
                app_state: self.app_state.clone(),
                legacy_app_state: std::sync::OnceLock::new(),
                shared_components: self.shared_components.clone(),
                cache_system: self.cache_system.clone(),
                external_apis: self.external_apis.clone(),
                websocket_service: self.websocket_service.clone(),
                health_system: self.health_system.clone(),
                dashboard: self.dashboard.clone(),
                crypto_reports: self.crypto_reports.clone(),
            })
        ).await?;
        
        println!("✅ Unified streaming initialized - all messages now use same Layer 2 access!");
        Ok(())
    }

    /// Initialize Redis Streams integration for real-time updates
    /// 
    /// Phase 3: Connects Redis Streams to WebSocket broadcasting for sub-millisecond real-time updates
    pub async fn initialize_stream_integration(&self) -> Result<(), anyhow::Error> {
        println!("🚀 Phase 3: Initializing Redis Streams → WebSocket integration...");
        
        // Connect Redis Streams consumer to WebSocket broadcasting
        if let Err(e) = self.websocket_service.start_stream_consumer(self.cache_system.clone()).await {
            println!("⚠️ Redis Streams consumer initialization failed: {}", e);
            return Err(anyhow::anyhow!("Stream integration failed: {}", e));
        }
        
        println!("✅ Phase 3: Redis Streams → WebSocket integration active!");
        println!("📡 Real-time updates: Stream → WebSocket broadcast (<1ms latency)");
        Ok(())
    }
    
    /// Perform health check on all Service Islands
    /// 
    /// Returns true if all islands are healthy, false otherwise.
    pub async fn health_check(&self) -> bool {
        println!("🔍 Performing Service Islands health check...");
        
        let shared_components_healthy = self.shared_components.health_check().await;
        let app_state_healthy = self.app_state.health_check().await;
        let cache_system_healthy = self.cache_system.health_check().await;
        let external_apis_healthy = self.external_apis.health_check().await.unwrap_or(false);
        let websocket_service_healthy = self.websocket_service.health_check().await.is_ok();
        let health_system_healthy = self.health_system.health_check().await;
        let dashboard_healthy = self.dashboard.health_check().await;
        let crypto_reports_healthy = self.crypto_reports.health_check().await;
        
        let all_healthy = shared_components_healthy && app_state_healthy && cache_system_healthy && external_apis_healthy && websocket_service_healthy && health_system_healthy && dashboard_healthy && crypto_reports_healthy;
        
        if all_healthy {
            println!("✅ All Service Islands are healthy!");
        } else {
            println!("❌ Some Service Islands are unhealthy!");
            println!("   Shared Components Island: {}", if shared_components_healthy { "✅" } else { "❌" });
            println!("   App State Island: {}", if app_state_healthy { "✅" } else { "❌" });
            println!("   Cache System Island: {}", if cache_system_healthy { "✅" } else { "❌" });
            println!("   External APIs Island: {}", if external_apis_healthy { "✅" } else { "❌" });
            println!("   WebSocket Service Island: {}", if websocket_service_healthy { "✅" } else { "❌" });
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
}
