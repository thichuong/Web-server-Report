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
use crate::service_islands::layer5_business_logic::market_data_service::fetch_realtime_market_data;

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
    
    // ✅ Send initial dashboard data via Layer 5 Market Data Service (improved architecture)
    if let Ok(dashboard_data) = fetch_realtime_market_data(&service_islands.websocket_service).await {
        // ✅ Use compatible message format that JavaScript client expects
        let initial_msg = json!({
            "type": "dashboard_data",  // Compatible with JavaScript client
            "data": dashboard_data,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "source": "layer5_market_data_service"  // Updated to reflect new service
        });
        
        if let Ok(_) = socket.send(axum::extract::ws::Message::Text(initial_msg.to_string())).await {
            println!("📊 Initial dashboard data sent via Layer 5 Market Data Service (shared architecture)");
        }
    }
    
    // ✅ Handle both broadcast messages AND client messages in single loop
    // This avoids mutex contention completely - no shared socket lock needed!
    println!("🔄 Starting unified WebSocket message handler");
    
    loop {
        tokio::select! {
            // Listen for broadcast messages from Layer 3
            broadcast_result = broadcast_rx.recv() => {
                match broadcast_result {
                    Ok(broadcast_message) => {
                        println!("� Received broadcast message, sending to client...");
                        if let Err(e) = socket.send(axum::extract::ws::Message::Text(broadcast_message)).await {
                            println!("❌ Failed to send broadcast message: {}", e);
                            break;
                        }
                        println!("✅ Broadcast message sent to client successfully");
                    }
                    Err(e) => {
                        println!("⚠️ Broadcast channel closed: {}", e);
                        break;
                    }
                }
            }
            
            // Listen for messages from client
            client_message = socket.recv() => {
                match client_message {
                    Some(Ok(axum::extract::ws::Message::Text(text))) => {
                        println!("📨 Received from client: {}", text);
                        
                        // Handle ping/pong heartbeat
                        if text == "ping" {
                            let pong_response = json!({
                                "type": "pong",
                                "timestamp": chrono::Utc::now().to_rfc3339()
                            });
                            
                            println!("🏓 Sending pong");
                            if let Err(_) = socket.send(axum::extract::ws::Message::Text(pong_response.to_string())).await {
                                break;
                            }
                        }
                        // Handle data refresh requests
                        else if text == "request_update" || text.contains("request_dashboard_data") {
                            println!("📊 Client requested fresh dashboard data");
                            
                            // Fetch fresh data via Layer 5 Market Data Service
                            if let Ok(dashboard_data) = fetch_realtime_market_data(&service_islands.websocket_service).await {
                                let refresh_msg = json!({
                                    "type": "dashboard_data",
                                    "data": dashboard_data,
                                    "timestamp": chrono::Utc::now().to_rfc3339(),
                                    "source": "client_request"
                                });
                                
                                if let Ok(_) = socket.send(axum::extract::ws::Message::Text(refresh_msg.to_string())).await {
                                    println!("✅ Fresh dashboard data sent in response to client request");
                                }
                            } else {
                                println!("⚠️ Failed to fetch fresh data for client request");
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
        }
    }
    
    println!("🔌 WebSocket connection handler finished");
}
