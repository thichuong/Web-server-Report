//! Shared Components Island
//! 
//! This island provides foundational components that are shared across all other service islands:
//! - Template management with Tera engine
//! - Common data model definitions
//! - Utility functions and helpers
//! - Configuration management
//! - Chart modules pre-loading and caching

use std::sync::Arc;
use anyhow::Result;

pub mod template_registry;
pub mod model_registry;
pub mod utility_functions;
pub mod chart_modules_service;

use template_registry::TemplateRegistry;
use model_registry::ModelRegistry;
use utility_functions::UtilityFunctions;
use chart_modules_service::ChartModulesService;

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
    pub async fn new() -> Result<Self> {
        println!("🧩 Initializing Shared Components Island...");
        
        // Initialize template registry
        let template_registry = Arc::new(TemplateRegistry::new()?);
        println!("  ✅ Template Registry initialized");
        
        // Initialize model registry
        let model_registry = Arc::new(ModelRegistry::new().await?);
        println!("  ✅ Model Registry initialized");
        
        // Initialize utility functions
        let utility_functions = Arc::new(UtilityFunctions::new().await?);
        println!("  ✅ Utility Functions initialized");
        
        // Initialize chart modules service with pre-loading
        println!("📦 Layer 1: Đang chuẩn bị và cache lại chart_modules.js...");
        let chart_modules_service = Arc::new(ChartModulesService::new().await?);
        let chart_modules_content = chart_modules_service.get_content();
        println!("  ✅ Chart Modules Service initialized");
        
        println!("🧩 Shared Components Island initialization complete!");
        
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
    pub async fn health_check(&self) -> bool {
        println!("🔍 Checking Shared Components Island health...");
        
        let template_healthy = self.template_registry.health_check().await;
        let model_healthy = self.model_registry.health_check().await;
        let utility_healthy = self.utility_functions.health_check().await;
        let chart_modules_healthy = self.chart_modules_service.health_check().await;
        
        let all_healthy = template_healthy && model_healthy && utility_healthy && chart_modules_healthy;
        
        if all_healthy {
            println!("✅ Shared Components Island is healthy!");
        } else {
            println!("❌ Shared Components Island health issues detected:");
            if !template_healthy { println!("  ❌ Template Registry unhealthy"); }
            if !model_healthy { println!("  ❌ Model Registry unhealthy"); }
            if !utility_healthy { println!("  ❌ Utility Functions unhealthy"); }
            if !chart_modules_healthy { println!("  ❌ Chart Modules Service unhealthy"); }
        }
        
        all_healthy
    }
}
