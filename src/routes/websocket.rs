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
use std::sync::atomic::{AtomicUsize, Ordering};
use serde_json::json;
use tokio::time::Duration;

use crate::service_islands::ServiceIslands;
use crate::service_islands::layer5_business_logic::market_data_service::fetch_realtime_market_data;

/// Connection guard for automatic cleanup tracking
/// Implements Drop trait to ensure cleanup happens even if task panics
struct ConnectionGuard {
    id: usize,
    active_connections: Arc<AtomicUsize>,
}

impl ConnectionGuard {
    fn new(active_connections: Arc<AtomicUsize>) -> Self {
        let count = active_connections.fetch_add(1, Ordering::SeqCst) + 1;
        let id = count;
        println!("✅ WebSocket connection #{} established. Total active: {}", id, count);
        Self { id, active_connections }
    }
}

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        let remaining = self.active_connections.fetch_sub(1, Ordering::SeqCst) - 1;
        println!("🧹 Cleaning up WebSocket connection #{}. Remaining active: {}", 
                 self.id, remaining);
    }
}

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
    
    // Add Cloudflare compatibility logging
    ws.protocols(["chat", "superchat"])  // Optional: specify supported protocols
        .on_upgrade(move |socket| websocket_connection_handler(socket, service_islands))
}

/// Handle individual WebSocket connections with Service Islands integration
/// 
/// ✅ CORRECTED: Uses proper Layer 3 (Communication) → Layer 2 (External APIs) workflow
/// ✅ MEMORY SAFE: Implements explicit cleanup and connection tracking
async fn websocket_connection_handler(
    mut socket: axum::extract::ws::WebSocket,
    service_islands: Arc<ServiceIslands>
) {
    // Create connection guard - ensures cleanup on drop (even if panic)
    let _guard = ConnectionGuard::new(service_islands.active_ws_connections.clone());
    
    println!("✅ WebSocket connection established - connecting to Layer 3 broadcast system");
    
    // ✅ Get Layer 3 broadcast receiver for real-time updates
    let mut broadcast_rx = service_islands.websocket_service.get_broadcast_tx().subscribe();
    
    // Track last activity for timeout detection
    let mut last_activity = tokio::time::Instant::now();
    let connection_timeout = Duration::from_secs(300); // 5 minutes idle timeout
    
    // Send initial welcome message - serialize 1 lần
    let welcome_msg = json!({
        "type": "connected",
        "message": "WebSocket connected to Service Islands Layer 3",
        "timestamp": chrono::Utc::now().to_rfc3339()
    });
    let welcome_msg_str = welcome_msg.to_string(); // Serialize 1 lần
    
    if let Err(_) = socket.send(axum::extract::ws::Message::Text(welcome_msg_str)).await {
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
        let initial_msg_str = initial_msg.to_string(); // Serialize 1 lần
        
        if let Ok(_) = socket.send(axum::extract::ws::Message::Text(initial_msg_str)).await {
            println!("📊 Initial dashboard data sent via Layer 5 Market Data Service (shared architecture)");
        }
    }
    
    // ✅ Handle both broadcast messages AND client messages in single loop
    // This avoids mutex contention completely - no shared socket lock needed!
    println!("🔄 Starting unified WebSocket message handler with timeout protection");
    
    loop {
        // Check for connection timeout
        if last_activity.elapsed() > connection_timeout {
            println!("⏰ Connection timeout after {:?} of inactivity - closing connection", connection_timeout);
            break;
        }
        
        tokio::select! {
            // Listen for broadcast messages from Layer 3
            broadcast_result = broadcast_rx.recv() => {
                match broadcast_result {
                    Ok(broadcast_message) => {
                        last_activity = tokio::time::Instant::now(); // Reset timeout on activity
                        println!("📡 Received broadcast message, sending to client...");
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
                        last_activity = tokio::time::Instant::now(); // Reset timeout on activity
                        println!("📨 Received from client: {}", text);
                        
                        // Handle ping/pong heartbeat
                        if text == "ping" {
                            let pong_response = json!({
                                "type": "pong",
                                "timestamp": chrono::Utc::now().to_rfc3339()
                            });
                            let pong_str = pong_response.to_string(); // Serialize 1 lần
                            
                            println!("🏓 Sending pong");
                            if let Err(_) = socket.send(axum::extract::ws::Message::Text(pong_str)).await {
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
                                let refresh_str = refresh_msg.to_string(); // Serialize 1 lần
                                
                                if let Ok(_) = socket.send(axum::extract::ws::Message::Text(refresh_str)).await {
                                    println!("✅ Fresh dashboard data sent in response to client request");
                                }
                            } else {
                                println!("⚠️ Failed to fetch fresh data for client request");
                            }
                        }
                    }
                    Some(Ok(axum::extract::ws::Message::Close(frame))) => {
                        println!("🔌 WebSocket client sent close frame: {:?}", frame);
                        break;
                    }
                    Some(Err(e)) => {
                        println!("❌ WebSocket error: {}", e);
                        break;
                    }
                    None => {
                        println!("🔌 WebSocket stream ended (client disconnected)");
                        break;
                    }
                    _ => {
                        // Ignore other message types (binary, pong, etc.)
                        last_activity = tokio::time::Instant::now(); // Still count as activity
                    }
                }
            }
            
            // Periodic timeout check (every 30 seconds)
            _ = tokio::time::sleep(Duration::from_secs(30)) => {
                // Just continue - timeout will be checked at loop start
                continue;
            }
        }
    }
    
    // Explicit cleanup before function exits
    println!("🧹 Starting WebSocket connection cleanup...");
    
    // 1. Try to close socket gracefully
    if let Err(e) = socket.close().await {
        println!("⚠️ Error during socket close (may already be closed): {}", e);
    } else {
        println!("✅ Socket closed gracefully");
    }
    
    // 2. broadcast_rx will be automatically dropped here
    println!("✅ Broadcast receiver dropped");
    
    // 3. ConnectionGuard will be dropped here, decrementing counter
    println!("✅ WebSocket connection fully cleaned up");
}
