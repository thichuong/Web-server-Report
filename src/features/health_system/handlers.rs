//! Health System Integration - Handler Integration Layer
//!
//! Provides Axum handler functions that integrate the health_system Service Island
//! with the existing web server architecture.

use crate::features::health_system::HealthSystem;
use crate::state::AppState;
use axum::{
    extract::State,
    response::{IntoResponse, Json},
};
use serde_json::Value;
use std::sync::Arc;
use tokio::time::Instant;

/// Health check endpoint using the health_system Service Island
pub async fn health(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let start_time = Instant::now();
    
    // Create health system instance (TODO: Move to AppState for persistence)
    let health_system = HealthSystem::new();
    
    // Record request for metrics
    let response_time = start_time.elapsed().as_millis() as u64;
    state.metrics.record_request(response_time);
    
    // Get comprehensive health status
    let mut health_data = health_system.get_system_health().await;
    
    // Add actual cache statistics from AppState
    if let Some(health_obj) = health_data.as_object_mut() {
        // Add performance metrics from AppState
        health_obj.insert("request_count".to_string(), 
            serde_json::json!(state.metrics.request_count.load(std::sync::atomic::Ordering::Relaxed))
        );
        health_obj.insert("avg_response_time_ms".to_string(), 
            serde_json::json!(state.metrics.avg_response_time())
        );
        
        // Add actual cache statistics if available
        if let Some(ref cache) = state.cache {
            let stats = cache.stats().await;
            health_obj.insert("cache_stats".to_string(), serde_json::json!({
                "l1_cache": {
                    "entries": stats.entries,
                    "hits": stats.hits,
                    "misses": stats.misses,
                    "hit_rate": stats.hit_rate
                },
                "l2_cache": {
                    "connected": true,
                    "hits": stats.hits,
                    "misses": stats.misses
                }
            }));
        }
    }
    
    Json(health_data)
}

/// Performance metrics endpoint using the health_system Service Island
pub async fn performance_metrics(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    // Create health system instance (TODO: Move to AppState for persistence)
    let health_system = HealthSystem::new();
    
    // Get performance metrics
    let mut metrics = health_system.get_performance_metrics().await;
    
    // Enhance with actual AppState metrics
    if let Some(metrics_obj) = metrics.as_object_mut() {
        if let Some(performance) = metrics_obj.get_mut("performance").and_then(|p| p.as_object_mut()) {
            performance.insert("total_requests".to_string(),
                serde_json::json!(state.metrics.request_count.load(std::sync::atomic::Ordering::Relaxed))
            );
            performance.insert("avg_response_time_ms".to_string(),
                serde_json::json!(state.metrics.avg_response_time())
            );
        }
    }
    
    Json(metrics)
}

/// SSL connectivity test endpoint using the health_system Service Island
pub async fn ssl_connectivity_test() -> impl IntoResponse {
    // Create health system instance (TODO: Move to AppState for persistence)
    let health_system = HealthSystem::new();
    
    // Test connectivity to external services
    let connectivity_results = health_system.test_connectivity().await;
    
    Json(connectivity_results)
}

/// Comprehensive metrics endpoint combining all health system capabilities
pub async fn comprehensive_metrics(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    // Create health system instance (TODO: Move to AppState for persistence)
    let health_system = HealthSystem::new();
    
    // Get all metrics
    let aggregated_metrics = health_system.get_aggregated_metrics().await;
    
    Json(aggregated_metrics)
}

/// Routes for health system endpoints
pub fn health_system_routes() -> axum::Router<Arc<AppState>> {
    use axum::routing::get;
    
    axum::Router::new()
        .route("/health", get(health))
        .route("/health/metrics", get(performance_metrics))
        .route("/health/ssl", get(ssl_connectivity_test))
        .route("/health/comprehensive", get(comprehensive_metrics))
}
