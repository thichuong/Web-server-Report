//! Model Registry Component
//! 
//! Manages all data model definitions and provides model utilities.
//! Central registry for data structures used across the application.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json;

/// Core Report data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub id: i32,
    pub title: String,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
    pub report_type: String,
    pub status: String,
    pub metadata: serde_json::Value,
}

/// Report Summary structure for list views
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSummary {
    pub id: i32,
    pub title: String,
    pub report_type: String,
    pub status: String,
    pub created_at: String,
    pub summary: String,
}

/// Report List Item for efficient rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportListItem {
    pub id: i32,
    pub title: String,
    pub report_type: String,
    pub created_at: String,
    pub status: String,
}

/// Market Data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketData {
    pub symbol: String,
    pub price: f64,
    pub change_24h: f64,
    pub change_percent_24h: f64,
    pub volume_24h: f64,
    pub market_cap: f64,
    pub last_updated: String,
}

/// Dashboard Data aggregation structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardData {
    pub btc_price: Option<MarketData>,
    pub total_market_cap: Option<f64>,
    pub fear_greed_index: Option<i32>,
    pub top_gainers: Vec<MarketData>,
    pub top_losers: Vec<MarketData>,
    pub recent_reports: Vec<ReportSummary>,
    pub last_updated: String,
}

/// Model Registry manages all data model definitions
pub struct ModelRegistry {
    /// Model definitions registry
    models: HashMap<String, serde_json::Value>,
    /// Model access statistics
    access_count: Arc<AtomicUsize>,
    /// Model validation count
    validation_count: Arc<AtomicUsize>,
}

impl ModelRegistry {
    /// Initialize the Model Registry
    pub async fn new() -> Result<Self> {
        println!("📊 Initializing Model Registry...");
        
        let models = Self::initialize_models().await?;
        
        println!("  📊 Registered {} data models", models.len());
        
        Ok(Self {
            models,
            access_count: Arc::new(AtomicUsize::new(0)),
            validation_count: Arc::new(AtomicUsize::new(0)),
        })
    }
    
    /// Initialize all model definitions
    async fn initialize_models() -> Result<HashMap<String, serde_json::Value>> {
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
        
        models.insert("ReportSummary".to_string(), serde_json::json!({
            "type": "object",
            "properties": {
                "id": {"type": "integer"},
                "title": {"type": "string"},
                "report_type": {"type": "string"},
                "status": {"type": "string"},
                "created_at": {"type": "string", "format": "date-time"},
                "summary": {"type": "string", "maxLength": 500}
            },
            "required": ["id", "title", "report_type", "status", "created_at"]
        }));
        
        models.insert("MarketData".to_string(), serde_json::json!({
            "type": "object",
            "properties": {
                "symbol": {"type": "string"},
                "price": {"type": "number"},
                "change_24h": {"type": "number"},
                "change_percent_24h": {"type": "number"},
                "volume_24h": {"type": "number"},
                "market_cap": {"type": "number"},
                "last_updated": {"type": "string", "format": "date-time"}
            },
            "required": ["symbol", "price", "last_updated"]
        }));
        
        models.insert("DashboardData".to_string(), serde_json::json!({
            "type": "object",
            "properties": {
                "btc_price": {"$ref": "#/definitions/MarketData"},
                "total_market_cap": {"type": "number"},
                "fear_greed_index": {"type": "integer", "minimum": 0, "maximum": 100},
                "top_gainers": {"type": "array", "items": {"$ref": "#/definitions/MarketData"}},
                "top_losers": {"type": "array", "items": {"$ref": "#/definitions/MarketData"}},
                "recent_reports": {"type": "array", "items": {"$ref": "#/definitions/ReportSummary"}},
                "last_updated": {"type": "string", "format": "date-time"}
            },
            "required": ["last_updated"]
        }));
        
        Ok(models)
    }
    
    /// Get a model definition by name
    pub async fn get_model_definition(&self, model_name: &str) -> Result<serde_json::Value> {
        self.access_count.fetch_add(1, Ordering::Relaxed);
        
        self.models.get(model_name)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Model '{}' not found", model_name))
    }
    
    /// Validate data against a model
    pub async fn validate_data(&self, model_name: &str, data: &serde_json::Value) -> Result<bool> {
        self.validation_count.fetch_add(1, Ordering::Relaxed);
        
        let model = self.get_model_definition(model_name).await?;
        
        // Basic validation logic - in a real implementation, use jsonschema crate
        match model.get("required") {
            Some(required_fields) => {
                if let Some(required_array) = required_fields.as_array() {
                    for field in required_array {
                        if let Some(field_name) = field.as_str() {
                            if !data.get(field_name).is_some() {
                                return Ok(false);
                            }
                        }
                    }
                }
            },
            None => {}
        }
        
        Ok(true)
    }
    
    /// Create a new instance of a model with default values
    pub async fn create_model_instance(&self, model_name: &str) -> Result<serde_json::Value> {
        match model_name {
            "Report" => Ok(serde_json::json!({
                "id": 0,
                "title": "",
                "content": "",
                "created_at": chrono::Utc::now().to_rfc3339(),
                "updated_at": chrono::Utc::now().to_rfc3339(),
                "report_type": "crypto",
                "status": "draft",
                "metadata": {}
            })),
            "MarketData" => Ok(serde_json::json!({
                "symbol": "",
                "price": 0.0,
                "change_24h": 0.0,
                "change_percent_24h": 0.0,
                "volume_24h": 0.0,
                "market_cap": 0.0,
                "last_updated": chrono::Utc::now().to_rfc3339()
            })),
            _ => Err(anyhow::anyhow!("Unknown model type: {}", model_name))
        }
    }
    
    /// Health check for model registry
    pub async fn health_check(&self) -> bool {
        // Verify that core models are available
        let core_models = vec!["Report", "MarketData", "DashboardData"];
        
        for model_name in core_models {
            if !self.models.contains_key(model_name) {
                eprintln!("❌ Core model '{}' is missing", model_name);
                return false;
            }
        }
        
        println!("✅ Model Registry health check passed");
        true
    }
    
    /// Get model registry statistics
    pub async fn get_statistics(&self) -> Result<serde_json::Value> {
        let total_accesses = self.access_count.load(Ordering::Relaxed);
        let total_validations = self.validation_count.load(Ordering::Relaxed);
        
        Ok(serde_json::json!({
            "component": "model_registry",
            "registered_models": self.models.len(),
            "total_accesses": total_accesses,
            "total_validations": total_validations,
            "available_models": self.models.keys().collect::<Vec<_>>()
        }))
    }
    
    /// List all available models
    pub async fn list_models(&self) -> Vec<String> {
        self.models.keys().cloned().collect()
    }
}
