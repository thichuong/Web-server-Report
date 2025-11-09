//! Layer 3: Communication
//!
//! This layer handles all communication-related functionality including:
//! - HTTP request/response handling
//! - Data communication with infrastructure layer
//! - Dashboard data communication and caching
//! - Redis stream reading for inter-service communication
//!
//! Note: WebSocket, Layer2 adapters, and external APIs have been moved to
//! the separate Web-server-Report-websocket service.

pub mod data_communication;
pub mod dashboard_communication;
pub mod redis_stream_reader;

