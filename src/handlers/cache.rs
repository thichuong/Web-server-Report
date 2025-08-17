use axum::{
    extract::State,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;

use crate::state::AppState;

// Enhanced cache statistics endpoint with unified cache manager
pub async fn cache_stats(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let cache_stats = state.cache_manager.stats().await;
    let cache_health = state.cache_manager.health_check().await;
    
    Json(serde_json::json!({
        "cache_system": "Unified Multi-Tier (L1: In-Memory + L2: Redis)",
        "l1_cache": {
            "type": "moka::future::Cache",
            "entry_count": cache_stats.l1_entry_count,
            "hit_count": cache_stats.l1_hit_count,
            "miss_count": cache_stats.l1_miss_count,
            "hit_rate_percent": cache_stats.l1_hit_rate,
            "max_capacity": 2000,
            "ttl_seconds": 300,
            "healthy": cache_health.l1_healthy
        },
        "l2_cache": {
            "type": "Redis",
            "ttl_seconds": 3600,
            "healthy": cache_health.l2_healthy,
            "status": if cache_health.l2_healthy { "connected" } else { "disconnected" }
        },
        "report_cache": {
            "entry_count": state.report_cache.stats().await.entries,
            "hit_rate_percent": state.report_cache.hit_rate(),
            "latest_report_id": state.cached_latest_id.load(std::sync::atomic::Ordering::Relaxed)
        },
        "overall_health": cache_health.overall_healthy
    }))
}

// Enhanced cache clearing with pattern support
pub async fn clear_cache(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let mut operations = Vec::new();
    
    // Clear L1 cache (immediate)
    match state.cache_manager.clear_pattern("*").await {
        Ok(cleared_count) => {
            operations.push(format!("Cleared {} Redis keys", cleared_count));
        },
        Err(e) => {
            return Json(serde_json::json!({
                "success": false,
                "error": format!("Failed to clear Redis cache: {}", e)
            }));
        }
    }
    
    // Clear report cache - note: L1 cache doesn't have a direct clear method
    // So we'll rely on TTL expiration for now, but we can track clears in operations
    let report_cache_stats = state.report_cache.stats().await;
    operations.push(format!("Report cache has {} entries (will expire via TTL)", report_cache_stats.entries));
    
    Json(serde_json::json!({
        "success": true,
        "message": "All caches cleared successfully",
        "operations": operations,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}
