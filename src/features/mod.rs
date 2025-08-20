// src/features/mod.rs - Feature Registry & Integration Layer
//
// This module serves as the central registry for all Service Island features
// and provides the integration layer that connects features together while
// maintaining their independence.

pub mod crypto_reports;
pub mod dashboard;  
pub mod health_system;
pub mod cache_system;
pub mod websocket_service;
pub mod external_apis;
pub mod shared_components;

// Re-export feature APIs for easy access
pub use crypto_reports::CryptoReportsFeature;
pub use dashboard::DashboardFeature;
pub use health_system::HealthSystem;
pub use cache_system::CacheSystem;
pub use websocket_service::WebSocketService;
pub use external_apis::ExternalApis;
pub use shared_components::SharedComponents;

/// Feature integration traits for dependency injection
pub mod integration {
    use std::sync::Arc;
    use super::*;
    
    /// Feature service registry for dependency injection
    pub struct FeatureRegistry {
        pub cache_system: Arc<CacheSystem>,
        pub shared_components: Arc<SharedComponents>,
        pub external_apis: Arc<ExternalApis>,
        pub websocket_service: Arc<WebSocketService>,
        pub health_system: Arc<HealthSystem>,
        pub dashboard: Arc<DashboardFeature>,
        pub crypto_reports: Arc<CryptoReportsFeature>,
    }
    
    impl FeatureRegistry {
        /// Initialize all features in dependency order
        pub async fn new(
            database_url: &str,
            redis_url: &str, 
            taapi_secret: String
        ) -> Result<Self, anyhow::Error> {
            // Layer 1: Infrastructure (no dependencies)
            let shared_components = Arc::new(SharedComponents::new());
            let cache_system = Arc::new(CacheSystem::new(redis_url).await?);
            
            // Layer 2: External Services (depend on infrastructure)
            let external_apis = Arc::new(ExternalApis::new(taapi_secret));
            
            // Layer 3: Communication Services
            let websocket_service = Arc::new(WebSocketService::new());
            
            // Layer 4: Observability (monitors all other layers)
            let health_system = Arc::new(HealthSystem::new());
            
            // Layer 5: Business Features (TODO: Extract from monolithic handlers)
            let dashboard = Arc::new(DashboardFeature::new(
                external_apis.clone(),
                cache_system.clone(),
                websocket_service.clone()
            ).await?);
            
            let crypto_reports = Arc::new(CryptoReportsFeature::new(
                database_url,
                cache_system.clone(),
                shared_components.clone()
            ).await?);
            
            Ok(Self {
                cache_system,
                shared_components,
                external_apis,
                websocket_service,
                health_system,
                dashboard,
                crypto_reports,
            })
        }
        
        /// Get route definitions from all features
        pub fn collect_routes(&self) -> axum::Router<std::sync::Arc<crate::state::AppState>> {
            use axum::Router;
            
            Router::new()
                // Health system routes (Layer 4 - Observability)
                .merge(HealthSystem::routes())
                // TODO: Add business layer routes when extracted
                // .merge(self.dashboard.routes())
                // .merge(self.crypto_reports.routes())
                // WebSocket routes (Layer 3)
                // .merge(self.websocket_service.routes())
                // Shared static routes (Layer 1)
                // .merge(self.shared_components.static_routes())
        }
    }
}

/// Feature trait that all service islands must implement
pub trait Feature {
    type Config;
    type Error;
    
    /// Initialize the feature with its configuration
    async fn new(config: Self::Config) -> Result<Self, Self::Error> 
    where 
        Self: Sized;
    
    /// Get the feature name for logging and identification
    fn name(&self) -> &'static str;
    
    /// Health check for the feature
    async fn health_check(&self) -> FeatureHealthStatus;
}

/// Health status for individual features
#[derive(Debug, Clone)]
pub struct FeatureHealthStatus {
    pub feature_name: String,
    pub healthy: bool,
    pub message: String,
    pub details: serde_json::Value,
}

impl FeatureHealthStatus {
    pub fn healthy(feature_name: &str) -> Self {
        Self {
            feature_name: feature_name.to_string(),
            healthy: true,
            message: "Feature operating normally".to_string(),
            details: serde_json::json!({}),
        }
    }
    
    pub fn unhealthy(feature_name: &str, message: &str) -> Self {
        Self {
            feature_name: feature_name.to_string(),
            healthy: false,
            message: message.to_string(),
            details: serde_json::json!({}),
        }
    }
    
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = details;
        self
    }
}
