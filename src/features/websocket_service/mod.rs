// src/features/websocket_service/mod.rs - WebSocket Service Feature
//
// Real-time communication layer with broadcast capabilities,
// connection management, and heartbeat functionality

pub mod connection_manager;
pub mod broadcast_service;
pub mod message_handler;
pub mod heartbeat;

pub use connection_manager::ConnectionManager;
pub use broadcast_service::BroadcastService;
pub use message_handler::MessageHandler;

use crate::features::Feature;
use crate::features::external_apis::MarketDataProvider;
use crate::features::cache_system::CacheManager;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

/// WebSocket Service Feature - Layer 3 (Communication)
pub struct WebSocketServiceFeature {
    connection_manager: Option<Arc<ConnectionManager>>,
    broadcast_service: Option<Arc<BroadcastService>>,
}

impl WebSocketServiceFeature {
    pub fn new() -> Self {
        Self {
            connection_manager: None,
            broadcast_service: None,
        }
    }

    pub fn connection_manager(&self) -> Option<&Arc<ConnectionManager>> {
        self.connection_manager.as_ref()
    }

    pub fn broadcast_service(&self) -> Option<&Arc<BroadcastService>> {
        self.broadcast_service.as_ref()
    }
}

#[async_trait]
impl Feature for WebSocketServiceFeature {
    fn name(&self) -> &'static str {
        "websocket_service"
    }

    fn version(&self) -> &'static str {
        "1.0.0"
    }

    fn dependencies(&self) -> Vec<&'static str> {
        vec!["shared_components", "cache_system", "external_apis"] // Layer 3 depends on Layers 1-2
    }

    async fn initialize(&mut self, context: &HashMap<String, String>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Initialize broadcast service first
        let broadcast_service = Arc::new(BroadcastService::new(100).await?); // 100 message buffer
        
        // Initialize connection manager with broadcast service
        let connection_manager = Arc::new(
            ConnectionManager::new(broadcast_service.clone()).await?
        );

        self.broadcast_service = Some(broadcast_service);
        self.connection_manager = Some(connection_manager);

        println!("✅ WebSocket Service Feature initialized");
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(_manager) = self.connection_manager.take() {
            println!("🔽 WebSocket connections shutting down");
        }
        
        if let Some(_broadcast) = self.broadcast_service.take() {
            println!("🔽 WebSocket broadcast service shutting down");
        }
        
        Ok(())
    }

    fn is_healthy(&self) -> bool {
        self.connection_manager.is_some() && self.broadcast_service.is_some()
    }
}
