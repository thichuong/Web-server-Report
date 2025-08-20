//! Circuit Breaker Component
//! 
//! This component implements the circuit breaker pattern to handle failing external services gracefully.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use anyhow::Result;

/// Circuit breaker states
#[derive(Debug, Clone, PartialEq)]
pub enum CircuitState {
    Closed,     // Normal operation
    Open,       // Circuit is open, requests are blocked
    HalfOpen,   // Testing if service has recovered
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: usize,      // Number of failures to open circuit
    pub success_threshold: usize,      // Number of successes to close circuit
    pub timeout_seconds: u64,          // Time to wait before trying half-open
    pub reset_timeout_seconds: u64,    // Time to fully reset after recovery
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,      // Open after 5 failures
            success_threshold: 3,      // Close after 3 successes
            timeout_seconds: 60,       // Wait 1 minute before testing
            reset_timeout_seconds: 300, // Reset completely after 5 minutes
        }
    }
}

/// Circuit breaker tracker for a specific service
#[derive(Debug)]
struct CircuitBreakerTracker {
    state: CircuitState,
    config: CircuitBreakerConfig,
    failure_count: usize,
    success_count: usize,
    last_failure_time: Option<Instant>,
    last_success_time: Option<Instant>,
    state_change_time: Instant,
    total_requests: usize,
    total_failures: usize,
}

impl CircuitBreakerTracker {
    fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            state: CircuitState::Closed,
            config,
            failure_count: 0,
            success_count: 0,
            last_failure_time: None,
            last_success_time: None,
            state_change_time: Instant::now(),
            total_requests: 0,
            total_failures: 0,
        }
    }
    
    /// Check if requests are allowed
    fn can_proceed(&mut self) -> bool {
        let now = Instant::now();
        
        match self.state {
            CircuitState::Closed => true, // Always allow requests when closed
            CircuitState::Open => {
                // Check if timeout has passed to try half-open
                if let Some(last_failure) = self.last_failure_time {
                    if now.duration_since(last_failure) >= Duration::from_secs(self.config.timeout_seconds) {
                        self.state = CircuitState::HalfOpen;
                        self.state_change_time = now;
                        self.success_count = 0;
                        println!("🔄 Circuit breaker half-open, testing service recovery");
                        true
                    } else {
                        false // Still in timeout period
                    }
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => true, // Allow limited requests to test recovery
        }
    }
    
    /// Record a successful request
    fn record_success(&mut self) {
        self.total_requests += 1;
        self.last_success_time = Some(Instant::now());
        
        match self.state {
            CircuitState::Closed => {
                self.failure_count = 0; // Reset failure count on success
            }
            CircuitState::HalfOpen => {
                self.success_count += 1;
                if self.success_count >= self.config.success_threshold {
                    self.state = CircuitState::Closed;
                    self.state_change_time = Instant::now();
                    self.failure_count = 0;
                    self.success_count = 0;
                    println!("✅ Circuit breaker closed, service recovered");
                }
            }
            CircuitState::Open => {
                // Shouldn't happen, but reset if it does
                self.state = CircuitState::Closed;
                self.state_change_time = Instant::now();
                self.failure_count = 0;
                self.success_count = 0;
            }
        }
    }
    
    /// Record a failed request
    fn record_failure(&mut self) {
        self.total_requests += 1;
        self.total_failures += 1;
        self.last_failure_time = Some(Instant::now());
        
        match self.state {
            CircuitState::Closed => {
                self.failure_count += 1;
                if self.failure_count >= self.config.failure_threshold {
                    self.state = CircuitState::Open;
                    self.state_change_time = Instant::now();
                    println!("🚨 Circuit breaker opened due to {} failures", self.failure_count);
                }
            }
            CircuitState::HalfOpen => {
                // Failed during testing, go back to open
                self.state = CircuitState::Open;
                self.state_change_time = Instant::now();
                self.failure_count += 1;
                self.success_count = 0;
                println!("⚠️ Circuit breaker re-opened, service still failing");
            }
            CircuitState::Open => {
                self.failure_count += 1; // Continue counting failures
            }
        }
    }
    
    /// Get current status
    fn get_status(&self) -> serde_json::Value {
        let failure_rate = if self.total_requests > 0 {
            (self.total_failures as f64 / self.total_requests as f64) * 100.0
        } else {
            0.0
        };
        
        serde_json::json!({
            "state": match self.state {
                CircuitState::Closed => "closed",
                CircuitState::Open => "open", 
                CircuitState::HalfOpen => "half_open"
            },
            "failure_count": self.failure_count,
            "success_count": self.success_count,
            "total_requests": self.total_requests,
            "total_failures": self.total_failures,
            "failure_rate_percent": failure_rate,
            "time_in_current_state": self.state_change_time.elapsed().as_secs(),
            "last_failure": self.last_failure_time.map(|t| t.elapsed().as_secs()),
            "last_success": self.last_success_time.map(|t| t.elapsed().as_secs())
        })
    }
}

/// Circuit Breaker
/// 
/// Implements the circuit breaker pattern to handle failing external services gracefully.
pub struct CircuitBreaker {
    breakers: Arc<RwLock<HashMap<String, CircuitBreakerTracker>>>,
    total_blocked: Arc<AtomicU64>,
    total_opened: Arc<AtomicU64>,
    start_time: Instant,
}

impl CircuitBreaker {
    /// Create a new CircuitBreaker
    pub fn new() -> Self {
        println!("⚡ Initializing Circuit Breaker...");
        
        let mut breakers = HashMap::new();
        
        // Configure circuit breakers for different services
        breakers.insert(
            "global".to_string(),
            CircuitBreakerTracker::new(CircuitBreakerConfig {
                failure_threshold: 5,
                success_threshold: 3,
                timeout_seconds: 60,
                reset_timeout_seconds: 300,
            })
        );
        
        breakers.insert(
            "btc".to_string(),
            CircuitBreakerTracker::new(CircuitBreakerConfig {
                failure_threshold: 3,     // More sensitive for BTC (critical data)
                success_threshold: 2,     // Faster recovery
                timeout_seconds: 30,      // Shorter timeout
                reset_timeout_seconds: 180,
            })
        );
        
        breakers.insert(
            "fear_greed".to_string(),
            CircuitBreakerTracker::new(CircuitBreakerConfig {
                failure_threshold: 8,     // Less sensitive (non-critical)
                success_threshold: 4,     
                timeout_seconds: 120,     // Longer timeout
                reset_timeout_seconds: 600,
            })
        );
        
        breakers.insert(
            "rsi".to_string(),
            CircuitBreakerTracker::new(CircuitBreakerConfig {
                failure_threshold: 6,
                success_threshold: 3,
                timeout_seconds: 90,
                reset_timeout_seconds: 300,
            })
        );
        
        Self {
            breakers: Arc::new(RwLock::new(breakers)),
            total_blocked: Arc::new(AtomicU64::new(0)),
            total_opened: Arc::new(AtomicU64::new(0)),
            start_time: Instant::now(),
        }
    }
    
    /// Health check for Circuit Breaker
    pub async fn health_check(&self) -> bool {
        let breakers = self.breakers.read().await;
        let total_services = breakers.len();
        let open_breakers = breakers.values()
            .filter(|b| b.state == CircuitState::Open)
            .count();
        
        if open_breakers == 0 {
            println!("  ✅ Circuit Breaker: All {} services healthy", total_services);
            true
        } else {
            println!("  ⚠️ Circuit Breaker: {}/{} services have open circuits", open_breakers, total_services);
            true // Circuit breaker itself is healthy, even if some services are not
        }
    }
    
    /// Check if requests are allowed for a service
    pub async fn can_proceed(&self, service: &str) -> bool {
        let mut breakers = self.breakers.write().await;
        if let Some(breaker) = breakers.get_mut(service) {
            let allowed = breaker.can_proceed();
            if !allowed {
                self.total_blocked.fetch_add(1, Ordering::Relaxed);
            }
            allowed
        } else {
            // Service not configured, add default circuit breaker and allow
            breakers.insert(
                service.to_string(),
                CircuitBreakerTracker::new(CircuitBreakerConfig::default())
            );
            true
        }
    }
    
    /// Record successful request
    pub async fn record_success(&self, service: &str) {
        let mut breakers = self.breakers.write().await;
        if let Some(breaker) = breakers.get_mut(service) {
            let previous_state = breaker.state.clone();
            breaker.record_success();
            
            // Track state changes
            if previous_state == CircuitState::Open && breaker.state == CircuitState::Closed {
                println!("✅ Circuit breaker for '{}' closed after successful recovery", service);
            }
        }
    }
    
    /// Record failed request
    pub async fn record_failure(&self, service: &str) {
        let mut breakers = self.breakers.write().await;
        if let Some(breaker) = breakers.get_mut(service) {
            let previous_state = breaker.state.clone();
            breaker.record_failure();
            
            // Track state changes
            if previous_state != CircuitState::Open && breaker.state == CircuitState::Open {
                self.total_opened.fetch_add(1, Ordering::Relaxed);
                println!("🚨 Circuit breaker for '{}' opened due to failures", service);
            }
        }
    }
    
    /// Get circuit breaker status
    pub async fn get_status(&self) -> serde_json::Value {
        let breakers = self.breakers.read().await;
        let mut service_status = serde_json::Map::new();
        
        for (service, breaker) in breakers.iter() {
            service_status.insert(service.clone(), breaker.get_status());
        }
        
        serde_json::json!({
            "services": service_status,
            "total_blocked": self.total_blocked.load(Ordering::Relaxed),
            "total_opened": self.total_opened.load(Ordering::Relaxed),
            "uptime_seconds": self.start_time.elapsed().as_secs()
        })
    }
    
    /// Get circuit breaker statistics
    pub async fn get_statistics(&self) -> serde_json::Value {
        let breakers = self.breakers.read().await;
        let total_services = breakers.len();
        let open_count = breakers.values().filter(|b| b.state == CircuitState::Open).count();
        let half_open_count = breakers.values().filter(|b| b.state == CircuitState::HalfOpen).count();
        let closed_count = breakers.values().filter(|b| b.state == CircuitState::Closed).count();
        
        let total_requests: usize = breakers.values().map(|b| b.total_requests).sum();
        let total_failures: usize = breakers.values().map(|b| b.total_failures).sum();
        
        let overall_failure_rate = if total_requests > 0 {
            (total_failures as f64 / total_requests as f64) * 100.0
        } else {
            0.0
        };
        
        serde_json::json!({
            "total_services": total_services,
            "services_open": open_count,
            "services_half_open": half_open_count,
            "services_closed": closed_count,
            "total_requests": total_requests,
            "total_failures": total_failures,
            "overall_failure_rate_percent": overall_failure_rate,
            "total_circuit_opens": self.total_opened.load(Ordering::Relaxed),
            "total_blocked_requests": self.total_blocked.load(Ordering::Relaxed),
            "uptime_seconds": self.start_time.elapsed().as_secs()
        })
    }
    
    /// Configure circuit breaker for a service
    pub async fn configure_service(&self, service: &str, config: CircuitBreakerConfig) {
        let mut breakers = self.breakers.write().await;
        breakers.insert(service.to_string(), CircuitBreakerTracker::new(config));
        println!("⚙️ Circuit breaker configured for service: {}", service);
    }
    
    /// Force open a circuit breaker (for testing or maintenance)
    pub async fn force_open(&self, service: &str) -> Result<()> {
        let mut breakers = self.breakers.write().await;
        if let Some(breaker) = breakers.get_mut(service) {
            breaker.state = CircuitState::Open;
            breaker.state_change_time = Instant::now();
            self.total_opened.fetch_add(1, Ordering::Relaxed);
            println!("🔧 Circuit breaker for '{}' manually opened", service);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Service '{}' not found", service))
        }
    }
    
    /// Force close a circuit breaker (for testing or recovery)
    pub async fn force_close(&self, service: &str) -> Result<()> {
        let mut breakers = self.breakers.write().await;
        if let Some(breaker) = breakers.get_mut(service) {
            breaker.state = CircuitState::Closed;
            breaker.state_change_time = Instant::now();
            breaker.failure_count = 0;
            breaker.success_count = 0;
            println!("🔧 Circuit breaker for '{}' manually closed", service);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Service '{}' not found", service))
        }
    }
}
