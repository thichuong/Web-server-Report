//! WebSocket-related response DTOs

use serde::Serialize;

/// Response for GET /api/websocket/stats endpoint
/// Indicates that WebSocket functionality is in a separate service
#[derive(Debug, Serialize)]
pub struct WebSocketStatsResponse {
    pub message: String,
    pub websocket_service: String,
    pub websocket_health_endpoint: String,
    pub timestamp: String,
}
