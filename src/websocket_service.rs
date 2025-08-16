// src/websocket_service.rs - WebSocket service với Redis cache

use axum::extract::ws::{WebSocket, Message};
use redis::{AsyncCommands, Client as RedisClient};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::time::{interval, Duration};

use crate::data_service::{DataService, DashboardSummary};

const REDIS_KEY_DASHBOARD: &str = "dashboard_summary";
const UPDATE_INTERVAL_SECONDS: u64 = 300; // 5 phút

pub struct WebSocketService {
    redis_client: RedisClient,
    data_service: Arc<DataService>,
    broadcast_tx: broadcast::Sender<String>,
}

impl WebSocketService {
    pub fn new(redis_url: &str, data_service: DataService) -> Result<Self, anyhow::Error> {
        let redis_client = RedisClient::open(redis_url)?;
        let (broadcast_tx, _) = broadcast::channel(100);

        Ok(Self {
            redis_client,
            data_service: Arc::new(data_service),
            broadcast_tx,
        })
    }

    pub fn get_broadcast_receiver(&self) -> broadcast::Receiver<String> {
        self.broadcast_tx.subscribe()
    }

    /// Khởi tạo background task để update data định kỳ
    pub async fn start_background_updates(&self) {
        let redis_client = self.redis_client.clone();
        let data_service = self.data_service.clone();
        let broadcast_tx = self.broadcast_tx.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(UPDATE_INTERVAL_SECONDS));

            loop {
                interval.tick().await;
                
                match Self::update_dashboard_data(&redis_client, &data_service, &broadcast_tx).await {
                    Ok(_) => println!("✅ Dashboard data updated successfully"),
                    Err(e) => eprintln!("❌ Failed to update dashboard data: {}", e),
                }
            }
        });

        // Fetch initial data
        if let Err(e) = Self::update_dashboard_data(&self.redis_client, &self.data_service, &self.broadcast_tx).await {
            eprintln!("⚠️ Failed to fetch initial dashboard data: {}", e);
        }
    }

    /// Update dashboard data và broadcast tới WebSocket clients
    async fn update_dashboard_data(
        redis_client: &RedisClient,
        data_service: &DataService,
        broadcast_tx: &broadcast::Sender<String>,
    ) -> Result<(), anyhow::Error> {
        // Fetch fresh data
        let summary = data_service.fetch_dashboard_summary().await?;

        // Store in Redis
        let mut redis_conn = redis_client.get_multiplexed_async_connection().await?;
        let summary_json = serde_json::to_string(&summary)?;
        redis_conn.set::<_, _, ()>(REDIS_KEY_DASHBOARD, &summary_json).await?;
        redis_conn.expire::<_, ()>(REDIS_KEY_DASHBOARD, 3600).await?; // Expire after 1 hour

        // Broadcast to WebSocket clients
        let message = json!({
            "type": "dashboard_update",
            "data": summary
        }).to_string();

        if let Err(e) = broadcast_tx.send(message) {
            eprintln!("⚠️ No WebSocket clients to broadcast to: {}", e);
        }

        Ok(())
    }

    /// Lấy dashboard data từ Redis cache
    pub async fn get_cached_dashboard_data(&self) -> Result<Option<DashboardSummary>, anyhow::Error> {
        let mut redis_conn = self.redis_client.get_multiplexed_async_connection().await?;
        
        match redis_conn.get::<_, Option<String>>(REDIS_KEY_DASHBOARD).await? {
            Some(data) => {
                let summary: DashboardSummary = serde_json::from_str(&data)?;
                Ok(Some(summary))
            }
            None => Ok(None),
        }
    }

    /// Force update dashboard data (ví dụ từ API endpoint)
    pub async fn force_update_dashboard(&self) -> Result<DashboardSummary, anyhow::Error> {
        let summary = self.data_service.fetch_dashboard_summary().await?;

        // Store in Redis
        let mut redis_conn = self.redis_client.get_multiplexed_async_connection().await?;
        let summary_json = serde_json::to_string(&summary)?;
        redis_conn.set::<_, _, ()>(REDIS_KEY_DASHBOARD, &summary_json).await?;
        redis_conn.expire::<_, ()>(REDIS_KEY_DASHBOARD, 3600).await?;

        // Broadcast to WebSocket clients
        let message = json!({
            "type": "dashboard_update",
            "data": summary
        }).to_string();

        if let Err(e) = self.broadcast_tx.send(message) {
            eprintln!("⚠️ No WebSocket clients to broadcast to: {}", e);
        }

        Ok(summary)
    }

    /// Handle WebSocket connection
    pub async fn handle_websocket(&self, mut socket: WebSocket) {
        println!("🔗 New WebSocket connection established");

        // Send current data immediately
        if let Ok(Some(current_data)) = self.get_cached_dashboard_data().await {
            let welcome_message = json!({
                "type": "dashboard_update",
                "data": current_data
            }).to_string();

            if socket.send(Message::Text(welcome_message)).await.is_err() {
                println!("❌ Failed to send welcome message");
                return;
            }
        }

        // Subscribe to broadcast updates
        let mut broadcast_rx = self.get_broadcast_receiver();

        loop {
            tokio::select! {
                // Handle incoming WebSocket messages
                msg = socket.recv() => {
                    match msg {
                        Some(Ok(Message::Text(text))) => {
                            if text == "ping" {
                                if socket.send(Message::Text("pong".to_string())).await.is_err() {
                                    break;
                                }
                            }
                            // Handle other message types if needed
                        }
                        Some(Ok(Message::Close(_))) => {
                            println!("🔌 WebSocket connection closed");
                            break;
                        }
                        Some(Err(e)) => {
                            eprintln!("❌ WebSocket error: {}", e);
                            break;
                        }
                        None => break,
                        _ => {} // Ignore other message types
                    }
                }
                // Handle broadcast messages
                broadcast_msg = broadcast_rx.recv() => {
                    match broadcast_msg {
                        Ok(message) => {
                            if socket.send(Message::Text(message)).await.is_err() {
                                println!("❌ Failed to send broadcast message, client disconnected");
                                break;
                            }
                        }
                        Err(broadcast::error::RecvError::Closed) => {
                            println!("📡 Broadcast channel closed");
                            break;
                        }
                        Err(broadcast::error::RecvError::Lagged(_)) => {
                            println!("⚠️ WebSocket client lagged behind, continuing...");
                            continue;
                        }
                    }
                }
            }
        }

        println!("🔌 WebSocket connection terminated");
    }
}
