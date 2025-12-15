//! API Routes
//!
//! This module handles all API endpoints for the Service Islands Architecture.
//! Includes dashboard APIs, cache APIs, health APIs, and rate limiting APIs.

use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use crate::dto::{
    responses::{ApiHealthInfo, ApiHealthResponse, DashboardDataResponse, WebSocketStatsResponse},
    HealthStatus,
};
use crate::service_islands::ServiceIslands;

/// Configure API routes
pub fn configure_api_routes() -> Router<Arc<ServiceIslands>> {
    Router::new()
        .route("/api/crypto/dashboard-summary", get(api_dashboard_summary))
        .route("/api/dashboard/data", get(api_dashboard_data))
        .route(
            "/api/crypto_reports/:id/sandboxed",
            get(api_sandboxed_report),
        )
        .route(
            "/api/crypto_reports/:id/shadow_dom",
            get(api_shadow_dom_content),
        )
        .route("/api/health", get(api_health))
        .route("/api/websocket/stats", get(api_websocket_stats))
}

/// Dashboard data API endpoint - Enhanced with Redis Streams
/// Same functionality as `api_dashboard_summary` but with cleaner path
async fn api_dashboard_data(
    State(service_islands): State<Arc<ServiceIslands>>,
) -> Json<DashboardDataResponse> {
    // Phase 3: Primary reads from Redis Streams via RedisStreamReader
    debug!("🚀 [API] Reading dashboard data from Redis Stream...");

    // Use RedisStreamReader to fetch latest market data
    match service_islands
        .redis_stream_reader
        .read_latest_market_data()
        .await
    {
        Ok(Some(data)) => {
            debug!("✅ [API] Dashboard data served from Redis Stream (<1ms)");
            // Deserialize the Value into our typed response
            if let Ok(typed_data) = serde_json::from_value::<DashboardDataResponse>(data) {
                return Json(typed_data);
            }
            warn!("⚠️ [API] Failed to deserialize Redis Stream data, using fallback");
        }
        Ok(None) => {
            warn!("⚠️ [API] No data in Redis Stream yet, websocket service will populate it soon");
        }
        Err(e) => {
            warn!("⚠️ [API] Failed to read from Redis Stream: {}", e);
        }
    }

    // Return fallback data - websocket service will populate stream within 10 seconds
    info!("📊 [API] Returning fallback data (websocket service will update stream)");
    let fallback_response = DashboardDataResponse {
        // Bitcoin
        btc_price_usd: 96000.0,
        btc_change_24h: 0.0,
        btc_market_cap_percentage: 57.0,
        btc_rsi_14: 50.0,

        // Ethereum
        eth_price_usd: 3170.0,
        eth_change_24h: 0.0,
        eth_market_cap_percentage: 11.4,

        // BNB
        bnb_price_usd: 928.0,
        bnb_change_24h: 0.0,

        // Solana
        sol_price_usd: 142.0,
        sol_change_24h: 0.0,

        // XRP
        xrp_price_usd: 2.28,
        xrp_change_24h: 0.0,

        // Cardano
        ada_price_usd: 0.51,
        ada_change_24h: 0.0,

        // Chainlink
        link_price_usd: 14.17,
        link_change_24h: 0.0,

        // Market metrics
        market_cap_usd: 3_340_000_000_000.0,
        market_cap_change_percentage_24h_usd: 0.0,
        volume_24h_usd: 260_000_000_000.0,

        // Fear & Greed Index
        fng_value: 50,

        // US Stock Indices
        us_stock_indices: HashMap::new(),

        // Metadata
        fetch_duration_ms: 0,
        partial_failure: true,
        last_updated: chrono::Utc::now().to_rfc3339(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        note: Some("Fallback data - fresh data will be available within 10 seconds".to_string()),
    };
    Json(fallback_response)
}

/// Dashboard summary API endpoint - Reads from Redis Stream via `RedisStreamReader`
async fn api_dashboard_summary(
    State(service_islands): State<Arc<ServiceIslands>>,
) -> Json<DashboardDataResponse> {
    // Stream-first strategy: Read from Redis Stream populated by websocket service
    debug!("🚀 [API] Reading dashboard summary from Redis Stream...");

    // Use RedisStreamReader to fetch latest market data
    match service_islands
        .redis_stream_reader
        .read_latest_market_data()
        .await
    {
        Ok(Some(data)) => {
            debug!("✅ [API] Dashboard summary served from Redis Stream (<1ms)");
            // Deserialize the Value into our typed response
            if let Ok(typed_data) = serde_json::from_value::<DashboardDataResponse>(data) {
                return Json(typed_data);
            }
            warn!("⚠️ [API] Failed to deserialize Redis Stream data, using fallback");
        }
        Ok(None) => {
            warn!("⚠️ [API] No data in Redis Stream yet, websocket service will populate it soon");
        }
        Err(e) => {
            warn!("⚠️ [API] Failed to read from Redis Stream: {}", e);
        }
    }

    // Return fallback data - websocket service will populate stream within 10 seconds
    info!("📊 [API] Returning fallback data (websocket service will update stream)");
    let fallback_response = DashboardDataResponse {
        // Bitcoin
        btc_price_usd: 96000.0,
        btc_change_24h: 0.0,
        btc_market_cap_percentage: 57.0,
        btc_rsi_14: 50.0,

        // Ethereum
        eth_price_usd: 3170.0,
        eth_change_24h: 0.0,
        eth_market_cap_percentage: 11.4,

        // BNB
        bnb_price_usd: 928.0,
        bnb_change_24h: 0.0,

        // Solana
        sol_price_usd: 142.0,
        sol_change_24h: 0.0,

        // XRP
        xrp_price_usd: 2.28,
        xrp_change_24h: 0.0,

        // Cardano
        ada_price_usd: 0.51,
        ada_change_24h: 0.0,

        // Chainlink
        link_price_usd: 14.17,
        link_change_24h: 0.0,

        // Market metrics
        market_cap_usd: 3_340_000_000_000.0,
        market_cap_change_percentage_24h_usd: 0.0,
        volume_24h_usd: 260_000_000_000.0,

        // Fear & Greed Index
        fng_value: 50,

        // US Stock Indices
        us_stock_indices: HashMap::new(),

        // Metadata
        fetch_duration_ms: 0,
        partial_failure: true,
        last_updated: chrono::Utc::now().to_rfc3339(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        note: Some("Fallback data - fresh data will be available within 10 seconds".to_string()),
    };
    Json(fallback_response)
}

/// API health check endpoint
async fn api_health(State(service_islands): State<Arc<ServiceIslands>>) -> Json<ApiHealthResponse> {
    let is_healthy = service_islands.health_check().await;

    let response = ApiHealthResponse {
        api: ApiHealthInfo {
            status: if is_healthy {
                HealthStatus::Healthy
            } else {
                HealthStatus::Unhealthy
            },
            service_islands: 7,
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
    State(service_islands): State<Arc<ServiceIslands>>,
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
        Some(
            service_islands
                .shared_components
                .chart_modules_content
                .as_str(),
        )
    } else {
        warn!("⚠️ [API] No chart_modules parameter - iframe will have empty charts");
        None
    };

    // Use Service Islands to serve sandboxed content
    match service_islands
        .crypto_reports
        .handlers
        .serve_sandboxed_report(
            &service_islands.get_legacy_app_state(),
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
    State(service_islands): State<Arc<ServiceIslands>>,
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
        Some(
            service_islands
                .shared_components
                .chart_modules_content
                .as_str(),
        )
    } else {
        debug!("💡 [API] No chart_modules parameter - using default behavior");
        Some(
            service_islands
                .shared_components
                .chart_modules_content
                .as_str(),
        )
    };

    // Use Service Islands to serve Shadow DOM content
    match service_islands
        .crypto_reports
        .handlers
        .serve_shadow_dom_content(
            &service_islands.get_legacy_app_state(),
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
async fn api_websocket_stats(
    State(_service_islands): State<Arc<ServiceIslands>>,
) -> Json<WebSocketStatsResponse> {
    let response = WebSocketStatsResponse {
        message: "WebSocket functionality has been moved to a separate service".to_string(),
        websocket_service: "Check Web-server-Report-websocket service for WebSocket stats"
            .to_string(),
        websocket_health_endpoint: "http://localhost:8081/health".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    Json(response)
}
