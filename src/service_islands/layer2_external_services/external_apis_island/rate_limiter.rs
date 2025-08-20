//! Rate Limiter Component
//! 
//! This component manages API rate limiting to prevent exceeding service limits.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::time::sleep;
use anyhow::Result;
use serde_json;

/// Rate limit configuration for different API endpoints
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub burst_size: u32,
    pub cooldown_seconds: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 4,   // Reduced from 10 to 4 requests per minute
            burst_size: 2,            // Reduced from 5 to 2 burst requests  
            cooldown_seconds: 90,     // Increased from 60 to 90 seconds cooldown
        }
    }
}

/// Rate limit tracking for a specific endpoint
#[derive(Debug)]
struct RateLimitTracker {
    last_reset: Instant,
    current_count: u32,
    config: RateLimitConfig,
    blocked_until: Option<Instant>,
}

impl RateLimitTracker {
    fn new(config: RateLimitConfig) -> Self {
        Self {
            last_reset: Instant::now(),
            current_count: 0,
            config,
            blocked_until: None,
        }
    }
    
    /// Check if we can make a request
    fn can_proceed(&mut self) -> bool {
        let now = Instant::now();
        
        // Check if we're in a cooldown period
        if let Some(blocked_until) = self.blocked_until {
            if now < blocked_until {
                return false;
            } else {
                self.blocked_until = None;
            }
        }
        
        // Reset counter if minute has passed
        if now.duration_since(self.last_reset) >= Duration::from_secs(60) {
            self.last_reset = now;
            self.current_count = 0;
        }
        
        // Check if we're within rate limit
        if self.current_count < self.config.requests_per_minute {
            self.current_count += 1;
            true
        } else {
            // Rate limit exceeded, enter cooldown
            self.blocked_until = Some(now + Duration::from_secs(self.config.cooldown_seconds));
            false
        }
    }
    
    /// Get time until next allowed request
    fn time_until_next_request(&self) -> Option<Duration> {
        if let Some(blocked_until) = self.blocked_until {
            let now = Instant::now();
            if now < blocked_until {
                Some(blocked_until - now)
            } else {
                None
            }
        } else {
            None
        }
    }
}

/// Rate Limiter
/// 
/// Manages rate limiting for multiple API endpoints to prevent service abuse.
pub struct RateLimiter {
    trackers: Arc<RwLock<HashMap<String, RateLimitTracker>>>,
    total_requests: Arc<AtomicU64>,
    total_blocked: Arc<AtomicU64>,
    start_time: Instant,
}

impl RateLimiter {
    /// Create a new RateLimiter
    pub fn new() -> Self {
        println!("⏱️ Initializing Rate Limiter...");
        
        let mut trackers = HashMap::new();
        
        // Configure rate limits for different endpoints
        trackers.insert(
            "btc".to_string(),
            RateLimitTracker::new(RateLimitConfig {
                requests_per_minute: 20, // BTC price can be fetched more frequently
                burst_size: 5,
                cooldown_seconds: 30,
            })
        );
        
        trackers.insert(
            "dashboard".to_string(),
            RateLimitTracker::new(RateLimitConfig {
                requests_per_minute: 10, // Dashboard data less frequent
                burst_size: 3,
                cooldown_seconds: 60,
            })
        );
        
        trackers.insert(
            "fear_greed".to_string(),
            RateLimitTracker::new(RateLimitConfig {
                requests_per_minute: 5, // Fear & Greed updates slowly
                burst_size: 2,
                cooldown_seconds: 120,
            })
        );
        
        trackers.insert(
            "rsi".to_string(),
            RateLimitTracker::new(RateLimitConfig {
                requests_per_minute: 8, // Technical indicators moderate frequency
                burst_size: 3,
                cooldown_seconds: 90,
            })
        );
        
        Self {
            trackers: Arc::new(RwLock::new(trackers)),
            total_requests: Arc::new(AtomicU64::new(0)),
            total_blocked: Arc::new(AtomicU64::new(0)),
            start_time: Instant::now(),
        }
    }
    
    /// Health check for Rate Limiter
    pub async fn health_check(&self) -> bool {
        // Rate limiter is healthy if it's been running and tracking requests
        let total_requests = self.total_requests.load(Ordering::Relaxed);
        let uptime = self.start_time.elapsed();
        
        println!("  ✅ Rate Limiter active for {:?}, {} requests processed", uptime, total_requests);
        true
    }
    
    /// Wait for rate limit to allow a request
    pub async fn wait_for_limit(&self, endpoint: &str) -> Result<()> {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        
        loop {
            // Check if we can proceed
            {
                let mut trackers = self.trackers.write().await;
                if let Some(tracker) = trackers.get_mut(endpoint) {
                    if tracker.can_proceed() {
                        println!("✅ Rate limit OK for endpoint: {}", endpoint);
                        return Ok(());
                    }
                    
                    // Get wait time
                    if let Some(wait_duration) = tracker.time_until_next_request() {
                        println!("⏳ Rate limit exceeded for {}, waiting {:?}", endpoint, wait_duration);
                        self.total_blocked.fetch_add(1, Ordering::Relaxed);
                        
                        // Release the lock before sleeping
                        drop(trackers);
                        
                        // Wait with a maximum of 10 seconds to avoid long blocks
                        let wait_time = std::cmp::min(wait_duration, Duration::from_secs(10));
                        sleep(wait_time).await;
                        continue;
                    }
                } else {
                    // Endpoint not configured, add default rate limit
                    trackers.insert(
                        endpoint.to_string(),
                        RateLimitTracker::new(RateLimitConfig::default())
                    );
                    continue;
                }
            }
            
            // If no specific wait time, just wait a short period
            sleep(Duration::from_millis(100)).await;
        }
    }
    
    /// Check if request is allowed without waiting
    pub async fn is_allowed(&self, endpoint: &str) -> bool {
        let mut trackers = self.trackers.write().await;
        if let Some(tracker) = trackers.get_mut(endpoint) {
            tracker.can_proceed()
        } else {
            // Endpoint not configured, allow by default but add tracking
            trackers.insert(
                endpoint.to_string(),
                RateLimitTracker::new(RateLimitConfig::default())
            );
            true
        }
    }
    
    /// Get rate limit status
    pub async fn get_status(&self) -> serde_json::Value {
        let trackers = self.trackers.read().await;
        let mut endpoint_status = serde_json::Map::new();
        
        for (endpoint, tracker) in trackers.iter() {
            let status = serde_json::json!({
                "requests_per_minute": tracker.config.requests_per_minute,
                "current_count": tracker.current_count,
                "blocked": tracker.blocked_until.is_some(),
                "time_until_reset": if let Some(wait_time) = tracker.time_until_next_request() {
                    wait_time.as_secs()
                } else {
                    0
                }
            });
            endpoint_status.insert(endpoint.clone(), status);
        }
        
        serde_json::json!({
            "endpoints": endpoint_status,
            "total_requests": self.total_requests.load(Ordering::Relaxed),
            "total_blocked": self.total_blocked.load(Ordering::Relaxed),
            "uptime_seconds": self.start_time.elapsed().as_secs()
        })
    }
    
    /// Get rate limiter statistics
    pub async fn get_statistics(&self) -> serde_json::Value {
        let total_requests = self.total_requests.load(Ordering::Relaxed);
        let total_blocked = self.total_blocked.load(Ordering::Relaxed);
        let uptime = self.start_time.elapsed();
        
        let block_rate = if total_requests > 0 {
            (total_blocked as f64 / total_requests as f64) * 100.0
        } else {
            0.0
        };
        
        let requests_per_second = if uptime.as_secs() > 0 {
            total_requests as f64 / uptime.as_secs() as f64
        } else {
            0.0
        };
        
        let endpoints_configured = {
            let trackers = self.trackers.read().await;
            trackers.len()
        };
        
        serde_json::json!({
            "total_requests": total_requests,
            "total_blocked": total_blocked,
            "block_rate_percent": block_rate,
            "uptime_seconds": uptime.as_secs(),
            "requests_per_second": requests_per_second,
            "endpoints_configured": endpoints_configured
        })
    }
    
    /// Add or update rate limit configuration for an endpoint
    pub async fn configure_endpoint(&self, endpoint: &str, config: RateLimitConfig) {
        let mut trackers = self.trackers.write().await;
        trackers.insert(endpoint.to_string(), RateLimitTracker::new(config));
        println!("⚙️ Rate limit configured for endpoint: {}", endpoint);
    }
    
    /// Remove rate limiting for an endpoint
    pub async fn remove_endpoint(&self, endpoint: &str) {
        let mut trackers = self.trackers.write().await;
        trackers.remove(endpoint);
        println!("🗑️ Rate limit removed for endpoint: {}", endpoint);
    }
}
