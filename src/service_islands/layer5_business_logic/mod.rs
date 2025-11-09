//! Layer 5: Business Logic Islands
//!
//! This layer contains the core business logic islands that handle
//! domain-specific operations for the application.
//!
//! Note: Market data fetching has been moved to the Web-server-Report-websocket service.
//! This service now consumes data from Redis cache/streams.

pub mod dashboard;
pub mod crypto_reports;

