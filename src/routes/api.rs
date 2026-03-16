//! API Routes
//!
//! This module handles all API endpoints for the Service Islands Architecture.
//! Includes dashboard APIs, cache APIs, health APIs, and rate limiting APIs.

use axum::{
    Router,
    extract::{Path, Query, State},
    response::{IntoResponse, Json},
    routing::get,
};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use crate::dto::{
    HealthStatus,
    responses::{ApiHealthInfo, ApiHealthResponse, DashboardDataResponse, WebSocketStatsResponse},
};
use crate::state::AppState;

/// Configure API routes
pub fn configure_api_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/crypto/dashboard-summary", get(api_dashboard_summary))
        .route("/api/dashboard/data", get(api_dashboard_data))
        .route(
            "/api/crypto_reports/{id}/sandboxed",
            get(api_sandboxed_report),
        )
        .route(
            "/api/crypto_reports/{id}/shadow_dom",
            get(api_shadow_dom_content),
        )
        .route("/api/health", get(api_health))
        .route("/api/websocket/stats", get(api_websocket_stats))
}

/// Dashboard data API endpoint - Enhanced with Redis Streams
/// Same functionality as `api_dashboard_summary` but with cleaner path
async fn api_dashboard_data(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let cache_key = "api_dashboard_data_json";
    let mut cache_hit = "MISS";

    let response_data = state
        .cache_manager
        .get_or_compute_typed(
            cache_key,
            multi_tier_cache::CacheStrategy::RealTime,
            || async {
                debug!("🔍 [API] Cache MISS - reading from Redis Stream...");
                // Phase 3: Primary reads from Redis Streams via RedisStreamReader
                if let Ok(Some(data)) = state.redis_stream_reader.read_latest_market_data().await {
                    debug!("✅ [API] Data fetched from Redis Stream");
                    if let Ok(typed_data) =
                        serde_json::from_value::<DashboardDataResponse>(data)
                    {
                        return Ok(typed_data);
                    }
                }

                // Return fallback data if stream empty or failed
                Ok(get_fallback_dashboard_data())
            },
        )
        .await
        .unwrap_or_else(|_| {
            cache_hit = "ERROR";
            get_fallback_dashboard_data()
        });

    if cache_hit != "ERROR" && state.cache_manager.get(cache_key).await.is_ok() {
        cache_hit = "HIT";
    }

    (
        [("x-cache", cache_hit)],
        Json(response_data)
    )
}

fn get_fallback_dashboard_data() -> DashboardDataResponse {
    DashboardDataResponse {
        btc_price_usd: 96000.0,
        btc_change_24h: 0.0,
        btc_market_cap_percentage: 57.0,
        btc_rsi_14: 50.0,
        eth_price_usd: 3170.0,
        eth_change_24h: 0.0,
        eth_market_cap_percentage: 11.4,
        bnb_price_usd: 928.0,
        bnb_change_24h: 0.0,
        sol_price_usd: 142.0,
        sol_change_24h: 0.0,
        xrp_price_usd: 2.28,
        xrp_change_24h: 0.0,
        ada_price_usd: 0.51,
        ada_change_24h: 0.0,
        link_price_usd: 14.17,
        link_change_24h: 0.0,
        market_cap_usd: 3_340_000_000_000.0,
        market_cap_change_percentage_24h_usd: 0.0,
        volume_24h_usd: 260_000_000_000.0,
        fng_value: 50,
        us_stock_indices: std::collections::HashMap::new(),
        fetch_duration_ms: 0,
        partial_failure: true,
        last_updated: chrono::Utc::now().to_rfc3339(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        note: Some("Fallback data - fresh data will be available within 10 seconds".to_string()),
    }
}

/// Dashboard summary API endpoint - Reads from Redis Stream via `RedisStreamReader`
async fn api_dashboard_summary(state: State<Arc<AppState>>) -> impl IntoResponse {
    api_dashboard_data(state).await
}

/// API health check endpoint
async fn api_health(State(state): State<Arc<AppState>>) -> Json<ApiHealthResponse> {
    let is_healthy = state.health_check().await;

    let response = ApiHealthResponse {
        api: ApiHealthInfo {
            status: if is_healthy {
                HealthStatus::Healthy
            } else {
                HealthStatus::Unhealthy
            },
            services: 5,
            timestamp: chrono::Utc::now().to_rfc3339(),
        },
    };

    Json(response)
}

/// Sandboxed report content API endpoint
///
/// Serves sanitized HTML content for iframe embedding with security headers
async fn api_sandboxed_report(
    Path(id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    debug!("🔒 [API] Sandboxed report requested for ID: {}", id);

    // Parse report ID (-1 for latest)
    let report_id: i32 = if id == "latest" {
        -1
    } else if let Ok(id) = id.parse() {
        id
    } else {
        error!("❌ [API] Invalid report ID format for sandboxing: {}", id);
        return "Invalid report ID format".into_response();
    };

    // Get sandbox token from query parameters
    let Some(sandbox_token) = params.get("token") else {
        warn!("❌ [API] Missing sandbox token for report {}", report_id);
        return "Missing sandbox token".into_response();
    };

    // Get language parameter (optional, defaults to Vietnamese)
    // Note: Language switching is now handled dynamically inside iframe
    let initial_language = params.get("lang").map(std::string::String::as_str);

    // Get chart modules content for iframe inclusion
    let chart_modules = if params.contains_key("chart_modules") {
        // If chart_modules parameter is present, load actual chart modules
        debug!("📊 [API] Loading chart modules for iframe");
        Some(state.chart_modules_content.as_str())
    } else {
        warn!("⚠️ [API] No chart_modules parameter - iframe will have empty charts");
        None
    };

    // Use Service Islands to serve sandboxed content
    match state
        .crypto_handlers
        .serve_sandboxed_report(
            &state,
            report_id,
            sandbox_token,
            initial_language,
            chart_modules,
        )
        .await
    {
        Ok(response) => {
            info!(
                "✅ [API] Sandboxed report {} served successfully",
                report_id
            );
            response
        }
        Err(e) => {
            error!(
                "❌ [API] Failed to serve sandboxed report {}: {}",
                report_id, e
            );
            "Failed to serve sandboxed content".into_response()
        }
    }
}

/// Shadow DOM content endpoint for Declarative Shadow DOM architecture
///
/// Returns HTML fragment for embedding within <template shadowrootmode="open">
/// This is the modern replacement for `api_sandboxed_report`
async fn api_shadow_dom_content(
    Path(id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    debug!("🌓 [API] Shadow DOM content requested for ID: {}", id);

    // Parse report ID (-1 for latest)
    let report_id: i32 = if id == "latest" {
        -1
    } else if let Ok(id) = id.parse() {
        id
    } else {
        error!("❌ [API] Invalid report ID format for Shadow DOM: {}", id);
        return "Invalid report ID format".into_response();
    };

    // Get shadow DOM token from query parameters
    let Some(shadow_dom_token) = params.get("token") else {
        warn!("❌ [API] Missing shadow DOM token for report {}", report_id);
        return "Missing shadow DOM token".into_response();
    };

    // Get language parameter (optional, defaults to Vietnamese)
    let initial_language = params.get("lang").map(std::string::String::as_str);

    // Get chart modules content for Shadow DOM inclusion
    let chart_modules = if params.contains_key("chart_modules") {
        // If chart_modules parameter is present, load actual chart modules
        debug!("📊 [API] Loading chart modules for Shadow DOM");
        Some(state.chart_modules_content.as_str())
    } else {
        debug!("💡 [API] No chart_modules parameter - using default behavior");
        Some(state.chart_modules_content.as_str())
    };

    // Use Service Islands to serve Shadow DOM content
    match state
        .crypto_handlers
        .serve_shadow_dom_content(
            &state,
            report_id,
            shadow_dom_token,
            initial_language,
            chart_modules,
        )
        .await
    {
        Ok(response) => {
            info!(
                "✅ [API] Shadow DOM content for report {} served successfully",
                report_id
            );
            response
        }
        Err(e) => {
            error!(
                "❌ [API] Failed to serve Shadow DOM content for report {}: {}",
                report_id, e
            );
            "Failed to serve Shadow DOM content".into_response()
        }
    }
}

/// WebSocket statistics API endpoint
///
/// Note: WebSocket functionality is now in a separate service.
/// This endpoint returns a redirect message to the websocket service.
async fn api_websocket_stats(State(_state): State<Arc<AppState>>) -> Json<WebSocketStatsResponse> {
    let response = WebSocketStatsResponse {
        message: "WebSocket functionality has been moved to a separate service".to_string(),
        websocket_service: "Check Web-server-Report-websocket service for WebSocket stats"
            .to_string(),
        websocket_health_endpoint: "http://localhost:8081/health".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    Json(response)
}
