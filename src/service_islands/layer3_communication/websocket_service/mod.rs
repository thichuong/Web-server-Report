//! WebSocket Service Island - Layer 3 Communication
//! 
//! This island handles all WebSocket-related functionality including:
//! - Connection management
//! - Message handling and broadcasting  
//! - Real-time data updates from Layer 2 External APIs
//! - Client communication protocols

pub mod connection_manager;
pub mod message_handler;
pub mod broadcast_service;
pub mod handlers;
pub mod market_data_streamer;

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::broadcast;

use connection_manager::ConnectionManager;
use message_handler::MessageHandler;
use broadcast_service::BroadcastService;
use handlers::WebSocketHandlers;
use market_data_streamer::MarketDataStreamer;
use crate::service_islands::layer2_external_services::external_apis_island::ExternalApisIsland;

/// WebSocket Service Island
/// 
/// Central coordinator for all WebSocket communication functionality.
/// Manages real-time connections, message broadcasting, and data synchronization.
/// Integrates with Layer 2 External APIs following Service Islands Architecture.
pub struct WebSocketServiceIsland {
    /// Connection management component
    pub connection_manager: Arc<ConnectionManager>,
    /// Message processing component
    pub message_handler: Arc<MessageHandler>,
    /// Broadcast service component
    pub broadcast_service: Arc<BroadcastService>,
    /// HTTP handlers component
    pub handlers: Arc<WebSocketHandlers>,
    /// Market data streaming component
    pub market_data_streamer: Arc<MarketDataStreamer>,
    /// Broadcast transmitter for real-time updates
    pub broadcast_tx: broadcast::Sender<String>,
}

impl WebSocketServiceIsland {
    /// Initialize the WebSocket Service Island without External APIs dependency
    /// 
    /// Creates all components and establishes communication channels.
    pub async fn new() -> Result<Self> {
        println!("🏝️ Initializing WebSocket Service Island (Layer 3 Communication)...");
        
        // Create broadcast channel for real-time updates
        let (broadcast_tx, _) = broadcast::channel(1000);
        
        // Initialize components
        let connection_manager = Arc::new(ConnectionManager::new());
        let message_handler = Arc::new(MessageHandler::new());
        let broadcast_service = Arc::new(BroadcastService::new());
        let handlers = Arc::new(WebSocketHandlers::new());
        let market_data_streamer = Arc::new(MarketDataStreamer::new());
        
        // Start background services
        broadcast_service.start_background_updates().await;
        
        println!("✅ WebSocket Service Island initialized successfully");
        
        Ok(Self {
            connection_manager,
            message_handler,
            broadcast_service,
            handlers,
            market_data_streamer,
            broadcast_tx,
        })
    }
    
    /// Initialize WebSocket Service Island with Layer 2 External APIs dependency
    /// 
    /// This follows Service Islands Architecture dependency injection pattern:
    /// Layer 3 (Communication) depends on Layer 2 (External Services)
    pub async fn with_external_apis(external_apis: Arc<ExternalApisIsland>) -> Result<Self> {
        println!("🏝️ Initializing WebSocket Service Island with Layer 2 External APIs integration...");
        
        // Create broadcast channel for real-time updates
        let (broadcast_tx, _) = broadcast::channel(1000);
        
        // Initialize components with External APIs dependency
        let connection_manager = Arc::new(ConnectionManager::new());
        let message_handler = Arc::new(MessageHandler::new());
        let broadcast_service = Arc::new(BroadcastService::new());
        let handlers = Arc::new(WebSocketHandlers::new());
        let market_data_streamer = Arc::new(MarketDataStreamer::with_external_apis(external_apis));
        
        // Start background services
        broadcast_service.start_background_updates().await;
        
        // Start real-time market data streaming
        market_data_streamer.start_streaming(broadcast_tx.clone()).await?;
        market_data_streamer.start_btc_streaming(broadcast_tx.clone()).await?;
        
        println!("✅ WebSocket Service Island with External APIs integration initialized successfully");
        
        Ok(Self {
            connection_manager,
            message_handler,
            broadcast_service,
            handlers,
            market_data_streamer,
            broadcast_tx,
        })
    }
    
    /// Health check for the entire WebSocket Service Island
    /// 
    /// Validates that all components are operational.
    pub async fn health_check(&self) -> Result<()> {
        println!("🏥 Checking WebSocket Service Island health...");
        
        // Check all components
        let checks = vec![
            ("Connection Manager", self.connection_manager.health_check().await),
            ("Message Handler", self.message_handler.health_check().await),
            ("Broadcast Service", self.broadcast_service.health_check().await),
            ("WebSocket Handlers", self.handlers.health_check().await),
            ("Market Data Streamer", self.market_data_streamer.health_check().await),
        ];
        
        let mut all_healthy = true;
        for (component, healthy) in checks {
            if healthy {
                println!("  ✅ {} - Healthy", component);
            } else {
                println!("  ❌ {} - Unhealthy", component);
                all_healthy = false;
            }
        }
        
        if all_healthy {
            println!("✅ WebSocket Service Island - All components healthy");
            Ok(())
        } else {
            Err(anyhow::anyhow!("WebSocket Service Island - Some components unhealthy"))
        }
    }
    
    /// Get broadcast transmitter
    /// 
    /// Returns the broadcast transmitter for sending real-time updates.
    pub fn get_broadcast_tx(&self) -> broadcast::Sender<String> {
        self.broadcast_tx.clone()
    }
    
    /// Get connection statistics
    /// 
    /// Returns current WebSocket connection statistics.
    pub async fn get_connection_stats(&self) -> Result<serde_json::Value> {
        // This will be implemented when we have actual connection tracking
        Ok(serde_json::json!({
            "active_connections": 0,
            "total_messages_sent": 0,
            "total_messages_received": 0,
            "uptime": "0s"
        }))
    }
    
    /// Broadcast message to all connected clients
    /// 
    /// Sends a message to all active WebSocket connections.
    pub async fn broadcast(&self, message: String) -> Result<()> {
        self.broadcast_service.broadcast_message(message, &self.broadcast_tx).await
    }
    
    /// Broadcast dashboard update
    /// 
    /// Broadcasts a dashboard data update to all connected clients.
    pub async fn broadcast_dashboard_update(&self, dashboard_data: serde_json::Value) -> Result<()> {
        self.broadcast_service.broadcast_dashboard_update(dashboard_data, &self.broadcast_tx).await
    }
}
