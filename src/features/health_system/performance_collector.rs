//! Performance Collector - Performance metrics collection and analysis
//!
//! Collects and analyzes performance metrics including response times,
//! request rates, and system performance indicators.

use serde_json::{json, Value};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::time::Instant;

/// Performance metrics collection and tracking
pub struct PerformanceCollector {
    request_count: AtomicU64,
    total_response_time: AtomicU64,
    startup_time: Instant,
    active_connections: AtomicU64,
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,
    database_queries: AtomicU64,
    websocket_connections: AtomicU64,
}

impl PerformanceCollector {
    pub fn new() -> Self {
        Self {
            request_count: AtomicU64::new(0),
            total_response_time: AtomicU64::new(0),
            startup_time: Instant::now(),
            active_connections: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            database_queries: AtomicU64::new(0),
            websocket_connections: AtomicU64::new(0),
        }
    }

    /// Record a request with its response time
    pub fn record_request(&self, response_time_ms: u64) {
        self.request_count.fetch_add(1, Ordering::Relaxed);
        self.total_response_time.fetch_add(response_time_ms, Ordering::Relaxed);
    }

    /// Calculate average response time
    pub fn avg_response_time(&self) -> f64 {
        let total_requests = self.request_count.load(Ordering::Relaxed);
        let total_time = self.total_response_time.load(Ordering::Relaxed);
        
        if total_requests == 0 {
            0.0
        } else {
            total_time as f64 / total_requests as f64
        }
    }

    /// Calculate requests per second over system lifetime
    pub fn requests_per_second(&self) -> f64 {
        let uptime_secs = self.startup_time.elapsed().as_secs();
        let total_requests = self.request_count.load(Ordering::Relaxed);
        
        if uptime_secs == 0 {
            0.0
        } else {
            total_requests as f64 / uptime_secs as f64
        }
    }

    /// Calculate requests per second over specific duration
    pub fn requests_per_second_duration(&self, duration_secs: u64) -> f64 {
        let total_requests = self.request_count.load(Ordering::Relaxed);
        if duration_secs == 0 {
            0.0
        } else {
            total_requests as f64 / duration_secs as f64
        }
    }

    /// Record cache hit
    pub fn record_cache_hit(&self) {
        self.cache_hits.fetch_add(1, Ordering::Relaxed);
    }

    /// Record cache miss
    pub fn record_cache_miss(&self) {
        self.cache_misses.fetch_add(1, Ordering::Relaxed);
    }

    /// Record database query
    pub fn record_database_query(&self) {
        self.database_queries.fetch_add(1, Ordering::Relaxed);
    }

    /// Update active connections count
    pub fn set_active_connections(&self, count: u64) {
        self.active_connections.store(count, Ordering::Relaxed);
    }

    /// Update websocket connections count
    pub fn set_websocket_connections(&self, count: u64) {
        self.websocket_connections.store(count, Ordering::Relaxed);
    }

    /// Get comprehensive performance metrics
    pub async fn get_metrics(&self) -> Value {
        let uptime_secs = self.startup_time.elapsed().as_secs();
        let request_count = self.request_count.load(Ordering::Relaxed);
        let avg_response_time = self.avg_response_time();
        let rps = self.requests_per_second();
        let cpu_count = num_cpus::get();

        // Calculate cache hit rate
        let cache_hits = self.cache_hits.load(Ordering::Relaxed);
        let cache_misses = self.cache_misses.load(Ordering::Relaxed);
        let total_cache_requests = cache_hits + cache_misses;
        let cache_hit_rate = if total_cache_requests > 0 {
            cache_hits as f64 / total_cache_requests as f64
        } else {
            0.0
        };

        json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "uptime_seconds": uptime_secs,
            "cpu_count": cpu_count,
            "performance": {
                "total_requests": request_count,
                "avg_response_time_ms": avg_response_time,
                "requests_per_second": rps,
                "active_connections": self.active_connections.load(Ordering::Relaxed),
                "websocket_connections": self.websocket_connections.load(Ordering::Relaxed)
            },
            "cache_performance": {
                "hits": cache_hits,
                "misses": cache_misses,
                "hit_rate": cache_hit_rate,
                "total_requests": total_cache_requests
            },
            "database": {
                "total_queries": self.database_queries.load(Ordering::Relaxed)
            }
        })
    }

    /// Get basic performance summary
    pub fn get_summary(&self) -> Value {
        json!({
            "requests": self.request_count.load(Ordering::Relaxed),
            "avg_response_time_ms": self.avg_response_time(),
            "requests_per_second": self.requests_per_second(),
            "uptime_seconds": self.startup_time.elapsed().as_secs()
        })
    }

    /// Reset all metrics (useful for testing)
    pub fn reset(&self) {
        self.request_count.store(0, Ordering::Relaxed);
        self.total_response_time.store(0, Ordering::Relaxed);
        self.active_connections.store(0, Ordering::Relaxed);
        self.cache_hits.store(0, Ordering::Relaxed);
        self.cache_misses.store(0, Ordering::Relaxed);
        self.database_queries.store(0, Ordering::Relaxed);
        self.websocket_connections.store(0, Ordering::Relaxed);
    }
}

impl Default for PerformanceCollector {
    fn default() -> Self {
        Self::new()
    }
}
