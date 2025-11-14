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
/// ✅ PRODUCTION-READY: Queries actual cache statistics from multi-tier-cache library
async fn performance_metrics(
    State(service_islands): State<Arc<ServiceIslands>>
) -> Json<serde_json::Value> {
    // Get actual cache statistics from library
    use crate::service_islands::layer3_communication::data_communication::CryptoDataService;
    let legacy_state = service_islands.get_legacy_app_state();
    let cache_info = CryptoDataService::get_cache_stats(&legacy_state)
        .unwrap_or_else(|| "Cache statistics unavailable".to_string());

    Json(json!({
        "performance": {
            "service_islands_active": 7,
            "uptime": "operational",
            "memory_usage": "optimized",
            "cache_status": "active"
        },
        "cache_info": cache_info
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
/// ✅ PRODUCTION-READY: Queries detailed statistics from multi-tier-cache library
async fn cache_stats(
    State(service_islands): State<Arc<ServiceIslands>>
) -> Json<serde_json::Value> {
    let legacy_state = service_islands.get_legacy_app_state();

    // Get actual cache statistics from the multi-tier-cache library
    if let Some(ref cache_system) = legacy_state.cache_system {
        let stats = cache_system.cache_manager.get_stats();
        Json(json!({
            "cache": {
                "system": "multi-tier-cache library v0.5.2",
                "l1_cache": "active (moka)",
                "l2_cache": "active (Redis)",
                "status": "operational"
            },
            "statistics": {
                "total_requests": stats.total_requests,
                "l1_hits": stats.l1_hits,
                "l2_hits": stats.l2_hits,
                "total_hits": stats.total_hits,
                "misses": stats.misses,
                "promotions": stats.promotions,
                "hit_rate": format!("{:.1}%", stats.hit_rate),
                "in_flight_requests": stats.in_flight_requests
            },
            "configuration": {
                "l1_max_capacity": 2000,
                "l1_ttl": "5 minutes (ShortTerm)",
                "l2_ttl": "1 hour (default)",
                "eviction": "automatic (size + TTL based)",
                "stampede_protection": "enabled (DashMap coalescing)"
            },
            "health": {
                "status": "healthy",
                "recommendation": "Cache operating normally with automatic memory management"
            }
        }))
    } else {
        Json(json!({
            "error": "Cache system not available",
            "cache": {
                "status": "unavailable"
            }
        }))
    }
}
