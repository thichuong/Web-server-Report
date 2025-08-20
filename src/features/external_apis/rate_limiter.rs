// src/features/external_apis/rate_limiter.rs
//
// Rate limiting and circuit breaker implementation

use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use std::time::Duration;
use tokio::time;

/// Rate limiter with circuit breaker functionality
#[derive(Debug, Clone)]
pub struct RateLimiter {
    /// Last request timestamps per endpoint
    last_requests: Arc<tokio::sync::RwLock<HashMap<String, u64>>>,
    /// Circuit breakers per endpoint
    circuit_breakers: Arc<tokio::sync::RwLock<HashMap<String, Arc<AtomicBool>>>>,
    /// Rate limit intervals per endpoint (in seconds)
    intervals: Arc<tokio::sync::RwLock<HashMap<String, u64>>>,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            last_requests: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            circuit_breakers: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            intervals: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    /// Configure rate limiting interval for an endpoint
    pub async fn set_interval(&self, endpoint: &str, interval_seconds: u64) {
        let mut intervals = self.intervals.write().await;
        intervals.insert(endpoint.to_string(), interval_seconds);
    }

    /// Wait if rate limiting is needed for the endpoint
    pub async fn wait_if_needed(&self, endpoint: &str) -> Result<()> {
        // Check circuit breaker first
        if self.is_circuit_breaker_open(endpoint).await {
            return Err(anyhow!("Circuit breaker is open for endpoint: {}", endpoint));
        }

        let now = chrono::Utc::now().timestamp() as u64;
        
        // Get the rate limit interval for this endpoint
        let interval = {
            let intervals = self.intervals.read().await;
            intervals.get(endpoint).copied().unwrap_or(1) // Default 1 second
        };

        // Check if we need to wait
        let should_wait = {
            let mut last_requests = self.last_requests.write().await;
            if let Some(last_request) = last_requests.get(endpoint) {
                let elapsed = now - last_request;
                if elapsed < interval {
                    let wait_time = interval - elapsed;
                    println!("⏳ Rate limiting {} - waiting {}s", endpoint, wait_time);
                    
                    // Update the timestamp to reserve the slot
                    last_requests.insert(endpoint.to_string(), now + wait_time);
                    Some(wait_time)
                } else {
                    last_requests.insert(endpoint.to_string(), now);
                    None
                }
            } else {
                last_requests.insert(endpoint.to_string(), now);
                None
            }
        };

        // Wait if needed
        if let Some(wait_seconds) = should_wait {
            time::sleep(Duration::from_secs(wait_seconds)).await;
        }

        Ok(())
    }

    /// Open circuit breaker for an endpoint (e.g., due to rate limiting)
    pub async fn open_circuit_breaker(&self, endpoint: &str, duration_seconds: u64) {
        println!("🚨 Opening circuit breaker for {} for {}s", endpoint, duration_seconds);
        
        let circuit_breaker = {
            let mut circuit_breakers = self.circuit_breakers.write().await;
            circuit_breakers
                .entry(endpoint.to_string())
                .or_insert_with(|| Arc::new(AtomicBool::new(false)))
                .clone()
        };

        // Open the circuit breaker
        circuit_breaker.store(true, Ordering::Relaxed);

        // Schedule automatic reset
        let endpoint_name = endpoint.to_string();
        let circuit_breaker_clone = circuit_breaker.clone();
        tokio::spawn(async move {
            time::sleep(Duration::from_secs(duration_seconds)).await;
            circuit_breaker_clone.store(false, Ordering::Relaxed);
            println!("🔄 Circuit breaker reset for {}", endpoint_name);
        });
    }

    /// Check if circuit breaker is open for an endpoint
    pub async fn is_circuit_breaker_open(&self, endpoint: &str) -> bool {
        let circuit_breakers = self.circuit_breakers.read().await;
        if let Some(circuit_breaker) = circuit_breakers.get(endpoint) {
            circuit_breaker.load(Ordering::Relaxed)
        } else {
            false
        }
    }

    /// Get rate limiting status for monitoring
    pub async fn get_status(&self, endpoint: &str) -> RateLimitStatus {
        let now = chrono::Utc::now().timestamp() as u64;
        let circuit_breaker_open = self.is_circuit_breaker_open(endpoint).await;
        
        let (last_request, interval) = {
            let last_requests = self.last_requests.read().await;
            let intervals = self.intervals.read().await;
            
            let last_request = last_requests.get(endpoint).copied().unwrap_or(0);
            let interval = intervals.get(endpoint).copied().unwrap_or(1);
            
            (last_request, interval)
        };

        let seconds_since_last_request = if last_request > 0 { now - last_request } else { 0 };
        let can_make_request = !circuit_breaker_open && 
                               (last_request == 0 || seconds_since_last_request >= interval);

        RateLimitStatus {
            endpoint: endpoint.to_string(),
            circuit_breaker_open,
            seconds_since_last_request,
            interval_seconds: interval,
            can_make_request,
            next_available_in_seconds: if can_make_request { 0 } else { interval - seconds_since_last_request },
        }
    }
}

/// Rate limiting status information
#[derive(Debug, Clone)]
pub struct RateLimitStatus {
    pub endpoint: String,
    pub circuit_breaker_open: bool,
    pub seconds_since_last_request: u64,
    pub interval_seconds: u64,
    pub can_make_request: bool,
    pub next_available_in_seconds: u64,
}
