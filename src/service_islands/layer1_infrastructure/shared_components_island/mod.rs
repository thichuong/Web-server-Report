//! Shared Components Island
//! 
//! This island provides foundational components that are shared across all other service islands:
//! - Template management with Tera engine
//! - Common data model definitions
//! - Utility functions and helpers
//! - Configuration management

use std::sync::Arc;
use anyhow::Result;
use serde_json;

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
        let template_registry = Arc::new(TemplateRegistry::new().await?);
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
    
    /// Get comprehensive statistics about the shared components
    pub async fn get_statistics(&self) -> Result<serde_json::Value> {
        let template_stats = self.template_registry.get_statistics().await?;
        let model_stats = self.model_registry.get_statistics().await?;
        let utility_stats = self.utility_functions.get_statistics().await?;
        
        Ok(serde_json::json!({
            "island": "shared_components",
            "status": "operational",
            "components": {
                "template_registry": template_stats,
                "model_registry": model_stats,
                "utility_functions": utility_stats
            },
            "uptime_info": {
                "initialization_time": "startup",
                "components_count": 3,
                "health_status": "green"
            }
        }))
    }
    
    /// Render a template with provided context
    pub async fn render_template(&self, template_name: &str, context: &tera::Context) -> Result<String> {
        self.template_registry.render_template(template_name, context).await
    }
    
    /// Get a model definition by name
    pub async fn get_model_definition(&self, model_name: &str) -> Result<serde_json::Value> {
        self.model_registry.get_model_definition(model_name).await
    }
    
    /// Execute a utility function
    pub async fn execute_utility(&self, function_name: &str, args: serde_json::Value) -> Result<serde_json::Value> {
        self.utility_functions.execute_function(function_name, args).await
    }
}
