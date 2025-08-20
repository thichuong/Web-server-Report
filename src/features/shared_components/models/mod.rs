// src/features/shared_components/models/mod.rs
//
// Shared data models and structures used across multiple features
// These models represent common concepts that don't belong to any specific business domain

pub mod common;
pub mod response;
pub mod validation;

// Re-export commonly used types
pub use common::*;
pub use response::*;
pub use validation::*;
