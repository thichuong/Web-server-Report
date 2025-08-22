#![allow(dead_code)]
// src/websocket_service.rs - WebSocket service với CacheManager

use axum::extract::ws::{WebSocket, Message};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::time::{interval, Duration};

use crate::data_service::{DataService, DashboardSummary};
use crate::cache::CacheManager;

// Use CacheManager for unified L1+L2 caching instead of direct Redis operations
const UPDATE_INTERVAL_SECONDS: u64 = 600; // 10 phút để tránh rate limit

pub struct WebSocketService {
    cache_manager: Arc<CacheManager>,
    data_service: Arc<DataService>,
    broadcast_tx: broadcast::Sender<String>,
}

impl WebSocketService {
    pub fn new(cache_manager: Arc<CacheManager>, data_service: DataService) -> Result<Self, anyhow::Error> {
        let (broadcast_tx, _) = broadcast::channel(100);

        Ok(Self {
            cache_manager,
            data_service: Arc::new(data_service),
            broadcast_tx,
        })
    }

    pub fn get_broadcast_receiver(&self) -> broadcast::Receiver<String> {
        self.broadcast_tx.subscribe()
    }

    /// Khởi tạo background task để update data định kỳ với CacheManager
    pub async fn start_background_updates(&self) {
        let cache_manager = self.cache_manager.clone();
        let data_service = self.data_service.clone();
        let broadcast_tx = self.broadcast_tx.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(UPDATE_INTERVAL_SECONDS));
            let mut consecutive_failures = 0u32;

            loop {
                interval.tick().await;
                
                println!("🔄 Starting scheduled dashboard data update...");
                
                match Self::update_dashboard_data(&cache_manager, &data_service, &broadcast_tx).await {
                    Ok(_) => {
                        println!("✅ Dashboard data updated successfully after {} consecutive failures", consecutive_failures);
                        consecutive_failures = 0; // Reset failure counter on success
                    },
                    Err(e) => {
                        consecutive_failures += 1;
                        eprintln!("❌ Failed to update dashboard data (attempt {}): {}", consecutive_failures, e);
                        
                        // Exponential backoff for consecutive failures
                        if consecutive_failures > 3 {
                            let backoff_minutes = std::cmp::min(consecutive_failures * 2, 30); // Max 30 minutes
                            println!("⏳ Too many consecutive failures, backing off for {} minutes", backoff_minutes);
                            tokio::time::sleep(Duration::from_secs(backoff_minutes as u64 * 60)).await;
                        }
                    }
                }
            }
        });

        // Fetch initial data with retry
        println!("🔄 Fetching initial dashboard data...");
        for attempt in 1..=3 {
            match Self::update_dashboard_data(&self.cache_manager, &self.data_service, &self.broadcast_tx).await {
                Ok(_) => {
                    println!("✅ Initial dashboard data fetched successfully");
                    break;
                },
                Err(e) => {
                    eprintln!("⚠️ Failed to fetch initial dashboard data (attempt {}): {}", attempt, e);
                    if attempt < 3 {
                        tokio::time::sleep(Duration::from_secs(5)).await;
                    }
                }
            }
        }
    }

    /// Update dashboard data và broadcast tới WebSocket clients với CacheManager
    async fn update_dashboard_data(
        _cache_manager: &CacheManager,
        data_service: &DataService,
        broadcast_tx: &broadcast::Sender<String>,
    ) -> Result<(), anyhow::Error> {
        // Use DataService's public fetch_dashboard_summary which leverages CacheManager if available
        let summary = data_service.fetch_dashboard_summary()
            .await.map_err(|e| anyhow::anyhow!("Failed to fetch dashboard data via DataService: {}", e))?;

        println!("✅ Dashboard data fetched via DataService with CacheManager integration");

        // Broadcast to WebSocket clients
        let message = json!({
            "type": "dashboard_update",
            "data": summary,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }).to_string();

        if let Err(e) = broadcast_tx.send(message) {
            println!("ℹ️ No WebSocket clients connected to broadcast to: {}", e);
        } else {
            println!("📡 Dashboard data broadcasted to WebSocket clients");
        }

        Ok(())
    }

    /// Lấy dashboard data từ CacheManager với L1+L2 fallback logic
    pub async fn get_cached_dashboard_data(&self) -> Result<Option<DashboardSummary>, anyhow::Error> {
        // Use CacheManager to get cached data with automatic L1->L2->compute fallback
        use crate::cache::CacheKeys;
        let key = CacheKeys::dashboard_summary();
        
        match self.cache_manager.get::<DashboardSummary>(&key).await {
            Ok(Some(data)) => {
                println!("✅ Dashboard data retrieved from cache (L1 or L2)");
                Ok(Some(data))
            },
            Ok(None) => {
                println!("ℹ️ No cached dashboard data found in L1 or L2");
                Ok(None)
            },
            Err(e) => {
                eprintln!("⚠️ Cache error: {}", e);
                Ok(None) // Return None for graceful fallback
            }
        }
    }

    /// Get dashboard data with intelligent fallback (L1+L2 cache -> fresh fetch)
    pub async fn get_dashboard_data_with_fallback(&self) -> Result<DashboardSummary, anyhow::Error> {
        // Try L1+L2 cache first via CacheManager
        match self.get_cached_dashboard_data().await {
            Ok(Some(cached_data)) => {
                // Check if data is recent (within 15 minutes)
                let data_age = chrono::Utc::now().signed_duration_since(cached_data.last_updated);
                if data_age.num_minutes() < 15 {
                    println!("✅ Using fresh cached data ({}m old)", data_age.num_minutes());
                    return Ok(cached_data);
                } else {
                    println!("⏰ Cached data is stale ({}m old), fetching fresh data", data_age.num_minutes());
                }
            },
            Ok(None) => {
                println!("ℹ️ No cached data available, fetching fresh data");
            },
            Err(e) => {
                eprintln!("⚠️ Cache error: {}, fetching fresh data", e);
            }
        }

        // Fallback to fresh fetch via CacheManager (will populate both L1 and L2)
        println!("🔄 Fetching fresh dashboard data via CacheManager...");
        self.force_update_dashboard().await
    }

    /// Force update dashboard data với CacheManager
    pub async fn force_update_dashboard(&self) -> Result<DashboardSummary, anyhow::Error> {
        println!("🔄 Force updating dashboard data via DataService...");
        
        // Use DataService's public fetch_dashboard_summary which leverages CacheManager if available
        let summary = self.data_service.fetch_dashboard_summary()
            .await.map_err(|e| anyhow::anyhow!("Failed to force update via DataService: {}", e))?;

        println!("✅ Dashboard data force-updated via DataService with CacheManager integration");

        // Broadcast to WebSocket clients
        let message = json!({
            "type": "dashboard_update",
            "data": summary,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "source": "force_update"
        }).to_string();

        if let Err(e) = self.broadcast_tx.send(message) {
            println!("ℹ️ No WebSocket clients connected for force update broadcast: {}", e);
        } else {
            println!("📡 Force update broadcasted to WebSocket clients");
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
