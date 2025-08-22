//! Layer 1: Infrastructure Services
//! 
//! This layer provides the foundational infrastructure services that all other layers depend on.
//! It includes shared components, caching systems, and core utilities.

pub mod shared_components_island;
pub mod cache_system_island;

// Re-export the main island components for easy access
pub use shared_components_island::SharedComponentsIsland;
pub use cache_system_island::CacheSystemIsland;
