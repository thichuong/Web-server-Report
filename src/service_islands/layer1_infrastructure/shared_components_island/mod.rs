//! Shared Components Island
//!
//! This island provides foundational components that are shared across all other service islands:
//! - Template management with Tera engine
//! - Common data model definitions
//! - Utility functions and helpers
//! - Configuration management
//! - Chart modules pre-loading and caching

use anyhow::Result;
use std::sync::Arc;
use tracing::{debug, info, warn};

pub mod chart_modules_service;
pub mod model_registry;
pub mod template_registry;
pub mod utility_functions;

use chart_modules_service::ChartModulesService;
use model_registry::ModelRegistry;
use template_registry::TemplateRegistry;
use utility_functions::UtilityFunctions;

/// Shared Components Island
///
/// This service island manages all shared components and utilities that are used
/// across the entire application. It serves as the foundation layer for all other islands.
pub struct SharedComponentsIsland {
    /// Template management system
    pub template_registry: Arc<TemplateRegistry>,
    /// Data model definitions and utilities
    pub model_registry: Arc<ModelRegistry>,
    /// Common utility functions
    pub utility_functions: Arc<UtilityFunctions>,
    /// Chart modules service with pre-loaded content
    pub chart_modules_service: Arc<ChartModulesService>,
    /// Pre-loaded chart modules content for direct access
    pub chart_modules_content: Arc<String>,
}

impl SharedComponentsIsland {
    /// Initialize the Shared Components Island
    ///
    /// This method sets up all the core shared components including:
    /// - Template engine initialization
    /// - Model registry setup
    /// - Utility functions preparation
    /// - Chart modules pre-loading and caching
    ///
    /// # Errors
    ///
    /// Returns error if any component initialization fails (templates, models, utilities, or chart modules)
    pub async fn new() -> Result<Self> {
        info!("🧩 Initializing Shared Components Island...");

        // Initialize template registry
        let template_registry = Arc::new(TemplateRegistry::new()?);
        debug!("  ✅ Template Registry initialized");

        // Initialize model registry
        let model_registry = Arc::new(ModelRegistry::new()?);
        debug!("  ✅ Model Registry initialized");

        // Initialize utility functions
        let utility_functions = Arc::new(UtilityFunctions::new()?);
        debug!("  ✅ Utility Functions initialized");

        // Initialize chart modules service with pre-loading
        info!("📦 Layer 1: Đang chuẩn bị và cache lại chart_modules.js...");
        let chart_modules_service = Arc::new(ChartModulesService::new().await?);
        let chart_modules_content = chart_modules_service.get_content();
        info!("  ✅ Chart Modules Service initialized");

        info!("🧩 Shared Components Island initialization complete!");

        Ok(Self {
            template_registry,
            model_registry,
            utility_functions,
            chart_modules_service,
            chart_modules_content,
        })
    }

    /// Perform health check on the Shared Components Island
    ///
    /// Returns true if all components are healthy and operational
    pub fn health_check(&self) -> bool {
        debug!("🔍 Checking Shared Components Island health...");

        let template_healthy = self.template_registry.health_check();
        let model_healthy = self.model_registry.health_check();
        let utility_healthy = self.utility_functions.health_check();
        let chart_modules_healthy = self.chart_modules_service.health_check();

        let all_healthy =
            template_healthy && model_healthy && utility_healthy && chart_modules_healthy;

        if all_healthy {
            debug!("✅ Shared Components Island is healthy!");
        } else {
            warn!("❌ Shared Components Island health issues detected:");
            if !template_healthy {
                warn!("  ❌ Template Registry unhealthy");
            }
            if !model_healthy {
                warn!("  ❌ Model Registry unhealthy");
            }
            if !utility_healthy {
                warn!("  ❌ Utility Functions unhealthy");
            }
            if !chart_modules_healthy {
                warn!("  ❌ Chart Modules Service unhealthy");
            }
        }

        all_healthy
    }
}
