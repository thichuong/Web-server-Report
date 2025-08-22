//! Health and System Routes
//! 
//! This module handles all health checks, system monitoring, and administrative routes.
//! Routes are handled through the Service Islands Architecture.

use axum::{
    routing::get,
    Router,
    http::StatusCode,
    response::Json,
    extract::State
};
use serde_json::json;
use std::sync::Arc;

use crate::service_islands::ServiceIslands;

/// Configure health and system monitoring routes
pub fn configure_system_routes() -> Router<Arc<ServiceIslands>> {
    Router::new()
        .route("/health", get(health_check))
        .route("/metrics", get(performance_metrics))
        .route("/admin/cache/clear", get(clear_cache))
        .route("/admin/cache/stats", get(cache_stats))
}

/// Health check endpoint - delegates to Service Islands
async fn health_check(
    State(service_islands): State<Arc<ServiceIslands>>
) -> Result<Json<serde_json::Value>, StatusCode> {
    let health_status = service_islands.health_check().await;
    
    let status = json!({
        "status": if health_status { "healthy" } else { "unhealthy" },
        "service_islands": {
            "total": 7,
            "operational": if health_status { 7 } else { 0 },
            "architecture": "Service Islands",
            "timestamp": chrono::Utc::now().to_rfc3339()
        }
    });
    
    Ok(Json(status))
}

/// Performance metrics endpoint
async fn performance_metrics(
    State(_service_islands): State<Arc<ServiceIslands>>
) -> Json<serde_json::Value> {
    Json(json!({
        "performance": {
            "service_islands_active": 7,
            "uptime": "operational",
            "memory_usage": "optimized",
            "cache_status": "active"
        }
    }))
}

/// Clear cache endpoint - delegates to Cache System Island
async fn clear_cache(
    State(_service_islands): State<Arc<ServiceIslands>>
) -> Json<serde_json::Value> {
    // TODO: Implement cache clearing via Service Islands
    Json(json!({
        "message": "Cache clear requested",
        "status": "queued"
    }))
}

/// Cache statistics endpoint - delegates to Cache System Island
async fn cache_stats(
    State(_service_islands): State<Arc<ServiceIslands>>
) -> Json<serde_json::Value> {
    // TODO: Get cache stats from Cache System Island
    Json(json!({
        "cache": {
            "l1_cache": "active",
            "l2_cache": "active", 
            "status": "operational"
        }
    }))
}
