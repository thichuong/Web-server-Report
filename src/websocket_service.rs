#![allow(dead_code)]
// src/websocket_service.rs - WebSocket service với Redis cache

use axum::extract::ws::{WebSocket, Message};
use redis::{AsyncCommands, Client as RedisClient};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::time::{interval, Duration};

use crate::data_service::{DataService, DashboardSummary};
use crate::cache::CacheKeys;

// Use canonical cache key from CacheKeys::dashboard_summary() to ensure
// consistency with the multi-tier cache module (L1/L2).
const UPDATE_INTERVAL_SECONDS: u64 = 600; // 10 phút để tránh rate limit
const CACHE_TTL_SECONDS: u64 = 3600; // 1 giờ TTL cho Redis cache

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

    /// Khởi tạo background task để update data định kỳ với better error handling
    pub async fn start_background_updates(&self) {
        let redis_client = self.redis_client.clone();
        let data_service = self.data_service.clone();
        let broadcast_tx = self.broadcast_tx.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(UPDATE_INTERVAL_SECONDS));
            let mut consecutive_failures = 0u32;

            loop {
                interval.tick().await;
                
                println!("🔄 Starting scheduled dashboard data update...");
                
                match Self::update_dashboard_data(&redis_client, &data_service, &broadcast_tx).await {
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
            match Self::update_dashboard_data(&self.redis_client, &self.data_service, &self.broadcast_tx).await {
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

    /// Update dashboard data và broadcast tới WebSocket clients với improved caching
    async fn update_dashboard_data(
        redis_client: &RedisClient,
        data_service: &DataService,
        broadcast_tx: &broadcast::Sender<String>,
    ) -> Result<(), anyhow::Error> {
        // Fetch fresh data với timeout
        let fetch_timeout = Duration::from_secs(30); // 30 second timeout
        let summary = tokio::time::timeout(fetch_timeout, data_service.fetch_dashboard_summary())
            .await
            .map_err(|_| anyhow::anyhow!("Data fetch timeout after 30 seconds"))?
            .map_err(|e| anyhow::anyhow!("Failed to fetch dashboard data: {}", e))?;

        // Store in Redis with proper error handling
        match redis_client.get_multiplexed_async_connection().await {
            Ok(mut redis_conn) => {
                let summary_json = serde_json::to_string(&summary)?;
                
                // Set the data and TTL separately using canonical key
                let key = CacheKeys::dashboard_summary();
                let _: () = redis_conn.set(&key, &summary_json).await?;
                let _: () = redis_conn.expire(&key, CACHE_TTL_SECONDS as i64).await?;

                println!("✅ Dashboard data cached to Redis with TTL: {}s", CACHE_TTL_SECONDS);
            },
            Err(e) => {
                eprintln!("⚠️ Redis connection failed, data not cached: {}", e);
                // Continue without caching - still broadcast to WebSocket clients
            }
        }

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

    /// Lấy dashboard data từ Redis cache với fallback logic
    pub async fn get_cached_dashboard_data(&self) -> Result<Option<DashboardSummary>, anyhow::Error> {
        match self.redis_client.get_multiplexed_async_connection().await {
            Ok(mut redis_conn) => {
                let key = CacheKeys::dashboard_summary();
                match redis_conn.get::<_, Option<String>>(&key).await {
                    Ok(Some(data)) => {
                        match serde_json::from_str(&data) {
                            Ok(summary) => {
                                println!("✅ Dashboard data retrieved from Redis cache");
                                Ok(Some(summary))
                            },
                            Err(e) => {
                                eprintln!("⚠️ Failed to parse cached data: {}", e);
                                // Clear corrupted cache
                                let _: Result<(), _> = redis_conn.del(&key).await;
                                Ok(None)
                            }
                        }
                    },
                    Ok(None) => {
                        println!("ℹ️ No cached dashboard data found in Redis");
                        Ok(None)
                    },
                    Err(e) => {
                        eprintln!("⚠️ Redis query failed: {}", e);
                        Ok(None) // Return None instead of error for graceful fallback
                    }
                }
            },
            Err(e) => {
                eprintln!("⚠️ Redis connection failed: {}", e);
                Ok(None) // Return None for graceful fallback
            }
        }
    }

    /// Get dashboard data with intelligent fallback (cache -> fresh fetch)
    pub async fn get_dashboard_data_with_fallback(&self) -> Result<DashboardSummary, anyhow::Error> {
        // Try cache first
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

        // Fallback to fresh fetch
        println!("🔄 Fetching fresh dashboard data...");
        self.force_update_dashboard().await
    }

    /// Force update dashboard data (ví dụ từ API endpoint) với better error handling
    pub async fn force_update_dashboard(&self) -> Result<DashboardSummary, anyhow::Error> {
        println!("🔄 Force updating dashboard data...");
        
        // Fetch with timeout
        let fetch_timeout = Duration::from_secs(45); // Longer timeout for force update
        let summary = tokio::time::timeout(fetch_timeout, self.data_service.fetch_dashboard_summary())
            .await
            .map_err(|_| anyhow::anyhow!("Force update timeout after 45 seconds"))?
            .map_err(|e| anyhow::anyhow!("Failed to fetch dashboard data: {}", e))?;

        // Store in Redis with proper error handling
        match self.redis_client.get_multiplexed_async_connection().await {
            Ok(mut redis_conn) => {
                let summary_json = serde_json::to_string(&summary)?;
                
                // Set the data and TTL separately
                let key = CacheKeys::dashboard_summary();
                let _: () = redis_conn.set(&key, &summary_json).await?;
                let _: () = redis_conn.expire(&key, CACHE_TTL_SECONDS as i64).await?;

                println!("✅ Dashboard data force-updated and cached to Redis");
            },
            Err(e) => {
                eprintln!("⚠️ Redis connection failed during force update: {}", e);
                // Continue without caching
            }
        }

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
