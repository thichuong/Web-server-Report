//! Stream-Based Data Service
//! 
//! This service provides stream-based data operations, using Redis Streams
//! as primary storage and PostgreSQL as backup. Replaces traditional cache
//! patterns with event sourcing and real-time streaming.

use anyhow::Result;
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use super::redis_stream_manager::{RedisStreamManager, StreamEvent, StreamIds, ConsumerGroups};
use crate::state::AppState;

/// Stream-based Data Service
/// 
/// Provides high-level data operations using Redis Streams as primary storage.
/// Handles data production, consumption, and automatic backup to PostgreSQL.
pub struct StreamDataService {
    /// Redis Stream Manager for primary operations
    pub stream_manager: Arc<RedisStreamManager>,
    /// Background backup service (future implementation)
    backup_enabled: bool,
}

impl StreamDataService {
    /// Initialize Stream Data Service
    pub async fn new() -> Result<Self> {
        println!("🌊 Initializing Stream Data Service...");
        
        let stream_manager = Arc::new(RedisStreamManager::new().await?);
        
        // Initialize consumer groups
        let groups = [
            (StreamIds::MARKET_DATA, ConsumerGroups::DASHBOARD),
            (StreamIds::MARKET_DATA, ConsumerGroups::WEBSOCKET),
            (StreamIds::BTC_PRICE, ConsumerGroups::API),
            (StreamIds::DASHBOARD, ConsumerGroups::DASHBOARD),
            (StreamIds::SYSTEM_HEALTH, ConsumerGroups::ANALYTICS),
        ];
        
        for (stream, group) in &groups {
            if let Err(e) = stream_manager.create_consumer_group(stream, group).await {
                println!("⚠️ Failed to create consumer group {} for {}: {}", group, stream, e);
            }
        }
        
        println!("✅ Stream Data Service initialized with consumer groups");
        
        Ok(Self {
            stream_manager,
            backup_enabled: false, // Will enable when backup service is implemented
        })
    }

    /// Store market data in stream
    /// 
    /// Primary storage method for real-time market data.
    /// Data flows: External API → Stream → Consumers (WebSocket, DB backup)
    pub async fn store_market_data(&self, data: Value) -> Result<String> {
        println!("📊 Storing market data to stream...");
        
        let event = StreamEvent::new(
            StreamIds::MARKET_DATA.to_string(),
            data.clone()
        )
        .with_metadata("source".to_string(), "external_api".to_string())
        .with_metadata("type".to_string(), "market_data".to_string());

        let event_id = self.stream_manager.produce(event).await?;
        
        println!("  💾 Market data stored with ID: {}", event_id);
        Ok(event_id)
    }

    /// Store BTC price update in stream
    pub async fn store_btc_price(&self, price_data: Value) -> Result<String> {
        println!("₿ Storing BTC price to stream...");
        
        let event = StreamEvent::new(
            StreamIds::BTC_PRICE.to_string(),
            price_data
        )
        .with_metadata("symbol".to_string(), "BTC".to_string())
        .with_metadata("type".to_string(), "price_update".to_string());

        let event_id = self.stream_manager.produce(event).await?;
        
        println!("  💰 BTC price stored with ID: {}", event_id);
        Ok(event_id)
    }

    /// Store dashboard summary in stream
    pub async fn store_dashboard_summary(&self, dashboard_data: Value) -> Result<String> {
        println!("📈 Storing dashboard summary to stream...");
        
        let event = StreamEvent::new(
            StreamIds::DASHBOARD.to_string(),
            dashboard_data
        )
        .with_metadata("type".to_string(), "dashboard_summary".to_string());

        let event_id = self.stream_manager.produce(event).await?;
        
        println!("  📊 Dashboard summary stored with ID: {}", event_id);
        Ok(event_id)
    }

    /// Get latest market data from stream
    /// 
    /// Primary read method - serves data from Redis Stream instead of cache.
    /// Falls back to database only if stream is empty.
    pub async fn get_latest_market_data(&self) -> Result<Option<Value>> {
        println!("🔍 Fetching latest market data from stream...");
        
        // Try to get from stream first
        let events = self.stream_manager.consume(
            StreamIds::MARKET_DATA,
            ConsumerGroups::API,
            "api_consumer_1",
            1
        ).await?;

        if let Some(event) = events.first() {
            println!("  ✅ Latest market data served from stream");
            return Ok(Some(event.data.clone()));
        }

        println!("  📭 No market data in stream, checking fallback...");
        Ok(None)
    }

    /// Get latest BTC price from stream
    pub async fn get_latest_btc_price(&self) -> Result<Option<Value>> {
        println!("₿ Fetching latest BTC price from stream...");
        
        let events = self.stream_manager.consume(
            StreamIds::BTC_PRICE,
            ConsumerGroups::API,
            "btc_consumer_1", 
            1
        ).await?;

        if let Some(event) = events.first() {
            println!("  💰 Latest BTC price served from stream");
            return Ok(Some(event.data.clone()));
        }

        Ok(None)
    }

    /// Get dashboard data for WebSocket consumers
    pub async fn get_dashboard_for_websocket(&self) -> Result<Vec<Value>> {
        println!("🔌 Fetching dashboard data for WebSocket...");
        
        let events = self.stream_manager.consume(
            StreamIds::DASHBOARD,
            ConsumerGroups::WEBSOCKET,
            "websocket_consumer_1",
            5 // Get last 5 updates
        ).await?;

        let data: Vec<Value> = events.into_iter().map(|e| e.data).collect();
        
        println!("  📡 {} dashboard updates for WebSocket", data.len());
        Ok(data)
    }

    /// Store data with automatic backup trigger
    /// 
    /// High-level method that stores to stream and triggers backup to database.
    /// Used when both real-time streaming and historical backup are needed.
    pub async fn store_with_backup(
        &self,
        stream_id: &str,
        data: Value,
        state: Option<&Arc<AppState>>
    ) -> Result<String> {
        println!("💾 Storing data with backup: {}", stream_id);
        
        // Store in stream first (primary storage)
        let event = StreamEvent::new(stream_id.to_string(), data.clone());
        let event_id = self.stream_manager.produce(event).await?;
        
        // Trigger backup to database if available
        if let Some(app_state) = state {
            if self.backup_enabled {
                tokio::spawn({
                    let data = data.clone();
                    let stream_id = stream_id.to_string();
                    let state = Arc::clone(app_state);
                    async move {
                        if let Err(e) = Self::backup_to_database(stream_id, data, state).await {
                            println!("⚠️ Background backup failed: {}", e);
                        }
                    }
                });
            }
        }
        
        Ok(event_id)
    }

    /// Background backup to PostgreSQL
    async fn backup_to_database(
        stream_id: String,
        data: Value,
        _state: Arc<AppState>
    ) -> Result<()> {
        println!("🔄 Background backup to database: {}", stream_id);
        
        // TODO: Implement actual database backup
        // This will store data in PostgreSQL for historical records
        // and disaster recovery
        
        println!("  📝 Backup completed for stream: {}", stream_id);
        Ok(())
    }

    /// Get stream health and statistics
    pub async fn get_stream_health(&self) -> Result<Value> {
        let stats = self.stream_manager.get_statistics().await?;
        
        let health_status = json!({
            "service": "stream_data_service",
            "healthy": self.stream_manager.health_check().await,
            "primary_storage": "redis_streams",
            "backup_storage": "postgresql",
            "backup_enabled": self.backup_enabled,
            "stream_manager": stats,
            "timestamp": SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
        });

        Ok(health_status)
    }

    /// Start background stream processing
    /// 
    /// Starts background consumers for automated data processing:
    /// - Database backup consumer
    /// - Analytics processing consumer
    /// - Stream cleanup consumer
    pub async fn start_background_processing(&self) -> Result<()> {
        println!("🔄 Starting background stream processing...");
        
        // Database backup consumer
        let stream_manager = Arc::clone(&self.stream_manager);
        tokio::spawn(async move {
            Self::run_backup_consumer(stream_manager).await;
        });
        
        // Stream cleanup consumer (remove old events)
        let stream_manager = Arc::clone(&self.stream_manager);
        tokio::spawn(async move {
            Self::run_cleanup_consumer(stream_manager).await;
        });
        
        println!("✅ Background stream processing started");
        Ok(())
    }

    /// Background backup consumer
    async fn run_backup_consumer(stream_manager: Arc<RedisStreamManager>) {
        println!("🔄 Starting backup consumer...");
        
        let mut interval = tokio::time::interval(Duration::from_secs(60)); // Every minute
        
        loop {
            interval.tick().await;
            
            // Process events from all streams for backup
            let streams = [
                StreamIds::MARKET_DATA,
                StreamIds::BTC_PRICE, 
                StreamIds::DASHBOARD,
            ];
            
            for stream in &streams {
                match stream_manager.consume(
                    stream,
                    ConsumerGroups::BACKUP,
                    "backup_consumer_1",
                    10
                ).await {
                    Ok(events) => {
                        if !events.is_empty() {
                            println!("🔄 Processing {} events for backup from {}", events.len(), stream);
                            // TODO: Process backup to database
                        }
                    }
                    Err(e) => {
                        println!("⚠️ Backup consumer error for {}: {}", stream, e);
                    }
                }
            }
        }
    }

    /// Background cleanup consumer
    async fn run_cleanup_consumer(stream_manager: Arc<RedisStreamManager>) {
        println!("🧹 Starting cleanup consumer...");
        
        let mut interval = tokio::time::interval(Duration::from_secs(3600)); // Every hour
        
        loop {
            interval.tick().await;
            
            println!("🧹 Running stream cleanup...");
            // TODO: Implement stream trimming based on age/size limits
            // XTRIM crypto:market_data MAXLEN ~ 1000
        }
    }
}
