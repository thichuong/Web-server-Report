// src/features/websocket_service/message_handler.rs
//
// WebSocket message processing and response generation

use anyhow::{Result, anyhow};
use serde_json::{json, Value};

/// Message handler for WebSocket communications
#[derive(Debug)]
pub struct MessageHandler {
    // Future: Could add command handlers, authentication, etc.
}

impl MessageHandler {
    pub fn new() -> Self {
        Self {}
    }

    /// Handle incoming message from WebSocket client
    pub async fn handle_message(&self, message: &str) -> Result<Option<String>> {
        // Handle simple ping/pong
        if message == "ping" {
            return Ok(Some("pong".to_string()));
        }

        // Try to parse as JSON for structured commands
        if let Ok(json_msg) = serde_json::from_str::<Value>(message) {
            return self.handle_json_message(json_msg).await;
        }

        // Handle plain text commands
        self.handle_text_command(message).await
    }

    /// Handle structured JSON messages
    async fn handle_json_message(&self, message: Value) -> Result<Option<String>> {
        let msg_type = message.get("type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Message must have a 'type' field"))?;

        match msg_type {
            "ping" => {
                Ok(Some(json!({
                    "type": "pong",
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }).to_string()))
            }
            "subscribe" => {
                self.handle_subscription(&message).await
            }
            "unsubscribe" => {
                self.handle_unsubscription(&message).await
            }
            "request_data" => {
                self.handle_data_request(&message).await
            }
            _ => {
                println!("⚠️ Unknown message type: {}", msg_type);
                Ok(Some(json!({
                    "type": "error",
                    "error": "unknown_message_type",
                    "message": format!("Unknown message type: {}", msg_type),
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }).to_string()))
            }
        }
    }

    /// Handle plain text commands
    async fn handle_text_command(&self, command: &str) -> Result<Option<String>> {
        match command.to_lowercase().as_str() {
            "status" => {
                Ok(Some(json!({
                    "type": "status_response",
                    "status": "healthy",
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "uptime_seconds": 0 // Would be calculated from actual uptime
                }).to_string()))
            }
            "help" => {
                Ok(Some(json!({
                    "type": "help_response",
                    "available_commands": [
                        "ping - Test connection",
                        "status - Get service status", 
                        "help - Show this help"
                    ],
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }).to_string()))
            }
            _ => {
                println!("ℹ️ Received unhandled text message: {}", command);
                Ok(None) // Don't respond to unknown commands
            }
        }
    }

    /// Handle subscription requests
    async fn handle_subscription(&self, message: &Value) -> Result<Option<String>> {
        let channel = message.get("channel")
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        println!("📡 Client subscribing to channel: {}", channel);

        Ok(Some(json!({
            "type": "subscription_confirmed",
            "channel": channel,
            "message": format!("Subscribed to channel: {}", channel),
            "timestamp": chrono::Utc::now().to_rfc3339()
        }).to_string()))
    }

    /// Handle unsubscription requests  
    async fn handle_unsubscription(&self, message: &Value) -> Result<Option<String>> {
        let channel = message.get("channel")
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        println!("📡 Client unsubscribing from channel: {}", channel);

        Ok(Some(json!({
            "type": "unsubscription_confirmed", 
            "channel": channel,
            "message": format!("Unsubscribed from channel: {}", channel),
            "timestamp": chrono::Utc::now().to_rfc3339()
        }).to_string()))
    }

    /// Handle data requests
    async fn handle_data_request(&self, message: &Value) -> Result<Option<String>> {
        let data_type = message.get("data_type")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        match data_type {
            "dashboard" => {
                Ok(Some(json!({
                    "type": "data_response",
                    "data_type": "dashboard",
                    "message": "Dashboard data will be sent via next broadcast update",
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }).to_string()))
            }
            "status" => {
                Ok(Some(json!({
                    "type": "data_response",
                    "data_type": "status",
                    "data": {
                        "service_status": "healthy",
                        "connections": 0, // Would be actual count
                        "uptime_seconds": 0 // Would be actual uptime
                    },
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }).to_string()))
            }
            _ => {
                Ok(Some(json!({
                    "type": "error",
                    "error": "unsupported_data_type",
                    "message": format!("Unsupported data type: {}", data_type),
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }).to_string()))
            }
        }
    }
}
