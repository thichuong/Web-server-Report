//! Shared Utilities for Layer 5 Business Logic
//!
//! This module contains common utilities used across Layer 5 components:
//! - compression: Gzip compression for HTTP responses
//! - `response_builder`: Safe HTTP response construction
//! - error: Custom error types for Layer 5 operations
//! - websocket: WebSocket URL resolution utilities
//! - security: Cryptographically secure token generation
//! - `sitemap_creator`: Dynamic sitemap.xml generation

pub mod compression;
pub mod error;
pub mod response_builder;
pub mod rss_creator;
pub mod security;
pub mod sitemap_creator;
pub mod websocket;

// Re-export commonly used items
pub use compression::{compress_html_to_gzip, CompressionStats};
pub use error::{Layer5Error, Layer5Result};
pub use response_builder::{
    build_compressed_response, build_error_response, build_forbidden_response, build_html_response,
    build_not_found_response, build_sandboxed_response, build_shadow_dom_response,
};
pub use rss_creator::RssCreator;
pub use security::{generate_sandbox_token, verify_sandbox_token};
pub use sitemap_creator::SitemapCreator;
pub use websocket::get_websocket_url;
