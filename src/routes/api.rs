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
    responses::{ApiHealthInfo, ApiHealthResponse, WebSocketStatsResponse},
};
use crate::state::AppState;

/// Configure API routes
pub fn configure_api_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/api/crypto_reports/{id}/shadow_dom",
            get(api_shadow_dom_content),
        )
        .route("/api/health", get(api_health))
        .route("/api/websocket/stats", get(api_websocket_stats))
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

/// Shadow DOM content endpoint for Declarative Shadow DOM architecture
///
/// Returns HTML fragment for embedding within <template shadowrootmode="open">
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

    // Use Service Islands to serve Shadow DOM content
    match state
        .crypto_handlers
        .serve_shadow_dom_content(
            &state,
            report_id,
            shadow_dom_token,
            initial_language,
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
