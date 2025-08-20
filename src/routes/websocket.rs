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
async fn websocket_connection_handler(
    mut socket: axum::extract::ws::WebSocket,
    service_islands: Arc<ServiceIslands>
) {
    println!("✅ WebSocket connection established successfully");
    
    // Send initial welcome message with current data
    let welcome_msg = json!({
        "type": "connected",
        "message": "WebSocket connected to Service Islands",
        "timestamp": chrono::Utc::now().to_rfc3339()
    });
    
    if let Err(_) = socket.send(axum::extract::ws::Message::Text(welcome_msg.to_string())).await {
        println!("❌ Failed to send welcome message");
        return;
    }
    
    // Send initial dashboard data
    if let Ok(dashboard_data) = service_islands.crypto_reports.fetch_realtime_market_data().await {
        // Extract actual values from the JSON
        let btc_price = dashboard_data.get("btc_price_usd").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let btc_change = dashboard_data.get("btc_change_24h").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let market_cap = dashboard_data.get("market_cap_usd").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let volume = dashboard_data.get("volume_24h_usd").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let fng = dashboard_data.get("fng_value").and_then(|v| v.as_u64()).unwrap_or(50) as u32;
        let rsi = dashboard_data.get("rsi_14").and_then(|v| v.as_f64()).unwrap_or(50.0);
        
        let dashboard_msg = json!({
            "type": "dashboard_data",
            "data": {
                "btc_price_usd": btc_price,
                "btc_change_24h": btc_change,
                "market_cap_usd": market_cap,
                "volume_24h_usd": volume,
                "fng_value": fng,
                "rsi_14": rsi
            },
            "timestamp": chrono::Utc::now().to_rfc3339()
        });
        
        // 🔍 DEBUG: Log extracted values
        println!("🔍 [WebSocket] Extracted values:");
        println!("  💰 Market Cap: ${}", market_cap);
        println!("  📊 Volume: ${}", volume);
        println!("  ₿ BTC Price: ${}", btc_price);
        println!("  📈 BTC Change: {}%", btc_change);
        println!("  😨 F&G: {}", fng);
        println!("  📈 RSI: {}", rsi);
        
        if let Ok(_) = socket.send(axum::extract::ws::Message::Text(dashboard_msg.to_string())).await {
            println!("📊 Initial dashboard data sent to WebSocket client");
        }
    }
    
    // Handle incoming messages from client 
    loop {
        match socket.recv().await {
            Some(Ok(axum::extract::ws::Message::Text(text))) => {
                println!("📨 Received from client: {}", text);
                
                if text == "ping" {
                    let pong_response = json!({
                        "type": "pong",
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    });
                    
                    println!("🏓 Sending pong: {}", pong_response);
                    if let Err(_) = socket.send(axum::extract::ws::Message::Text(pong_response.to_string())).await {
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
    
    println!("🔌 WebSocket connection handler finished");
}
