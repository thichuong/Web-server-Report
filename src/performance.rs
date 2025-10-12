//! Performance Module - Placeholder Implementation
//! 
//! This module provides performance optimizations including an optimized HTTP client.
//! Originally this was part of the legacy architecture but we're creating a minimal
//! placeholder to allow Service Islands to compile and function.

use reqwest::Client;
use std::sync::LazyLock;
use std::time::Duration;

/// Optimized HTTP client with connection pooling and timeout configurations
/// This is used by various Service Islands components for external API calls
pub static OPTIMIZED_HTTP_CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(10))
        .pool_idle_timeout(Duration::from_secs(90))
        .pool_max_idle_per_host(10)
        .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build()
        .expect("Failed to create HTTP client")
});
