//! Layer 5: Business Logic Islands
//! 
//! This layer contains the core business logic islands that handle
//! domain-specific operations for the application.

pub mod dashboard;
pub mod crypto_reports;

pub use dashboard::DashboardIsland;
pub use crypto_reports::CryptoReportsIsland;
