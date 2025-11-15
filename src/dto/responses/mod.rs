//! Response DTOs for API endpoints

pub mod health;
pub mod cache;
pub mod dashboard;
pub mod websocket;

// Re-export all response types for convenience
pub use health::*;
pub use cache::*;
pub use dashboard::{DashboardDataResponse, StockIndexData};
pub use websocket::*;
