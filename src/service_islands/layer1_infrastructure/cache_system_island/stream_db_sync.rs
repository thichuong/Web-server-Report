//! Stream to Database Sync Service
//! 
//! Phase 3: Background service that syncs data from Redis Streams to PostgreSQL
//! for backup purposes. Redis Streams is primary storage, PostgreSQL is backup.

use std::sync::Arc;
use anyhow::Result;
use serde_json::Value;
use sqlx::PgPool;
use tokio::time::{Duration, interval};

use super::redis_stream_manager::RedisStreamManager;

/// Stream to Database Sync Service
/// 
/// Provides background synchronization from Redis Streams (primary) to PostgreSQL (backup).
/// Runs periodically to ensure data durability and backup consistency.
pub struct StreamDbSyncService {
    /// Redis Stream Manager for reading stream data
    stream_manager: Arc<RedisStreamManager>,
    /// PostgreSQL connection pool for backup storage
    db_pool: PgPool,
    /// Sync interval in seconds
    sync_interval: Duration,
    /// Last processed stream ID for each stream
    last_processed_ids: std::collections::HashMap<String, String>,
}

impl StreamDbSyncService {
    /// Create new Stream to Database Sync Service
    pub fn new(stream_manager: Arc<RedisStreamManager>, db_pool: PgPool) -> Self {
        Self {
            stream_manager,
            db_pool,
            sync_interval: Duration::from_secs(30), // Sync every 30 seconds
            last_processed_ids: std::collections::HashMap::new(),
        }
    }

    /// Start background sync service
    /// 
    /// Spawns a background task that continuously syncs new stream entries
    /// to PostgreSQL for backup purposes.
    pub async fn start_background_sync(&mut self) -> Result<()> {
        println!("🔄 Starting Redis Streams → PostgreSQL backup sync service...");
        
        let stream_manager = self.stream_manager.clone();
        let db_pool = self.db_pool.clone();
        let sync_interval = self.sync_interval;
        
        tokio::spawn(async move {
            let mut interval = interval(sync_interval);
            let mut last_ids: std::collections::HashMap<String, String> = std::collections::HashMap::new();
            
            loop {
                interval.tick().await;
                
                // Sync market_data stream
                if let Err(e) = Self::sync_stream_to_db(
                    &stream_manager,
                    &db_pool,
                    "market_data",
                    &mut last_ids
                ).await {
                    println!("⚠️ Failed to sync market_data stream: {}", e);
                }
                
                // Sync other streams as needed
                // Note: Add more streams here as the system grows
                
                println!("💾 Stream → DB sync cycle completed (every {:?})", sync_interval);
            }
        });
        
        println!("✅ Background sync service started (interval: {:?})", sync_interval);
        Ok(())
    }

    /// Sync a specific stream to database
    /// 
    /// Reads new entries from a Redis Stream and stores them in PostgreSQL backup.
    async fn sync_stream_to_db(
        stream_manager: &RedisStreamManager,
        db_pool: &PgPool,
        stream_name: &str,
        last_ids: &mut std::collections::HashMap<String, String>
    ) -> Result<()> {
        println!("📤 Syncing stream '{}' to PostgreSQL...", stream_name);
        
        // Get last processed ID for this stream
        let last_id = last_ids.get(stream_name).cloned();
        
        // Read new entries from stream
        match stream_manager.consume("market_data", "backup_sync", "sync_consumer", 10).await {
            Ok(stream_events) => {
                if stream_events.is_empty() {
                    // No new data to sync
                    return Ok(());
                }
                
                println!("📦 Found {} entries in stream '{}'", stream_events.len(), stream_name);
                
                // Filter out already processed entries
                let mut new_entries = Vec::new();
                for event in stream_events {
                    if let Some(ref last) = last_id {
                        if event.event_id > *last {
                            new_entries.push(event);
                        }
                    } else {
                        new_entries.push(event);
                    }
                }
                
                if new_entries.is_empty() {
                    // No new entries after filtering
                    return Ok(());
                }
                
                println!("📦 Found {} new entries to sync from '{}'", new_entries.len(), stream_name);
                
                for event in &new_entries {
                    // Store in PostgreSQL backup
                    if let Err(e) = Self::store_event_to_db(db_pool, stream_name, &event).await {
                        println!("⚠️ Failed to store event {} to DB: {}", event.event_id, e);
                        continue;
                    }
                    
                    // Update last processed ID
                    last_ids.insert(stream_name.to_string(), event.event_id.clone());
                }
                
                println!("✅ Synced {} entries from '{}' to PostgreSQL", new_entries.len(), stream_name);
            }
            Err(e) => {
                println!("⚠️ Failed to read from stream '{}': {}", stream_name, e);
            }
        }
        
        Ok(())
    }

    /// Store a stream event to PostgreSQL backup
    async fn store_event_to_db(
        db_pool: &PgPool,
        stream_name: &str,
        event: &crate::service_islands::layer1_infrastructure::cache_system_island::redis_stream_manager::StreamEvent
    ) -> Result<()> {
        // Create backup table for stream data if not exists
        let create_table_sql = format!(
            r#"
            CREATE TABLE IF NOT EXISTS stream_backup_{} (
                id SERIAL PRIMARY KEY,
                event_id TEXT NOT NULL UNIQUE,
                stream_id TEXT NOT NULL,
                data JSONB NOT NULL,
                metadata JSONB NOT NULL,
                event_timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
                backup_timestamp TIMESTAMP WITH TIME ZONE DEFAULT NOW()
            )
            "#,
            stream_name.replace("-", "_")  // Replace hyphens for valid table names
        );
        
        if let Err(e) = sqlx::query(&create_table_sql).execute(db_pool).await {
            println!("⚠️ Failed to create backup table: {}", e);
        }
        
        // Convert metadata to JSON
        let metadata_json = serde_json::to_value(&event.metadata)?;
        
        // Convert timestamp to DateTime
        let event_timestamp = chrono::DateTime::from_timestamp_millis(event.timestamp as i64)
            .unwrap_or_else(|| chrono::Utc::now());
        
        // Insert into backup table
        let insert_sql = format!(
            r#"
            INSERT INTO stream_backup_{} (event_id, stream_id, data, metadata, event_timestamp)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (event_id) DO UPDATE SET
                data = EXCLUDED.data,
                metadata = EXCLUDED.metadata,
                backup_timestamp = NOW()
            "#,
            stream_name.replace("-", "_")
        );
        
        sqlx::query(&insert_sql)
            .bind(&event.event_id)
            .bind(&event.stream_id)
            .bind(&event.data)
            .bind(metadata_json)
            .bind(event_timestamp)
            .execute(db_pool)
            .await?;
        
        Ok(())
    }
}

