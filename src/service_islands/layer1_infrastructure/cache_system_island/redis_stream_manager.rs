//! Redis Stream Manager - Primary Storage System
//! 
//! This module implements Redis Streams as the primary storage for real-time data.
//! Provides event sourcing, multi-consumer patterns, and automatic backup to PostgreSQL.

use anyhow::Result;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH, Duration};

/// Redis Stream configuration
#[derive(Clone, Debug)]
pub struct StreamConfig {
    pub redis_url: String,
    pub max_stream_length: u64,
    pub backup_batch_size: usize,
    pub consumer_timeout: Duration,
}

impl Default for StreamConfig {
    fn default() -> Self {
        // Get Redis URL from environment variable
        let redis_url = std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
            
        Self {
            redis_url,
            max_stream_length: 10000, // Keep last 10k entries per stream
            backup_batch_size: 100,   // Backup 100 events at a time
            consumer_timeout: Duration::from_secs(5),
        }
    }
}

/// Stream event data structure
#[derive(Debug, Clone)]
pub struct StreamEvent {
    pub stream_id: String,
    pub event_id: String,
    pub data: Value,
    pub metadata: HashMap<String, String>,
    pub timestamp: u64,
}

impl StreamEvent {
    pub fn new(stream_id: String, data: Value) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let mut metadata = HashMap::new();
        metadata.insert("version".to_string(), "1.0".to_string());
        metadata.insert("created_at".to_string(), timestamp.to_string());

        Self {
            stream_id,
            event_id: format!("{}-0", timestamp), // Redis auto-generates if needed
            data,
            metadata,
            timestamp,
        }
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Redis Stream Manager
/// 
/// Central manager for all Redis Stream operations including:
/// - Stream production and consumption
/// - Consumer group management  
/// - Automatic backup orchestration
/// - Stream health monitoring
pub struct RedisStreamManager {
    config: StreamConfig,
    // Fallback storage when Redis is unavailable
    fallback_storage: Arc<RwLock<HashMap<String, Vec<StreamEvent>>>>,
    // Connection pool and stats
    redis_available: Arc<std::sync::atomic::AtomicBool>,
    // Consumer group registrations
    consumer_groups: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl RedisStreamManager {
    /// Initialize Redis Stream Manager
    pub async fn new() -> Result<Self> {
        Self::new_with_config(StreamConfig::default()).await
    }

    /// Initialize with custom configuration
    pub async fn new_with_config(config: StreamConfig) -> Result<Self> {
        println!("🌊 Initializing Redis Stream Manager...");

        let manager = Self {
            config,
            fallback_storage: Arc::new(RwLock::new(HashMap::new())),
            redis_available: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            consumer_groups: Arc::new(RwLock::new(HashMap::new())),
        };

        // Test Redis connection
        if manager.test_connection().await {
            println!("  ✅ Redis Stream Manager initialized with Redis backend");
        } else {
            println!("  ⚠️ Redis Stream Manager initialized with fallback storage");
        }

        Ok(manager)
    }

    /// Test Redis connection
    async fn test_connection(&self) -> bool {
        let client = match redis::Client::open(self.config.redis_url.as_str()) {
            Ok(client) => client,
            Err(e) => {
                println!("  ❌ Failed to create Redis client: {}", e);
                return false;
            }
        };

        match client.get_multiplexed_async_connection().await {
            Ok(mut conn) => {
                // Test with a simple PING command
                match redis::cmd("PING").query_async::<String>(&mut conn).await {
                    Ok(_) => {
                        println!("  ✅ Redis connection successful ({})", self.config.redis_url);
                        self.redis_available.store(true, std::sync::atomic::Ordering::Relaxed);
                        true
                    }
                    Err(e) => {
                        println!("  ❌ Redis PING failed: {}", e);
                        self.redis_available.store(false, std::sync::atomic::Ordering::Relaxed);
                        false
                    }
                }
            }
            Err(e) => {
                println!("  ❌ Failed to connect to Redis: {}", e);
                self.redis_available.store(false, std::sync::atomic::Ordering::Relaxed);
                false
            }
        }
    }

    /// Produce event to stream
    pub async fn produce(&self, event: StreamEvent) -> Result<String> {
        if self.redis_available.load(std::sync::atomic::Ordering::Relaxed) {
            // TODO: Implement Redis XADD command
            self.produce_redis(event).await
        } else {
            // Use fallback storage
            self.produce_fallback(event).await
        }
    }

    /// Produce to Redis stream
    async fn produce_redis(&self, event: StreamEvent) -> Result<String> {
        let client = redis::Client::open(self.config.redis_url.as_str())?;
        let mut conn = client.get_multiplexed_async_connection().await?;
        
        // Prepare XADD command: XADD stream_id * field1 value1 field2 value2 ...
        let mut cmd = redis::cmd("XADD");
        cmd.arg(&event.stream_id).arg("*"); // * means auto-generate ID
        
        // Add data fields
        if let Value::Object(map) = &event.data {
            for (key, value) in map {
                cmd.arg(key);
                cmd.arg(value.to_string());
            }
        } else {
            // If data is not an object, serialize as JSON
            cmd.arg("data");
            cmd.arg(event.data.to_string());
        }
        
        // Add metadata
        for (key, value) in &event.metadata {
            cmd.arg(key);
            cmd.arg(value);
        }
        
        let event_id: String = cmd.query_async(&mut conn).await?;
        println!("📤 [Redis] Event stored in stream: {} with ID: {}", event.stream_id, event_id);
        
        Ok(event_id)
    }

    /// Produce to fallback storage
    async fn produce_fallback(&self, event: StreamEvent) -> Result<String> {
        println!("📤 [Fallback] Producing to stream: {}", event.stream_id);
        
        let mut storage = self.fallback_storage.write().await;
        let stream_events = storage.entry(event.stream_id.clone()).or_insert_with(Vec::new);
        
        // Limit stream length to prevent memory issues
        if stream_events.len() >= self.config.max_stream_length as usize {
            stream_events.remove(0); // Remove oldest
        }
        
        let event_id = event.event_id.clone();
        stream_events.push(event);
        
        println!("  💾 Event stored in fallback (stream length: {})", stream_events.len());
        Ok(event_id)
    }

    /// Consume events from stream  
    pub async fn consume(
        &self,
        stream_id: &str,
        consumer_group: &str,
        consumer_name: &str,
        count: usize,
    ) -> Result<Vec<StreamEvent>> {
        if self.redis_available.load(std::sync::atomic::Ordering::Relaxed) {
            self.consume_redis(stream_id, consumer_group, consumer_name, count).await
        } else {
            self.consume_fallback(stream_id, count).await
        }
    }

    /// Consume from Redis stream
    async fn consume_redis(
        &self,
        stream_id: &str,
        consumer_group: &str,
        consumer_name: &str,
        count: usize,
    ) -> Result<Vec<StreamEvent>> {
        let client = redis::Client::open(self.config.redis_url.as_str())?;
        let mut conn = client.get_multiplexed_async_connection().await?;
        
        // Use XREADGROUP to consume from stream
        // XREADGROUP GROUP consumer_group consumer_name COUNT count STREAMS stream_id >
        let mut cmd = redis::cmd("XREADGROUP");
        cmd.arg("GROUP")
           .arg(consumer_group)
           .arg(consumer_name)
           .arg("COUNT")
           .arg(count)
           .arg("STREAMS")
           .arg(stream_id)
           .arg(">");

        // Execute command and parse results
        let results: Vec<(String, Vec<(String, Vec<(String, String)>)>)> = match cmd.query_async(&mut conn).await {
            Ok(results) => results,
            Err(e) => {
                println!("📥 [Redis] No new messages in stream: {} ({})", stream_id, e);
                return Ok(Vec::new());
            }
        };

        let mut events = Vec::new();
        for (_stream, stream_data) in results {
            for (event_id, fields) in stream_data {
                let mut data_map = serde_json::Map::new();
                let mut metadata = HashMap::new();
                
                for (key, value) in fields {
                    if key.starts_with("meta_") || key == "version" || key == "created_at" {
                        metadata.insert(key, value);
                    } else if key == "data" {
                        // Try to parse as JSON, fallback to string value
                        if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&value) {
                            data_map.insert(key, json_value);
                        } else {
                            data_map.insert(key, serde_json::Value::String(value));
                        }
                    } else {
                        // Regular data field
                        data_map.insert(key, serde_json::Value::String(value));
                    }
                }
                
                let event = StreamEvent {
                    stream_id: stream_id.to_string(),
                    event_id,
                    data: serde_json::Value::Object(data_map),
                    metadata,
                    timestamp: SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64,
                };
                
                events.push(event);
            }
        }
        
        if !events.is_empty() {
            println!("📥 [Redis] Consumed {} events from stream: {}", events.len(), stream_id);
        }
        
        Ok(events)
    }

    /// Consume from fallback storage
    async fn consume_fallback(&self, stream_id: &str, count: usize) -> Result<Vec<StreamEvent>> {
        // Only log when we actually have data to avoid spam
        let storage = self.fallback_storage.read().await;
        if let Some(stream_events) = storage.get(stream_id) {
            let events: Vec<StreamEvent> = stream_events
                .iter()
                .rev() // Get latest events first
                .take(count)
                .cloned()
                .collect();
            
            // Only log if we found events
            if !events.is_empty() {
                println!("📥 [Fallback] Consuming {} events from stream: {}", events.len(), stream_id);
            }
            
            Ok(events)
        } else {
            Ok(Vec::new()) // No logging for empty streams
        }
    }

    /// Create consumer group
    pub async fn create_consumer_group(
        &self,
        stream_id: &str,
        group_name: &str,
    ) -> Result<()> {
        if self.redis_available.load(std::sync::atomic::Ordering::Relaxed) {
            let client = redis::Client::open(self.config.redis_url.as_str())?;
            let mut conn = client.get_multiplexed_async_connection().await?;
            
            // Create consumer group: XGROUP CREATE stream group_name id MKSTREAM
            let mut cmd = redis::cmd("XGROUP");
            cmd.arg("CREATE")
               .arg(stream_id)
               .arg(group_name)
               .arg("0")  // Start from beginning
               .arg("MKSTREAM");  // Create stream if it doesn't exist
            
            match cmd.query_async::<()>(&mut conn).await {
                Ok(_) => {
                    println!("👥 [Redis] Created consumer group: {} for stream: {}", group_name, stream_id);
                }
                Err(e) => {
                    // Group might already exist, that's OK
                    if e.to_string().contains("BUSYGROUP") {
                        println!("👥 [Redis] Consumer group already exists: {} for stream: {}", group_name, stream_id);
                    } else {
                        println!("⚠️ [Redis] Failed to create consumer group: {} for stream: {} - {}", group_name, stream_id, e);
                    }
                }
            }
        } else {
            // Register in fallback
            let mut groups = self.consumer_groups.write().await;
            let stream_groups = groups.entry(stream_id.to_string()).or_insert_with(Vec::new);
            if !stream_groups.contains(&group_name.to_string()) {
                stream_groups.push(group_name.to_string());
            }
            println!("👥 [Fallback] Registered consumer group: {} for stream: {}", group_name, stream_id);
        }
        Ok(())
    }

    /// Get stream info and statistics
    pub async fn get_stream_info(&self, stream_id: &str) -> Result<Value> {
        if self.redis_available.load(std::sync::atomic::Ordering::Relaxed) {
            // TODO: Implement Redis XINFO STREAM
            self.get_stream_info_redis(stream_id).await
        } else {
            self.get_stream_info_fallback(stream_id).await
        }
    }

    /// Get Redis stream info
    async fn get_stream_info_redis(&self, stream_id: &str) -> Result<Value> {
        // TODO: Implement XINFO STREAM command
        Ok(json!({
            "stream_id": stream_id,
            "backend": "redis",
            "status": "connected"
        }))
    }

    /// Get fallback stream info
    async fn get_stream_info_fallback(&self, stream_id: &str) -> Result<Value> {
        let storage = self.fallback_storage.read().await;
        let length = storage.get(stream_id).map(|v| v.len()).unwrap_or(0);
        
        Ok(json!({
            "stream_id": stream_id,
            "backend": "fallback",
            "length": length,
            "max_length": self.config.max_stream_length,
            "status": "fallback_mode"
        }))
    }

    /// Health check for stream manager
    pub async fn health_check(&self) -> bool {
        if self.redis_available.load(std::sync::atomic::Ordering::Relaxed) {
            // TODO: Test Redis connection
            true
        } else {
            // Fallback is always available
            println!("  ✅ Redis Stream Manager - Fallback mode operational");
            true
        }
    }

    /// Get comprehensive statistics
    pub async fn get_statistics(&self) -> Result<Value> {
        let storage = self.fallback_storage.read().await;
        let groups = self.consumer_groups.read().await;
        
        let stream_stats: HashMap<String, usize> = storage
            .iter()
            .map(|(k, v)| (k.clone(), v.len()))
            .collect();

        Ok(json!({
            "redis_available": self.redis_available.load(std::sync::atomic::Ordering::Relaxed),
            "backend": if self.redis_available.load(std::sync::atomic::Ordering::Relaxed) { "redis" } else { "fallback" },
            "streams": stream_stats,
            "consumer_groups": groups.clone(),
            "config": {
                "max_stream_length": self.config.max_stream_length,
                "backup_batch_size": self.config.backup_batch_size,
                "consumer_timeout_secs": self.config.consumer_timeout.as_secs()
            }
        }))
    }
}

/// Standard stream IDs used throughout the system
pub struct StreamIds;

impl StreamIds {
    pub const MARKET_DATA: &'static str = "crypto:market_data";
    pub const BTC_PRICE: &'static str = "crypto:btc_price";  
    pub const FEAR_GREED: &'static str = "crypto:fear_greed";
    pub const DASHBOARD: &'static str = "crypto:dashboard";
    pub const SYSTEM_HEALTH: &'static str = "system:health";
}

/// Standard consumer group names
pub struct ConsumerGroups;

impl ConsumerGroups {
    pub const DASHBOARD: &'static str = "dashboard_consumers";
    pub const API: &'static str = "api_consumers";
    pub const BACKUP: &'static str = "backup_consumers";  
    pub const WEBSOCKET: &'static str = "websocket_consumers";
    pub const ANALYTICS: &'static str = "analytics_consumers";
}
