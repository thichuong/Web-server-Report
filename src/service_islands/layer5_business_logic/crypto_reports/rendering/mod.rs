//! Rendering Module
//!
//! This module contains rendering strategies for crypto reports:
//! - iframe_renderer: Legacy iframe-based rendering
//! - shadow_dom_renderer: Modern Declarative Shadow DOM rendering
//! - shared: Common utilities and models used by both strategies

pub mod shared;
pub mod iframe_renderer;
pub mod shadow_dom_renderer;

// Re-export commonly used items
pub use shared::{Report, SandboxedReport};
pub use iframe_renderer::IframeRenderer;
pub use shadow_dom_renderer::ShadowDomRenderer;
