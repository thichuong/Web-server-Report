//! API Routes
//! 
//! This module handles all API endpoints for the Service Islands Architecture.
//! Includes dashboard APIs, cache APIs, health APIs, and rate limiting APIs.

use axum::{
    routing::get,
    Router,
    response::Json,
    extract::State
};
use serde_json::json;
use std::sync::Arc;

use crate::service_islands::ServiceIslands;

/// Configure API routes
pub fn configure_api_routes() -> Router<Arc<ServiceIslands>> {
    Router::new()
        .route("/api/crypto/dashboard-summary", get(api_dashboard_summary))
        .route("/api/crypto/dashboard-summary/cached", get(api_dashboard_summary_cached))
        .route("/api/crypto/dashboard-summary/refresh", get(api_dashboard_refresh))
        .route("/api/crypto/rate-limit-status", get(api_rate_limit_status))
        .route("/api/health", get(api_health))
        .route("/api/cache/stats", get(api_cache_stats))
}

/// Dashboard summary API endpoint
async fn api_dashboard_summary(
    State(_service_islands): State<Arc<ServiceIslands>>
) -> Json<serde_json::Value> {
    Json(json!({
        "dashboard": {
            "crypto": {
                "btc_price": 45000.0,
                "eth_price": 3200.0,
                "market_cap": "2.1T"
            },
            "status": "active",
            "last_updated": chrono::Utc::now().to_rfc3339()
        }
    }))
}

/// Cached dashboard summary API endpoint
async fn api_dashboard_summary_cached(
    State(_service_islands): State<Arc<ServiceIslands>>
) -> Json<serde_json::Value> {
    Json(json!({
        "dashboard": {
            "source": "cache",
            "crypto_data": "cached_results",
            "cache_hit": true
        }
    }))
}

/// Dashboard refresh API endpoint
async fn api_dashboard_refresh(
    State(_service_islands): State<Arc<ServiceIslands>>
) -> Json<serde_json::Value> {
    Json(json!({
        "message": "Dashboard refresh requested",
        "status": "refreshing"
    }))
}

/// Rate limit status API endpoint
async fn api_rate_limit_status(
    State(_service_islands): State<Arc<ServiceIslands>>
) -> Json<serde_json::Value> {
    Json(json!({
        "rate_limit": {
            "status": "active",
            "requests_remaining": 100,
            "reset_time": chrono::Utc::now().to_rfc3339()
        }
    }))
}

/// API health check endpoint
async fn api_health(
    State(service_islands): State<Arc<ServiceIslands>>
) -> Json<serde_json::Value> {
    let is_healthy = service_islands.health_check().await;
    Json(json!({
        "api": {
            "status": if is_healthy { "healthy" } else { "unhealthy" },
            "service_islands": 7,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }
    }))
}

/// Cache statistics API endpoint
async fn api_cache_stats(
    State(_service_islands): State<Arc<ServiceIslands>>
) -> Json<serde_json::Value> {
    Json(json!({
        "cache": {
            "l1_cache": {
                "hits": 1500,
                "misses": 300,
                "hit_rate": 0.83
            },
            "l2_cache": {
                "status": "active",
                "backend": "redis_fallback"
            }
        }
    }))
}
