//! Model Registry Component
//!
//! Manages all data model definitions and provides model utilities.
//! Central registry for data structures used across the application.

use anyhow::Result;
use serde_json;
use std::collections::HashMap;
use tracing::{debug, error};

/// Model Registry manages all data model definitions
pub struct ModelRegistry {
    /// Model definitions registry
    models: HashMap<String, serde_json::Value>,
}

impl ModelRegistry {
    /// Initialize the Model Registry
    ///
    /// # Errors
    ///
    /// Returns error if model initialization fails
    pub fn new() -> Result<Self> {
        debug!("📊 Initializing Model Registry...");

        let models = Self::initialize_models()?;

        debug!("  📊 Registered {} data models", models.len());

        Ok(Self { models })
    }

    /// Initialize all model definitions
    fn initialize_models() -> Result<HashMap<String, serde_json::Value>> {
        let mut models = HashMap::new();

        // Register core models
        models.insert("Report".to_string(), serde_json::json!({
            "type": "object",
            "properties": {
                "id": {"type": "integer"},
                "title": {"type": "string", "maxLength": 255},
                "content": {"type": "string"},
                "created_at": {"type": "string", "format": "date-time"},
                "updated_at": {"type": "string", "format": "date-time"},
                "report_type": {"type": "string", "enum": ["crypto", "stock", "market_analysis"]},
                "status": {"type": "string", "enum": ["draft", "published", "archived"]},
                "metadata": {"type": "object"}
            },
            "required": ["id", "title", "content", "report_type", "status"]
        }));

        Ok(models)
    }

    /// Health check for model registry
    pub fn health_check(&self) -> bool {
        // Verify that core models are available
        let core_models = vec!["Report"];

        for model_name in core_models {
            if !self.models.contains_key(model_name) {
                error!("❌ Core model '{}' is missing", model_name);
                return false;
            }
        }

        debug!("✅ Model Registry health check passed");
        true
    }
}
