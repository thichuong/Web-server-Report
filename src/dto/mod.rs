//! Data Transfer Objects (DTOs) for API responses
//!
//! This module provides type-safe response structures for all API endpoints,
//! replacing ad-hoc `serde_json::Value` usage with proper Rust structs.

pub mod common;
pub mod responses;

// Re-export common types for convenience
pub use common::*;
