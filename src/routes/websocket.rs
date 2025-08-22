//! WebSocket Routes
//! 
//! This module handles WebSocket connections and real-time communication
//! through the Service Islands Architecture.

use axum::{
    routing::get,
    Router,
    extract::{WebSocketUpgrade, State},
    response::Response
};
use std::sync::Arc;
use serde_json::json;

use crate::service_islands::ServiceIslands;

/// Configure WebSocket routes
pub fn configure_websocket_routes() -> Router<Arc<ServiceIslands>> {
    Router::new()
        .route("/ws", get(websocket_handler))
}

/// WebSocket connection handler - Real WebSocket upgrade for Service Islands
async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(service_islands): State<Arc<ServiceIslands>>
) -> Response {
    println!("🔌 WebSocket upgrade request received - establishing real-time connection");
    
    ws.on_upgrade(move |socket| websocket_connection_handler(socket, service_islands))
}

/// Handle individual WebSocket connections with Service Islands integration
/// 
/// ✅ CORRECTED: Uses proper Layer 3 (Communication) → Layer 2 (External APIs) workflow
async fn websocket_connection_handler(
    mut socket: axum::extract::ws::WebSocket,
    service_islands: Arc<ServiceIslands>
) {
    println!("✅ WebSocket connection established - connecting to Layer 3 broadcast system");
    
    // ✅ Get Layer 3 broadcast receiver for real-time updates
    let mut broadcast_rx = service_islands.websocket_service.get_broadcast_tx().subscribe();
    
    // Send initial welcome message
    let welcome_msg = json!({
        "type": "connected",
        "message": "WebSocket connected to Service Islands Layer 3",
        "timestamp": chrono::Utc::now().to_rfc3339()
    });
    
    if let Err(_) = socket.send(axum::extract::ws::Message::Text(welcome_msg.to_string())).await {
        println!("❌ Failed to send welcome message");
        return;
    }
    
    // ✅ Send initial dashboard data via proper architecture: Layer 5 → Layer 3 → Layer 2
    if let Ok(dashboard_data) = service_islands.crypto_reports.fetch_realtime_market_data().await {
        // ✅ Use compatible message format that JavaScript client expects
        let initial_msg = json!({
            "type": "dashboard_data",  // Compatible with JavaScript client
            "data": dashboard_data,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "source": "layer5_via_layer3"  // Updated to reflect proper architecture
        });
        
        if let Ok(_) = socket.send(axum::extract::ws::Message::Text(initial_msg.to_string())).await {
            println!("📊 Initial dashboard data sent via Layer 5 → Layer 3 → Layer 2 (proper architecture)");
        }
    }
    
    // ✅ Start Layer 3 real-time broadcast listener in background
    let socket_clone = Arc::new(tokio::sync::Mutex::new(socket));
    let socket_for_broadcast = socket_clone.clone();
    
    // Background task: Listen to Layer 3 broadcast updates
    let broadcast_task = tokio::spawn(async move {
        println!("🔄 Starting Layer 3 broadcast listener for WebSocket client");
        
        while let Ok(broadcast_message) = broadcast_rx.recv().await {
            let mut socket_guard = socket_for_broadcast.lock().await;
            
            match socket_guard.send(axum::extract::ws::Message::Text(broadcast_message)).await {
                Ok(_) => {
                    // Success - continue listening
                }
                Err(_) => {
                    println!("❌ WebSocket client disconnected - stopping broadcast listener");
                    break;
                }
            }
        }
    });
    
    // Handle client messages (ping/pong, etc.)
    let socket_for_client = socket_clone.clone();
    loop {
        let message_result = {
            let mut socket_guard = socket_for_client.lock().await;
            socket_guard.recv().await
        };
        
        match message_result {
            Some(Ok(axum::extract::ws::Message::Text(text))) => {
                println!("📨 Received from client: {}", text);
                
                if text == "ping" {
                    let pong_response = json!({
                        "type": "pong",
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    });
                    
                    println!("🏓 Sending pong: {}", pong_response);
                    let mut socket_guard = socket_for_client.lock().await;
                    if let Err(_) = socket_guard.send(axum::extract::ws::Message::Text(pong_response.to_string())).await {
                        break;
                    }
                }
            }
            Some(Ok(axum::extract::ws::Message::Close(_))) => {
                println!("🔌 WebSocket client disconnected");
                break;
            }
            Some(Err(e)) => {
                println!("❌ WebSocket error: {}", e);
                break;
            }
            None => {
                println!("🔌 WebSocket stream ended");
                break;
            }
            _ => {
                // Ignore other message types (binary, pong, etc.)
            }
        }
    }
    
    // Cleanup: Cancel broadcast task when client disconnects
    broadcast_task.abort();
    println!("🔌 WebSocket connection handler finished - Layer 3 broadcast listener stopped");
}
