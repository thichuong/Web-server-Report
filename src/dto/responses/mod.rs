//! Response DTOs for API endpoints

pub mod cache;
pub mod dashboard;
pub mod health;
pub mod websocket;

// Re-export all response types for convenience
pub use cache::*;
pub use dashboard::{DashboardDataResponse, StockIndexData};
pub use health::*;
pub use websocket::*;
