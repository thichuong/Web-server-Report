//! Shared Components Island
//! 
//! This island provides foundational components that are shared across all other service islands:
//! - Template management with Tera engine
//! - Common data model definitions
//! - Utility functions and helpers
//! - Configuration management

use std::sync::Arc;
use anyhow::Result;

pub mod template_registry;
pub mod model_registry;
pub mod utility_functions;

use template_registry::TemplateRegistry;
use model_registry::ModelRegistry;
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
}


impl SharedComponentsIsland {
    /// Initialize the Shared Components Island
    /// 
    /// This method sets up all the core shared components including:
    /// - Template engine initialization
    /// - Model registry setup
    /// - Utility functions preparation
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
        
        println!("🧩 Shared Components Island initialization complete!");
        
        Ok(Self {
            template_registry,
            model_registry,
            utility_functions,
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
        
        let all_healthy = template_healthy && model_healthy && utility_healthy;
        
        if all_healthy {
            println!("✅ Shared Components Island is healthy!");
        } else {
            println!("❌ Shared Components Island health issues detected:");
            if !template_healthy { println!("  ❌ Template Registry unhealthy"); }
            if !model_healthy { println!("  ❌ Model Registry unhealthy"); }
            if !utility_healthy { println!("  ❌ Utility Functions unhealthy"); }
        }
        
        all_healthy
    }
}
