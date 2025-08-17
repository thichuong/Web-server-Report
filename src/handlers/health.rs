use axum::{
    extract::State,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use std::{sync::Arc, sync::atomic::Ordering, time::Instant};

use crate::state::AppState;

pub async fn health(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let start_time = Instant::now();
    
    let request_count = state.request_counter.load(Ordering::Relaxed);
    let report_cache_stats = state.report_cache.stats().await;
    let latest_id = state.cached_latest_id.load(Ordering::Relaxed);
    
    // Test SSL connectivity cho external APIs
    let ssl_check = test_ssl_connectivity().await;
    
    // Get unified cache stats
    let cache_stats = state.cache_manager.stats().await;
    let cache_health = state.cache_manager.health_check().await;
    
    // Record performance metrics
    let response_time = start_time.elapsed().as_millis() as u64;
    state.metrics.record_request(response_time);
    
    Json(serde_json::json!({
        "status": "healthy", 
        "message": "Crypto Dashboard Rust server with Unified Cache Manager",
        "ssl_status": ssl_check,
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "metrics": {
            "total_requests": request_count,
            "cache_size": report_cache_stats.entries,
            "latest_report_id": latest_id,
            "available_cpus": num_cpus::get(),
            "thread_pool_active": true,
            "avg_response_time_ms": state.metrics.avg_response_time(),
            "cache_hit_rate": state.report_cache.hit_rate()
        },
        "cache_system": {
            "type": "unified_multi_tier",
            "l1_entries": cache_stats.l1_entry_count,
            "l1_hit_count": cache_stats.l1_hit_count,
            "l1_miss_count": cache_stats.l1_miss_count,
            "l1_hit_rate": cache_stats.l1_hit_rate,
            "l2_healthy": cache_health.l2_healthy,
            "overall_healthy": cache_health.overall_healthy
        }
    }))
}

pub async fn performance_metrics(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let request_count = state.request_counter.load(Ordering::Relaxed);
    
    Json(serde_json::json!({
        "performance": {
            "total_requests": request_count,
            "avg_response_time_ms": state.metrics.avg_response_time(),
            "cache_size": state.report_cache.stats().await.entries,
            "cache_hit_rate": state.report_cache.hit_rate(),
        },
        "system": {
            "available_cpus": num_cpus::get(),
            "thread_pool_active": true,
        }
    }))
}

// Test SSL connectivity đến các external APIs
async fn test_ssl_connectivity() -> serde_json::Value {
    let client = crate::performance::OPTIMIZED_HTTP_CLIENT.clone();
    let mut results = serde_json::Map::new();
    
    // Test các endpoints chính
    let test_urls = vec![
        ("coingecko_global", "https://api.coingecko.com/api/v3/ping"),
        ("coingecko_price", "https://api.coingecko.com/api/v3/ping"),
        ("fear_greed", "https://api.alternative.me/"),
        ("taapi", "https://api.taapi.io/"),
    ];
    
    for (name, url) in test_urls {
        let result = match tokio::time::timeout(
            std::time::Duration::from_secs(5),
            client.get(url).send()
        ).await {
            Ok(Ok(response)) => json!({
                "status": "ok",
                "http_status": response.status().as_u16(),
                "ssl_version": "TLS 1.2+"
            }),
            Ok(Err(e)) => json!({
                "status": "error",
                "error": format!("SSL/HTTP error: {}", e)
            }),
            Err(_) => json!({
                "status": "timeout",
                "error": "Request timeout (5s)"
            })
        };
        
        results.insert(name.to_string(), result);
    }
    
    json!(results)
}
