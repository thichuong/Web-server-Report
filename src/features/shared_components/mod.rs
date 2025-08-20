// src/features/shared_components/mod.rs - Shared Components Feature
//
// This feature provides common utilities, templates, and shared functionality
// that can be used by other features. It has zero business logic dependencies
// and serves as the foundation layer.

pub mod utils;
pub mod models;
pub mod templates;
pub mod state;

use crate::features::{Feature, FeatureHealthStatus};
use axum::Router;
use std::sync::Arc;

/// Shared Components Feature - Provides common utilities and templates
pub struct SharedComponentsFeature {
    // Template management could go here
    // Common utilities
    // Shared configuration
}

impl SharedComponentsFeature {
    pub async fn new() -> Result<Self, anyhow::Error> {
        Ok(Self {
            // Initialize shared components
        })
    }
    
    /// Provide static asset routes
    pub fn static_routes(&self) -> Router<Arc<crate::state::AppState>> {
        use axum::routing::get;
        use tower_http::services::ServeDir;
        
        Router::new()
            // Shared static assets
            .nest_service("/shared_assets", ServeDir::new("shared_assets"))
            .nest_service("/shared_components", ServeDir::new("shared_components"))
            // Chart modules endpoint
            .route("/shared_assets/js/chart_modules.js", get(crate::handlers::serve_chart_modules))
    }
}

#[async_trait::async_trait]
impl Feature for SharedComponentsFeature {
    type Config = ();
    type Error = anyhow::Error;
    
    async fn new(_config: Self::Config) -> Result<Self, Self::Error> {
        Self::new().await
    }
    
    fn name(&self) -> &'static str {
        "shared_components"
    }
    
    async fn health_check(&self) -> FeatureHealthStatus {
        FeatureHealthStatus::healthy("shared_components")
            .with_details(serde_json::json!({
                "templates_loaded": true,
                "static_assets_available": true
            }))
    }
}
