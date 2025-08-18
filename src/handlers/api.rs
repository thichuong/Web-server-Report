use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use std::{sync::Arc, sync::atomic::Ordering, time::Instant};

use crate::state::AppState;

// Dashboard summary API endpoint with unified cache
pub async fn api_dashboard_summary(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let start_time = Instant::now();
    
    match state.data_service.fetch_dashboard_summary().await {
        Ok(summary) => {
            let response_time = start_time.elapsed().as_millis() as u64;
            state.metrics.record_request(response_time);
            state.request_counter.fetch_add(1, Ordering::Relaxed);

            // Return the raw summary object (frontend expects top-level fields like market_cap)
            Json(summary).into_response()
        },
        Err(e) => {
            eprintln!("❌ Dashboard summary API error: {}", e);

            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({
                    "error": "Failed to fetch dashboard data",
                    "details": e.to_string()
                }))
            ).into_response()
        }
    }
}

// API endpoint to get cached dashboard summary với intelligent fallback
pub async fn dashboard_summary_api(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match state.websocket_service.get_dashboard_data_with_fallback().await {
        Ok(data) => Json(data).into_response(),
        Err(e) => {
            eprintln!("Failed to fetch dashboard data with fallback: {}", e);
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({
                    "error": "Service temporarily unavailable",
                    "message": format!("Unable to fetch dashboard data: {}", e),
                    "suggestion": "Dashboard data may be temporarily unavailable due to API rate limits. Please try again in a few minutes."
                }))
            ).into_response()
        }
    }
}

// API endpoint to force refresh dashboard data
pub async fn force_refresh_dashboard(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match state.websocket_service.force_update_dashboard().await {
        Ok(data) => Json(json!({
            "status": "success",
            "message": "Dashboard data refreshed",
            "data": data
        })).into_response(),
        Err(e) => {
            eprintln!("Failed to refresh dashboard data: {}", e);
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({
                    "error": "Service temporarily unavailable",
                    "message": format!("Unable to refresh dashboard data: {}", e)
                }))
            ).into_response()
        }
    }
}

// API endpoint to get rate limiting status for monitoring
pub async fn api_rate_limit_status(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let rate_limit_status = state.data_service.get_rate_limit_status();
    
    Json(json!({
        "rate_limit_status": rate_limit_status,
        "timestamp": chrono::Utc::now(),
        "server_info": {
            "total_requests": state.request_counter.load(Ordering::Relaxed),
            "uptime_seconds": state.start_time.elapsed().as_secs()
        }
    })).into_response()
}
