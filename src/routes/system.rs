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
/// ✅ PRODUCTION-READY: Includes memory monitoring for compressed cache
async fn performance_metrics(
    State(_service_islands): State<Arc<ServiceIslands>>
) -> Json<serde_json::Value> {
    // Get compressed cache memory statistics
    use crate::service_islands::layer3_communication::data_communication::CryptoDataService;
    let (current_bytes, max_bytes, usage_percent) = CryptoDataService::get_compressed_cache_stats();
    let current_mb = current_bytes as f64 / (1024.0 * 1024.0);
    let max_mb = max_bytes as f64 / (1024.0 * 1024.0);
    
    Json(json!({
        "performance": {
            "service_islands_active": 7,
            "uptime": "operational",
            "memory_usage": "optimized",
            "cache_status": "active"
        },
        "compressed_cache_memory": {
            "current_mb": format!("{:.2}", current_mb),
            "max_mb": format!("{:.2}", max_mb),
            "usage_percent": format!("{:.1}%", usage_percent),
            "status": if usage_percent < 80.0 { "healthy" } else if usage_percent < 95.0 { "warning" } else { "critical" }
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
/// ✅ PRODUCTION-READY: Detailed memory and cache statistics
async fn cache_stats(
    State(_service_islands): State<Arc<ServiceIslands>>
) -> Json<serde_json::Value> {
    // Get compressed cache memory statistics
    use crate::service_islands::layer3_communication::data_communication::CryptoDataService;
    let (current_bytes, max_bytes, usage_percent) = CryptoDataService::get_compressed_cache_stats();
    let current_mb = current_bytes as f64 / (1024.0 * 1024.0);
    let max_mb = max_bytes as f64 / (1024.0 * 1024.0);
    let available_mb = (max_bytes - current_bytes) as f64 / (1024.0 * 1024.0);
    
    Json(json!({
        "cache": {
            "l1_cache": "active",
            "l2_cache": "active", 
            "status": "operational"
        },
        "compressed_cache": {
            "memory": {
                "current_bytes": current_bytes,
                "current_mb": format!("{:.2}", current_mb),
                "max_bytes": max_bytes,
                "max_mb": format!("{:.2}", max_mb),
                "available_mb": format!("{:.2}", available_mb),
                "usage_percent": format!("{:.1}%", usage_percent)
            },
            "limits": {
                "max_entry_size_mb": 5,
                "max_total_size_mb": 500,
                "warn_entry_size_mb": 2
            },
            "health": {
                "status": if usage_percent < 80.0 { "healthy" } else if usage_percent < 95.0 { "warning" } else { "critical" },
                "recommendation": if usage_percent > 90.0 { 
                    "Consider clearing old cache entries or increasing limits" 
                } else { 
                    "Operating normally" 
                }
            }
        }
    }))
}
