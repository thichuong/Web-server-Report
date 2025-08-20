// src/features/shared_components/state/mod.rs
//
// Shared application state structure and initialization

use sqlx::PgPool;
use std::sync::{Arc, atomic::AtomicUsize};
use tokio::sync::RwLock;
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};

use super::models::common::{SystemStatus, PerformanceMetrics};

/// Configuration for feature initialization
#[derive(Debug, Clone)]
pub struct FeatureConfig {
    pub database_url: String,
    pub redis_url: String,
    pub taapi_secret: String,
}

/// Shared application state components
#[derive(Debug)]
pub struct SharedState {
    /// Database connection pool
    pub db: PgPool,
    /// Request counter for monitoring
    pub request_counter: AtomicUsize,
    /// Server start time for uptime monitoring
    pub start_time: Instant,
    /// System health status
    pub system_status: RwLock<SystemStatus>,
}

impl SharedState {
    pub async fn new(config: &FeatureConfig) -> Result<Self, anyhow::Error> {
        // Initialize database connection
        let db = PgPool::connect(&config.database_url).await
            .map_err(|e| anyhow::anyhow!("Failed to connect to database: {}", e))?;

        Ok(Self {
            db,
            request_counter: AtomicUsize::new(0),
            start_time: Instant::now(),
            system_status: RwLock::new(SystemStatus::Healthy),
        })
    }

    /// Get current uptime in seconds
    pub fn uptime_seconds(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }

    /// Increment request counter and return new count
    pub fn increment_requests(&self) -> usize {
        self.request_counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1
    }

    /// Get current request count
    pub fn request_count(&self) -> usize {
        self.request_counter.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Update system status
    pub async fn set_system_status(&self, status: SystemStatus) {
        *self.system_status.write().await = status;
    }

    /// Get current system status
    pub async fn get_system_status(&self) -> SystemStatus {
        self.system_status.read().await.clone()
    }

    /// Calculate current performance metrics
    pub async fn get_performance_metrics(&self) -> PerformanceMetrics {
        let uptime = self.uptime_seconds();
        let total_requests = self.request_count();
        
        PerformanceMetrics {
            response_time_ms: 0, // To be calculated by individual requests
            cache_hit_rate: 0.0, // To be calculated by cache layer
            requests_per_second: if uptime > 0 { total_requests as f64 / uptime as f64 } else { 0.0 },
            active_connections: 0, // To be tracked by websocket layer
            memory_usage_mb: 0.0, // To be calculated by system monitoring
        }
    }
}

/// Context passed to feature initialization functions
#[derive(Debug)]
pub struct FeatureContext {
    pub config: FeatureConfig,
    pub shared_state: Arc<SharedState>,
}

impl FeatureContext {
    pub async fn new(database_url: &str, redis_url: &str, taapi_secret: &str) -> Result<Self, anyhow::Error> {
        let config = FeatureConfig {
            database_url: database_url.to_string(),
            redis_url: redis_url.to_string(),
            taapi_secret: taapi_secret.to_string(),
        };

        let shared_state = Arc::new(SharedState::new(&config).await?);

        Ok(Self {
            config,
            shared_state,
        })
    }
}

/// Trait for features that need shared state access
pub trait WithSharedState {
    fn get_shared_state(&self) -> &Arc<SharedState>;
}

/// Helper macros for accessing shared state components
#[macro_export]
macro_rules! db_pool {
    ($state:expr) => {
        &$state.get_shared_state().db
    };
}

#[macro_export]
macro_rules! increment_requests {
    ($state:expr) => {
        $state.get_shared_state().increment_requests()
    };
}
