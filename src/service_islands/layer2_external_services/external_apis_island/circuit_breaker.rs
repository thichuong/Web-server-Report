//! Circuit Breaker Component
//! 
//! This component implements the circuit breaker pattern to handle failing external services gracefully.

use std::collections::HashMap;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

/// Circuit breaker states
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum CircuitState {
    Closed,     // Normal operation
    Open,       // Circuit is open, requests are blocked
    HalfOpen,   // Testing if service has recovered
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
#[allow(dead_code)]
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
#[allow(dead_code)]
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
}

/// Circuit Breaker
/// 
/// Implements the circuit breaker pattern to handle failing external services gracefully.
#[allow(dead_code)]
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
}
