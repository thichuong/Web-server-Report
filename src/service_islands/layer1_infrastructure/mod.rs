//! Layer 1: Infrastructure Services
//!
//! This layer provides the foundational infrastructure services that all other layers depend on.
//! It includes application state, shared components, caching systems, chart modules, and core utilities.

pub mod app_state_island;
pub mod cache_system_island;
pub mod chart_modules_island;
pub mod shared_components_island;

// Re-export the main island components for easy access
pub use app_state_island::{AppState, AppStateIsland};
pub use cache_system_island::CacheSystemIsland;
pub use chart_modules_island::ChartModulesIsland;
pub use shared_components_island::SharedComponentsIsland;
