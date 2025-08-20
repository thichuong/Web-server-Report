use axum::{
    extract::{State, ws::WebSocketUpgrade},
    response::IntoResponse,
};
use std::sync::Arc;

use crate::state::AppState;

// WebSocket handler for real-time updates
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| async move {
        state.websocket_service.handle_websocket(socket).await;
    })
}
