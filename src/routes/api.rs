//! API Routes
//! 
//! This module handles all API endpoints for the Service Islands Architecture.
//! Includes dashboard APIs, cache APIs, health APIs, and rate limiting APIs.

use axum::{
    routing::get,
    Router,
    response::{Json, IntoResponse},
    extract::{State, Path, Query}
};
use serde_json::json;
use std::sync::Arc;
use std::collections::HashMap;

use crate::service_islands::ServiceIslands;

/// Configure API routes
pub fn configure_api_routes() -> Router<Arc<ServiceIslands>> {
    Router::new()
        .route("/api/crypto/dashboard-summary", get(api_dashboard_summary))
        .route("/api/dashboard/data", get(api_dashboard_data))
        .route("/api/crypto_reports/:id/sandboxed", get(api_sandboxed_report))
        .route("/api/health", get(api_health))
        .route("/api/websocket/stats", get(api_websocket_stats))
}

/// Dashboard data API endpoint - Enhanced with Redis Streams
/// Same functionality as api_dashboard_summary but with cleaner path
async fn api_dashboard_data(
    State(service_islands): State<Arc<ServiceIslands>>
) -> Json<serde_json::Value> {
    // Phase 3: Primary reads from Redis Streams, DB as fallback
    println!("🚀 [API] Attempting stream-first dashboard data fetch...");
    
    // Try cache manager first (populated by websocket service)
    let cache_system = &service_islands.cache_system;
    match cache_system.cache_manager().get("latest_market_data").await {
        Ok(Some(stream_data)) => {
            println!("✅ [API] Dashboard data served from cache (<1ms)");
            return Json(stream_data);
        }
        Ok(None) => {
            println!("⚠️ [API] No data in cache yet, websocket service will populate it soon");
        }
        Err(e) => {
            println!("⚠️ [API] Cache read failed: {}", e);
        }
    }

    // Return fallback data - websocket service will populate cache within 10 seconds
    println!("📊 [API] Returning fallback data (websocket service will update cache)");
    Json(json!({
        "btc_price_usd": 45000.0,
        "btc_change_24h": 0.0,
        "market_cap_usd": 2100000000000.0,
        "volume_24h_usd": 150000000000.0,
        "fng_value": 50,
        "btc_rsi_14": 50.0,
        "data_sources": {},
        "fetch_duration_ms": 0,
        "partial_failure": true,
        "last_updated": chrono::Utc::now().to_rfc3339(),
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "note": "Fallback data - fresh data will be available within 10 seconds"
    }))
}

/// Dashboard summary API endpoint - Reads from cache populated by websocket service
async fn api_dashboard_summary(
    State(service_islands): State<Arc<ServiceIslands>>
) -> Json<serde_json::Value> {
    // Cache-first strategy: Read from cache populated by websocket service
    println!("🚀 [API] Reading dashboard summary from cache...");

    // Try cache manager first (populated by websocket service)
    let cache_system = &service_islands.cache_system;
    match cache_system.cache_manager().get("latest_market_data").await {
        Ok(Some(stream_data)) => {
            println!("✅ [API] Dashboard summary served from cache (<1ms)");
            return Json(stream_data);
        }
        Ok(None) => {
            println!("⚠️ [API] No data in cache yet, websocket service will populate it soon");
        }
        Err(e) => {
            println!("⚠️ [API] Cache read failed: {}", e);
        }
    }

    // Return fallback data - websocket service will populate cache within 10 seconds
    println!("📊 [API] Returning fallback data (websocket service will update cache)");
    Json(json!({
        "btc_price_usd": 45000.0,
        "btc_change_24h": 0.0,
        "market_cap_usd": 2100000000000.0,
        "volume_24h_usd": 150000000000.0,
        "fng_value": 50,
        "btc_rsi_14": 50.0,
        "data_sources": {},
        "fetch_duration_ms": 0,
        "partial_failure": true,
        "last_updated": chrono::Utc::now().to_rfc3339(),
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "note": "Fallback data - fresh data will be available within 10 seconds"
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

/// Sandboxed report content API endpoint
/// 
/// Serves sanitized HTML content for iframe embedding with security headers
async fn api_sandboxed_report(
    Path(id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
    State(service_islands): State<Arc<ServiceIslands>>
) -> impl IntoResponse {
    println!("🔒 [API] Sandboxed report requested for ID: {}", id);
    
    // Parse report ID (-1 for latest)
    let report_id: i32 = if id == "latest" {
        -1
    } else {
        match id.parse() {
            Ok(id) => id,
            Err(_) => {
                println!("❌ [API] Invalid report ID format for sandboxing: {}", id);
                return "Invalid report ID format".into_response();
            }
        }
    };

    // Get sandbox token from query parameters
    let sandbox_token = match params.get("token") {
        Some(token) => token,
        None => {
            println!("❌ [API] Missing sandbox token for report {}", report_id);
            return "Missing sandbox token".into_response();
        }
    };

    // Get language parameter (optional, defaults to Vietnamese)
    // Note: Language switching is now handled dynamically inside iframe
    let initial_language = params.get("lang").map(|s| s.as_str());

    // Get chart modules content for iframe inclusion
    let chart_modules = match params.get("chart_modules") {
        Some(_) => {
            // If chart_modules parameter is present, load actual chart modules
            println!("📊 [API] Loading chart modules for iframe");
            Some(service_islands.shared_components.chart_modules_content.as_str())
        },
        None => {
            println!("⚠️ [API] No chart_modules parameter - iframe will have empty charts");
            None
        }
    };

    // Use Service Islands to serve sandboxed content
    match service_islands.crypto_reports.handlers.serve_sandboxed_report(
        &service_islands.get_legacy_app_state(),
        report_id,
        sandbox_token,
        initial_language,
        chart_modules
    ).await {
        Ok(response) => {
            println!("✅ [API] Sandboxed report {} served successfully", report_id);
            response
        }
        Err(e) => {
            eprintln!("❌ [API] Failed to serve sandboxed report {}: {}", report_id, e);
            "Failed to serve sandboxed content".into_response()
        }
    }
}

/// WebSocket statistics API endpoint
///
/// Note: WebSocket functionality is now in a separate service.
/// This endpoint returns a redirect message to the websocket service.
async fn api_websocket_stats(
    State(_service_islands): State<Arc<ServiceIslands>>
) -> Json<serde_json::Value> {
    Json(json!({
        "message": "WebSocket functionality has been moved to a separate service",
        "websocket_service": "Check Web-server-Report-websocket service for WebSocket stats",
        "websocket_health_endpoint": "http://localhost:8081/health",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}
