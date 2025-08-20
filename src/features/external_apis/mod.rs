// src/features/external_apis/mod.rs - External APIs Feature
//
// Handles all external service integrations with rate limiting,
// circuit breakers, and intelligent caching

pub mod api_client;
pub mod market_data_provider;
pub mod rate_limiter;
pub mod models;

pub use api_client::ApiClient;
pub use market_data_provider::MarketDataProvider;
pub use rate_limiter::RateLimiter;
pub use models::*;

use crate::features::Feature;
use crate::features::shared_components::state::FeatureContext;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

/// External APIs Feature - Layer 2 (External Services)
pub struct ExternalApisFeature {
    market_data_provider: Option<Arc<MarketDataProvider>>,
}

impl ExternalApisFeature {
    pub fn new() -> Self {
        Self {
            market_data_provider: None,
        }
    }

    pub fn market_data_provider(&self) -> Option<&Arc<MarketDataProvider>> {
        self.market_data_provider.as_ref()
    }
}

#[async_trait]
impl Feature for ExternalApisFeature {
    fn name(&self) -> &'static str {
        "external_apis"
    }

    fn version(&self) -> &'static str {
        "1.0.0"
    }

    fn dependencies(&self) -> Vec<&'static str> {
        vec!["shared_components", "cache_system"] // Layer 2 depends on Layer 1
    }

    async fn initialize(&mut self, context: &HashMap<String, String>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let taapi_secret = context
            .get("taapi_secret")
            .ok_or("taapi_secret not provided in context")?;

        let cache_manager = context
            .get("cache_manager")
            .and_then(|_| {
                // Cache manager would be passed through feature context
                // For now, we'll handle this in the integration layer
                None
            });

        let market_data_provider = Arc::new(
            MarketDataProvider::new(taapi_secret.clone(), cache_manager).await?
        );

        self.market_data_provider = Some(market_data_provider);
        println!("✅ External APIs Feature initialized");
        
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(_provider) = self.market_data_provider.take() {
            println!("🔽 External APIs Feature shutting down");
        }
        Ok(())
    }

    fn is_healthy(&self) -> bool {
        self.market_data_provider.is_some()
    }
}
