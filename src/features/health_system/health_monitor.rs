//! Health Monitor - Core health checking functionality
//! 
//! Provides comprehensive system health monitoring including cache statistics,
//! system metrics, and component health tracking.

use crate::features::shared_components::{ApiResult, SystemStatus};
use crate::features::cache_system::CacheManager;
use serde_json::{json, Value};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::time::Instant;

/// Core health monitoring component
pub struct HealthMonitor {
    startup_time: Instant,
    request_counter: AtomicU64,
    last_health_check: Arc<std::sync::RwLock<Instant>>,
}

impl HealthMonitor {
    pub fn new() -> Self {
        Self {
            startup_time: Instant::now(),
            request_counter: AtomicU64::new(0),
            last_health_check: Arc::new(std::sync::RwLock::new(Instant::now())),
        }
    }

    /// Record a new request for health tracking
    pub fn record_request(&self) {
        self.request_counter.fetch_add(1, Ordering::Relaxed);
    }

    /// Get basic health status with system information
    pub async fn get_basic_health(&self) -> Value {
        // Update last health check time
        {
            let mut last_check = self.last_health_check.write().unwrap();
            *last_check = Instant::now();
        }

        // Record this health check as a request
        self.record_request();

        let uptime_secs = self.startup_time.elapsed().as_secs();
        let request_count = self.request_counter.load(Ordering::Relaxed);
        let cpu_count = num_cpus::get();

        json!({
            "status": "healthy",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "uptime_seconds": uptime_secs,
            "request_count": request_count,
            "cpu_count": cpu_count,
            "service": "AI Investment Report Server",
            "version": "1.0.0"
        })
    }

    /// Get comprehensive health status including cache statistics
    pub async fn get_comprehensive_health(&self) -> Value {
        let basic_health = self.get_basic_health().await;
        
        // TODO: Integrate with actual cache system when available in AppState
        // For now, return basic health with placeholder for cache stats
        let mut health = basic_health.as_object().unwrap().clone();
        
        // Add cache statistics placeholder
        health.insert("cache_stats".to_string(), json!({
            "l1_cache": {
                "entries": 0,
                "hits": 0,
                "misses": 0,
                "hit_rate": 0.0
            },
            "l2_cache": {
                "connected": false,
                "hits": 0,
                "misses": 0
            }
        }));

        json!(health)
    }

    /// Get system uptime in seconds
    pub fn get_uptime_seconds(&self) -> u64 {
        self.startup_time.elapsed().as_secs()
    }

    /// Get total request count
    pub fn get_request_count(&self) -> u64 {
        self.request_counter.load(Ordering::Relaxed)
    }

    /// Check if system is healthy based on basic criteria
    pub fn is_healthy(&self) -> bool {
        // Basic health check - system is healthy if it's been running
        // and responding to requests within reasonable time
        let last_check = self.last_health_check.read().unwrap();
        let time_since_last_check = last_check.elapsed().as_secs();
        
        // Consider healthy if last check was within 60 seconds
        time_since_last_check < 60
    }

    /// Get current system status
    pub fn get_system_status(&self) -> SystemStatus {
        if self.is_healthy() {
            SystemStatus::Healthy
        } else {
            SystemStatus::Degraded
        }
    }
}

impl Default for HealthMonitor {
    fn default() -> Self {
        Self::new()
    }
}
