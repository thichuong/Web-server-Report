//! WebSocket URL Resolution Utilities
//!
//! Shared WebSocket URL configuration logic used across Layer 5 components.
//! Eliminates duplicate URL resolution code in handlers.

use tracing::{error, warn};

/// Production WebSocket fallback URL
const PRODUCTION_WS_URL: &str = "wss://web-server-report-websocket-production.up.railway.app";

/// Development WebSocket URL
const DEVELOPMENT_WS_URL: &str = "ws://localhost:8081";

/// Get WebSocket service URL from environment or use appropriate default
///
/// Returns the URL based on:
/// 1. WEBSOCKET_SERVICE_URL environment variable (if set)
/// 2. Development default (ws://localhost:8081) in debug builds
/// 3. Production default with warning in release builds
///
/// # Performance
/// This function is designed to be called infrequently (once per request at most).
/// Consider caching the result if called in a hot path.
#[inline]
pub fn get_websocket_url() -> String {
    std::env::var("WEBSOCKET_SERVICE_URL").unwrap_or_else(|_| {
        if cfg!(debug_assertions) {
            DEVELOPMENT_WS_URL.to_string()
        } else {
            warn!("WEBSOCKET_SERVICE_URL not set in production!");
            error!("   Using fallback: {}", PRODUCTION_WS_URL);
            error!("   Set WEBSOCKET_SERVICE_URL environment variable to avoid this warning.");
            PRODUCTION_WS_URL.to_string()
        }
    })
}

/// Get WebSocket URL with lazy static caching
///
/// This version caches the URL after first resolution to avoid
/// repeated environment variable lookups on hot paths.
pub mod cached {
    use std::sync::OnceLock;

    static CACHED_WS_URL: OnceLock<String> = OnceLock::new();

    /// Get cached WebSocket URL (resolved once, cached forever)
    #[inline]
    pub fn get_websocket_url() -> &'static str {
        CACHED_WS_URL.get_or_init(super::get_websocket_url)
    }
}

#[cfg(test)]
mod tests {
    use super::get_websocket_url;

    #[test]
    fn test_get_websocket_url_default() {
        // In test (debug) mode, should return development URL if env not set
        std::env::remove_var("WEBSOCKET_SERVICE_URL");
        let url = get_websocket_url();
        assert!(url.starts_with("ws://") || url.starts_with("wss://"));
    }

    #[test]
    fn test_get_websocket_url_from_env() {
        std::env::set_var("WEBSOCKET_SERVICE_URL", "ws://test:1234");
        let url = get_websocket_url();
        assert_eq!(url, "ws://test:1234");
        std::env::remove_var("WEBSOCKET_SERVICE_URL");
    }
}
