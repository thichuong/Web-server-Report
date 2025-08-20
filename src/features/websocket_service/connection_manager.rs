// src/features/websocket_service/connection_manager.rs
//
// WebSocket connection management with heartbeat and graceful handling

use anyhow::Result;
use axum::extract::ws::{WebSocket, Message};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::broadcast;

use super::broadcast_service::BroadcastService;
use super::message_handler::MessageHandler;
use super::heartbeat::HeartbeatManager;
use crate::features::external_apis::DashboardSummary;
use crate::features::cache_system::CacheManager;

/// WebSocket connection manager
#[derive(Debug)]
pub struct ConnectionManager {
    broadcast_service: Arc<BroadcastService>,
    message_handler: Arc<MessageHandler>,
    heartbeat_manager: Arc<HeartbeatManager>,
}

impl ConnectionManager {
    pub async fn new(broadcast_service: Arc<BroadcastService>) -> Result<Self> {
        let message_handler = Arc::new(MessageHandler::new());
        let heartbeat_manager = Arc::new(HeartbeatManager::new(30)); // 30 second heartbeat
        
        Ok(Self {
            broadcast_service,
            message_handler,
            heartbeat_manager,
        })
    }

    /// Handle a new WebSocket connection
    pub async fn handle_websocket(
        &self, 
        mut socket: WebSocket, 
        cache_manager: Option<Arc<CacheManager>>
    ) {
        println!("🔗 New WebSocket connection established");

        // Send current data immediately if available
        if let Some(cache_manager) = &cache_manager {
            if let Ok(Some(current_data)) = self.get_cached_dashboard_data(cache_manager).await {
                let welcome_message = json!({
                    "type": "dashboard_update",
                    "data": current_data,
                    "source": "connection_welcome"
                }).to_string();

                if let Err(e) = socket.send(Message::Text(welcome_message)).await {
                    println!("❌ Failed to send welcome message: {}", e);
                    return;
                }
                println!("👋 Sent welcome message with current data");
            }
        }

        // Subscribe to broadcast updates
        let mut broadcast_rx = self.broadcast_service.subscribe();

        // Start heartbeat for this connection
        let heartbeat_handle = self.heartbeat_manager.start_heartbeat();

        loop {
            tokio::select! {
                // Handle incoming WebSocket messages
                msg = socket.recv() => {
                    match msg {
                        Some(Ok(Message::Text(text))) => {
                            if let Err(e) = self.handle_incoming_message(&text, &mut socket).await {
                                println!("❌ Error handling message: {}", e);
                                break;
                            }
                        }
                        Some(Ok(Message::Binary(_))) => {
                            // Ignore binary messages for now
                            println!("ℹ️ Received binary message (ignored)");
                        }
                        Some(Ok(Message::Close(_))) => {
                            println!("🔌 WebSocket connection closed by client");
                            break;
                        }
                        Some(Ok(Message::Ping(data))) => {
                            if socket.send(Message::Pong(data)).await.is_err() {
                                break;
                            }
                        }
                        Some(Ok(Message::Pong(_))) => {
                            // Client responded to our ping
                            self.heartbeat_manager.reset_heartbeat();
                        }
                        Some(Err(e)) => {
                            eprintln!("❌ WebSocket error: {}", e);
                            break;
                        }
                        None => {
                            println!("🔌 WebSocket connection terminated");
                            break;
                        }
                    }
                }
                
                // Handle broadcast messages
                broadcast_msg = broadcast_rx.recv() => {
                    match broadcast_msg {
                        Ok(message) => {
                            if let Err(e) = socket.send(Message::Text(message)).await {
                                println!("❌ Failed to send broadcast message: {}", e);
                                break;
                            }
                        }
                        Err(broadcast::error::RecvError::Closed) => {
                            println!("📡 Broadcast channel closed");
                            break;
                        }
                        Err(broadcast::error::RecvError::Lagged(count)) => {
                            println!("⚠️ WebSocket client lagged behind {} messages, continuing...", count);
                            continue;
                        }
                    }
                }
                
                // Handle heartbeat timeout
                _ = heartbeat_handle.timeout() => {
                    println!("💔 WebSocket connection timed out (no heartbeat)");
                    let _ = socket.send(Message::Close(None)).await;
                    break;
                }
            }
        }

        println!("🔌 WebSocket connection terminated");
    }

    /// Handle incoming message from client
    async fn handle_incoming_message(&self, text: &str, socket: &mut WebSocket) -> Result<()> {
        match self.message_handler.handle_message(text).await {
            Ok(Some(response)) => {
                if let Err(e) = socket.send(Message::Text(response)).await {
                    println!("❌ Failed to send response: {}", e);
                }
            }
            Ok(None) => {
                // Message handled but no response needed
            }
            Err(e) => {
                println!("❌ Error processing message '{}': {}", text, e);
                
                // Send error response to client
                let error_response = json!({
                    "type": "error",
                    "message": "Failed to process message",
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }).to_string();
                
                let _ = socket.send(Message::Text(error_response)).await;
            }
        }
        
        Ok(())
    }

    /// Get cached dashboard data if available
    async fn get_cached_dashboard_data(&self, cache_manager: &CacheManager) -> Result<Option<DashboardSummary>> {
        // Try to get cached dashboard data
        let cache_key = "dashboard:summary:rapid";
        
        match cache_manager.get::<DashboardSummary>(cache_key).await {
            Ok(Some(data)) => {
                println!("✅ Retrieved cached dashboard data for welcome message");
                Ok(Some(data))
            },
            Ok(None) => {
                println!("ℹ️ No cached dashboard data found for welcome message");
                Ok(None)
            },
            Err(e) => {
                println!("⚠️ Cache error retrieving dashboard data: {}", e);
                Ok(None) // Return None for graceful fallback
            }
        }
    }

    /// Get connection manager statistics
    pub fn get_stats(&self) -> ConnectionStats {
        let broadcast_stats = self.broadcast_service.get_stats();
        
        ConnectionStats {
            active_connections: broadcast_stats.receiver_count,
            broadcast_buffer_capacity: broadcast_stats.channel_capacity,
            heartbeat_interval_seconds: self.heartbeat_manager.get_interval_seconds(),
        }
    }
}

/// Connection manager statistics
#[derive(Debug)]
pub struct ConnectionStats {
    pub active_connections: usize,
    pub broadcast_buffer_capacity: usize,
    pub heartbeat_interval_seconds: u64,
}
