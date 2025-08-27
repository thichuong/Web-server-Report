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
        .route("/api/crypto_reports/:id/sandboxed", get(api_sandboxed_report))
        .route("/api/health", get(api_health))
}

/// Dashboard summary API endpoint - Enhanced with Redis Streams
async fn api_dashboard_summary(
    State(service_islands): State<Arc<ServiceIslands>>
) -> Json<serde_json::Value> {
    // Phase 3: Primary reads from Redis Streams, DB as fallback
    println!("🚀 [API] Attempting stream-first dashboard summary fetch...");
    
    // Try cache manager first (primary storage)
    let app_state = &service_islands.app_state;
    if let Some(cache_system) = app_state.get_cache_system() {
        match cache_system.cache_manager().get("latest_market_data").await {
            Ok(Some(stream_data)) => {
                println!("✅ [API] Dashboard summary served from cache (<1ms)");
                return Json(stream_data);
            }
            Ok(None) => {
                println!("⚠️ [API] No data in cache, falling back to Layer 5...");
            }
            Err(e) => {
                println!("⚠️ [API] Cache read failed: {}, falling back to Layer 5...", e);
            }
        }
    }
    
    // Fallback to Service Islands Layer 5 + immediate stream storage
    match service_islands.crypto_reports.fetch_realtime_market_data().await {
        Ok(market_data) => {
            println!("✅ [API] Dashboard summary fetched via Layer 5 → Layer 3 → Layer 2");
            
            // Store in cache for future reads  
            let app_state = &service_islands.app_state;
            if let Some(cache_system) = app_state.get_cache_system() {
                use crate::service_islands::layer1_infrastructure::cache_system_island::cache_manager::CacheStrategy;
                if let Err(e) = cache_system.cache_manager().set_with_strategy(
                    "latest_market_data", 
                    market_data.clone(), 
                    CacheStrategy::ShortTerm
                ).await {
                    println!("⚠️ [API] Failed to store in cache: {}", e);
                } else {
                    println!("💾 [API] Data stored to cache for future reads");
                }
            }
            
            Json(market_data) // Return data in same format as WebSocket
        }
        Err(e) => {
            println!("❌ [API] Failed to fetch dashboard summary: {}", e);
            // Return fallback data in correct format (not wrapped in "dashboard")
            Json(json!({
                "btc_price_usd": 45000.0,
                "btc_change_24h": 0.0,
                "market_cap_usd": 2100000000000.0,
                "volume_24h_usd": 150000000000.0,
                "fng_value": 50,
                "rsi_14": 50.0,
                "data_sources": {},
                "fetch_duration_ms": 0,
                "partial_failure": true,
                "last_updated": chrono::Utc::now().to_rfc3339(),
                "timestamp": chrono::Utc::now().to_rfc3339()
            }))
        }
    }
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
    let language = params.get("lang").map(|s| s.as_str());

    // Use Service Islands to serve sandboxed content
    match service_islands.crypto_reports.handlers.serve_sandboxed_report(
        &service_islands.app_state,
        report_id,
        sandbox_token,
        language
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
