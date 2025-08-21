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
use crate::service_islands::layer3_communication::layer2_adapters::Layer2AdaptersHub;

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
    /// Layer 2 adapters hub for clean API access
    pub layer2_adapters: Arc<Layer2AdaptersHub>,
    /// Broadcast transmitter for real-time updates
    pub broadcast_tx: broadcast::Sender<String>,
}

impl WebSocketServiceIsland {
    /// Initialize the WebSocket Service Island without External APIs dependency
    /// 
    /// Creates all components and establishes communication channels.
    pub async fn new() -> Result<Self> {
        println!("🔧 Initializing WebSocket Service Island (standalone mode)...");
        
        // Initialize Layer 2 adapters without External APIs dependency
        let layer2_adapters = Arc::new(Layer2AdaptersHub::new());
        
        // Initialize components
        let connection_manager = Arc::new(ConnectionManager::new());
        let message_handler = Arc::new(MessageHandler::new());
        let broadcast_service = Arc::new(BroadcastService::new());
        let handlers = Arc::new(WebSocketHandlers::new());
        
        // Initialize market data streamer without External APIs (standalone mode)
        let market_data_streamer = Arc::new(MarketDataStreamer::new());
        
        // Create broadcast channel (increased buffer for high-frequency updates) 
        let (broadcast_tx, _) = broadcast::channel(1000);
        
        println!("✅ WebSocket Service Island initialized successfully (standalone mode)");
        
        Ok(Self {
            connection_manager,
            message_handler,
            broadcast_service,
            handlers,
            market_data_streamer,
            layer2_adapters,
            broadcast_tx,
        })
    }
    
    /// Initialize the WebSocket Service Island with External APIs dependency
    /// 
    /// Creates all components and establishes communication channels with Layer 2.
    pub async fn with_external_apis(external_apis: Arc<ExternalApisIsland>) -> Result<Self> {
        println!("🔧 Initializing WebSocket Service Island with Layer 2 External APIs...");
        
        // Initialize Layer 2 adapters with External APIs dependency
        let layer2_adapters = Arc::new(
            Layer2AdaptersHub::new()
                .with_external_apis(external_apis.clone())
        );
        
        // Initialize components
        let connection_manager = Arc::new(ConnectionManager::new());
        let message_handler = Arc::new(MessageHandler::new());
        let broadcast_service = Arc::new(BroadcastService::new());
        let handlers = Arc::new(WebSocketHandlers::new());
        
        // Initialize market data streamer with External APIs
        let market_data_streamer = Arc::new(MarketDataStreamer::with_external_apis(external_apis.clone()));
        
        // Create broadcast channel (increased buffer for high-frequency updates)
        let (broadcast_tx, _) = broadcast::channel(1000);
        
        // Start market data streaming
        market_data_streamer.start_streaming(broadcast_tx.clone()).await?;
        market_data_streamer.start_btc_streaming(broadcast_tx.clone()).await?;
        
        println!("✅ WebSocket Service Island with External APIs and Layer 2 Adapters initialized successfully");
        
        Ok(Self {
            connection_manager,
            message_handler,
            broadcast_service,
            handlers,
            market_data_streamer,
            layer2_adapters,
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
            ("Layer 2 Adapters Hub", self.layer2_adapters.health_check().await.is_ok()),
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
    
    /// Fetch market data via Layer 3 → Layer 2 (proper architecture flow)
    /// 
    /// This method allows Layer 5 to request market data through Layer 3,
    /// maintaining proper Service Islands Architecture dependency flow:
    /// Layer 5 → Layer 3 → Layer 2
    pub async fn fetch_market_data_for_layer5(&self) -> Result<serde_json::Value> {
        println!("🔄 Layer 3 WebSocketService handling Layer 5 market data request...");
        
        // Use Layer 2 adapters for clean API access
        self.layer2_adapters.market_data.fetch_normalized_market_data().await
    }

    /// Start streaming with Service Islands access
    /// 
    /// This enables the market data streamer to use the same Layer 5 → Layer 3 → Layer 2 flow
    /// as HTTP API and WebSocket initial messages, ensuring unified data sources.
    pub async fn start_streaming_with_service_islands(&self, service_islands: Arc<crate::service_islands::ServiceIslands>) -> Result<()> {
        println!("🌊 Starting WebSocket streaming with unified Layer 5 access...");
        
        // Configure market data streamer with ServiceIslands access
        let updated_streamer = Arc::new(
            MarketDataStreamer::new()
                .with_service_islands(service_islands)
        );
        
        // Replace the existing streamer (this is a design pattern for runtime updates)
        // In a production system, you might want to handle this more gracefully
        updated_streamer.start_streaming(self.broadcast_tx.clone()).await?;
        
        Ok(())
    }

    /// Start Redis Streams consumer for real-time WebSocket broadcasting
    /// 
    /// Phase 3: This method creates a background consumer that listens to Redis Streams
    /// and broadcasts updates to all WebSocket clients in real-time.
    pub async fn start_stream_consumer(&self, cache_system: Arc<crate::service_islands::layer1_infrastructure::cache_system_island::CacheSystemIsland>) -> Result<()> {
        println!("🔄 Starting Redis Streams consumer for WebSocket broadcasting...");
        
        let broadcast_tx = self.broadcast_tx.clone();
        let stream_manager = cache_system.stream_manager.clone();
        
        // Spawn background task for stream consumption
        tokio::spawn(async move {
            println!("📡 Redis Streams → WebSocket consumer started");
            
            // Create consumer group for WebSocket broadcasting
            if let Err(e) = stream_manager.create_consumer_group("market_data", "websocket_broadcast").await {
                println!("⚠️ Consumer group creation failed (may already exist): {}", e);
            }
            
            loop {
                match stream_manager.consume("market_data", "websocket_broadcast", "ws_consumer", 5).await {
                    Ok(stream_events) => {
                        if stream_events.is_empty() {
                            // No new data - wait longer before next poll to avoid spam
                            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                            continue;
                        }
                        
                        println!("📡 Processing {} stream events for WebSocket broadcast", stream_events.len());
                        
                        for event in stream_events {
                            // Convert stream event to WebSocket format
                            let websocket_message = serde_json::json!({
                                "type": "dashboard_data",
                                "data": event.data,
                                "stream_id": event.stream_id,
                                "event_id": event.event_id,
                                "source": "redis_streams",
                                "timestamp": chrono::Utc::now().to_rfc3339()
                            });
                            
                            // Broadcast to all WebSocket clients
                            if let Err(e) = broadcast_tx.send(websocket_message.to_string()) {
                                println!("⚠️ WebSocket broadcast failed: {}", e);
                            } else {
                                println!("📡 Stream data broadcasted to WebSocket clients: {}", event.event_id);
                            }
                        }
                        
                        // Short delay after processing events
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    }
                    Err(e) => {
                        println!("⚠️ Stream consumption error: {}, retrying in 5s...", e);
                        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    }
                }
                
                // Small delay to prevent overwhelming the system
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        });
        
        println!("✅ Redis Streams → WebSocket consumer task spawned");
        Ok(())
    }
}
