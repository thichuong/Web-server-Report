//! Rendering Module
//!
//! This module contains rendering strategies for crypto reports:
//! - shadow_dom_renderer: Modern Declarative Shadow DOM rendering
//! - shared: Common utilities and models used by rendering strategies
//! - geo_metadata: GEO (Generative Engine Optimization) metadata for AI bots
//! - breadcrumbs: Breadcrumb navigation and related reports for GEO optimization

pub mod breadcrumbs;
pub mod geo_metadata;
pub mod shadow_dom_renderer;
pub mod shared;

// Re-export commonly used items
pub use breadcrumbs::{
    format_related_reports, generate_breadcrumb_items, generate_breadcrumbs_and_related,
    generate_breadcrumbs_schema, BreadcrumbItem, RelatedReportItem,
};
pub use geo_metadata::{
    generate_complete_geo_metadata, generate_json_ld, generate_meta_tags, GeoMetadata,
};
pub use shadow_dom_renderer::ShadowDomRenderer;
pub use shared::{Report, SandboxedReport};
