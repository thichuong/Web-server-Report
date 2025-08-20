// src/features/websocket_service/broadcast_service.rs
//
// Broadcast service for real-time message distribution to WebSocket clients

use anyhow::{Result, anyhow};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::time::{interval, Duration};

use crate::features::external_apis::{MarketDataProvider, DashboardSummary};
use crate::features::cache_system::CacheManager;

/// Broadcast service for distributing real-time updates
#[derive(Debug, Clone)]
pub struct BroadcastService {
    broadcast_tx: broadcast::Sender<String>,
    update_interval_seconds: u64,
}

impl BroadcastService {
    pub async fn new(buffer_capacity: usize) -> Result<Self> {
        let (broadcast_tx, _) = broadcast::channel(buffer_capacity);
        
        Ok(Self {
            broadcast_tx,
            update_interval_seconds: 600, // 10 minutes default
        })
    }

    /// Get a receiver for subscribing to broadcasts
    pub fn subscribe(&self) -> broadcast::Receiver<String> {
        self.broadcast_tx.subscribe()
    }

    /// Broadcast a message to all connected clients
    pub async fn broadcast_message(&self, message: String) -> Result<()> {
        match self.broadcast_tx.send(message) {
            Ok(receiver_count) => {
                if receiver_count > 0 {
                    println!("📡 Broadcasted to {} WebSocket clients", receiver_count);
                } else {
                    println!("ℹ️ No WebSocket clients connected to broadcast to");
                }
            }
            Err(_) => {
                // Channel is closed or no receivers
                println!("⚠️ Broadcast channel error - no active receivers");
            }
        }
        Ok(())
    }

    /// Broadcast dashboard update
    pub async fn broadcast_dashboard_update(&self, dashboard_data: &DashboardSummary, source: Option<&str>) -> Result<()> {
        let message = json!({
            "type": "dashboard_update",
            "data": dashboard_data,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "source": source.unwrap_or("scheduled_update")
        }).to_string();

        self.broadcast_message(message).await
    }

    /// Broadcast system status update
    pub async fn broadcast_system_status(&self, status: &str, message: &str) -> Result<()> {
        let message = json!({
            "type": "system_status",
            "status": status,
            "message": message,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }).to_string();

        self.broadcast_message(message).await
    }

    /// Broadcast error notification
    pub async fn broadcast_error(&self, error_type: &str, error_message: &str) -> Result<()> {
        let message = json!({
            "type": "error",
            "error_type": error_type,
            "error_message": error_message,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }).to_string();

        self.broadcast_message(message).await
    }

    /// Start background updates with market data provider
    pub async fn start_background_updates(
        &self, 
        market_data_provider: Arc<MarketDataProvider>,
        cache_manager: Option<Arc<CacheManager>>
    ) -> Result<()> {
        let broadcast_service = self.clone();
        let update_interval = self.update_interval_seconds;

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(update_interval));
            let mut consecutive_failures = 0u32;

            println!("🚀 Starting background dashboard updates every {}s", update_interval);

            loop {
                interval.tick().await;
                
                println!("🔄 Starting scheduled dashboard data update...");
                
                match Self::update_and_broadcast_dashboard(
                    &market_data_provider, 
                    &broadcast_service,
                    cache_manager.as_ref()
                ).await {
                    Ok(_) => {
                        if consecutive_failures > 0 {
                            println!("✅ Dashboard data updated successfully after {} consecutive failures", consecutive_failures);
                        }
                        consecutive_failures = 0;
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

                        // Broadcast error notification
                        let _ = broadcast_service.broadcast_error(
                            "dashboard_update_failed",
                            &format!("Failed to update dashboard: {}", e)
                        ).await;
                    }
                }
            }
        });

        // Fetch and broadcast initial data with retry
        println!("🔄 Fetching initial dashboard data...");
        for attempt in 1..=3 {
            match Self::update_and_broadcast_dashboard(&market_data_provider, self, cache_manager.as_ref()).await {
                Ok(_) => {
                    println!("✅ Initial dashboard data fetched and broadcasted successfully");
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

        Ok(())
    }

    /// Update dashboard data and broadcast to clients
    async fn update_and_broadcast_dashboard(
        market_data_provider: &MarketDataProvider,
        broadcast_service: &BroadcastService,
        cache_manager: Option<&Arc<CacheManager>>
    ) -> Result<()> {
        // Fetch dashboard summary through market data provider
        let summary = market_data_provider.fetch_dashboard_summary()
            .await
            .map_err(|e| anyhow!("Failed to fetch dashboard data: {}", e))?;

        println!("✅ Dashboard data fetched successfully");

        // Broadcast the update
        broadcast_service.broadcast_dashboard_update(&summary, Some("background_update")).await?;

        Ok(())
    }

    /// Force update dashboard data (e.g., on user request)
    pub async fn force_update_dashboard(
        &self,
        market_data_provider: &MarketDataProvider
    ) -> Result<DashboardSummary> {
        println!("🔄 Force updating dashboard data...");
        
        let summary = market_data_provider.fetch_dashboard_summary()
            .await
            .map_err(|e| anyhow!("Failed to force update dashboard: {}", e))?;

        println!("✅ Dashboard data force-updated successfully");

        // Broadcast the force update
        self.broadcast_dashboard_update(&summary, Some("force_update")).await?;

        Ok(summary)
    }

    /// Get broadcast channel statistics
    pub fn get_stats(&self) -> BroadcastStats {
        BroadcastStats {
            receiver_count: self.broadcast_tx.receiver_count(),
            channel_capacity: self.broadcast_tx.capacity(),
            update_interval_seconds: self.update_interval_seconds,
        }
    }
}

/// Broadcast service statistics
#[derive(Debug)]
pub struct BroadcastStats {
    pub receiver_count: usize,
    pub channel_capacity: usize,
    pub update_interval_seconds: u64,
}
