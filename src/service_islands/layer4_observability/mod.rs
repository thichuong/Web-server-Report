//! Layer 4: Observability Islands
//! 
//! This layer contains islands responsible for system monitoring,
//! health checking, performance tracking, and observability.

pub mod health_system;

pub use health_system::HealthSystemIsland;
