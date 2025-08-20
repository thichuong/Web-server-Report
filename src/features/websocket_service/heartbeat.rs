// src/features/websocket_service/heartbeat.rs
//
// Heartbeat management for WebSocket connections

use std::time::{Duration, Instant};
use tokio::time;

/// Heartbeat manager for WebSocket connections
#[derive(Debug)]
pub struct HeartbeatManager {
    interval_seconds: u64,
    last_heartbeat: std::sync::Arc<std::sync::Mutex<Instant>>,
}

impl HeartbeatManager {
    pub fn new(interval_seconds: u64) -> Self {
        Self {
            interval_seconds,
            last_heartbeat: std::sync::Arc::new(std::sync::Mutex::new(Instant::now())),
        }
    }

    /// Start heartbeat monitoring and return a handle for timeout checking
    pub fn start_heartbeat(&self) -> HeartbeatHandle {
        HeartbeatHandle::new(self.interval_seconds * 2, self.last_heartbeat.clone()) // 2x interval for timeout
    }

    /// Reset heartbeat timer (called when client responds)
    pub fn reset_heartbeat(&self) {
        if let Ok(mut last) = self.last_heartbeat.lock() {
            *last = Instant::now();
        }
    }

    /// Get heartbeat interval in seconds
    pub fn get_interval_seconds(&self) -> u64 {
        self.interval_seconds
    }
}

/// Handle for checking heartbeat timeouts
#[derive(Debug)]
pub struct HeartbeatHandle {
    timeout_duration: Duration,
    last_heartbeat: std::sync::Arc<std::sync::Mutex<Instant>>,
}

impl HeartbeatHandle {
    fn new(timeout_seconds: u64, last_heartbeat: std::sync::Arc<std::sync::Mutex<Instant>>) -> Self {
        Self {
            timeout_duration: Duration::from_secs(timeout_seconds),
            last_heartbeat,
        }
    }

    /// Wait for heartbeat timeout
    pub async fn timeout(&self) {
        loop {
            // Check if we've timed out
            let should_timeout = {
                if let Ok(last) = self.last_heartbeat.lock() {
                    last.elapsed() >= self.timeout_duration
                } else {
                    false
                }
            };

            if should_timeout {
                break;
            }

            // Sleep for a short interval before checking again
            time::sleep(Duration::from_secs(5)).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{timeout, Duration};

    #[tokio::test]
    async fn test_heartbeat_timeout() {
        let heartbeat_manager = HeartbeatManager::new(1); // 1 second interval
        let handle = heartbeat_manager.start_heartbeat();

        // Should timeout after ~2 seconds (2x interval)
        let result = timeout(Duration::from_secs(3), handle.timeout()).await;
        assert!(result.is_ok(), "Heartbeat should timeout within 3 seconds");
    }

    #[tokio::test] 
    async fn test_heartbeat_reset() {
        let heartbeat_manager = HeartbeatManager::new(1); // 1 second interval
        let handle = heartbeat_manager.start_heartbeat();

        // Reset heartbeat after 1 second
        time::sleep(Duration::from_millis(1100)).await;
        heartbeat_manager.reset_heartbeat();

        // Should not timeout for another ~2 seconds
        let result = timeout(Duration::from_millis(1500), handle.timeout()).await;
        assert!(result.is_err(), "Heartbeat should not timeout immediately after reset");
    }
}
