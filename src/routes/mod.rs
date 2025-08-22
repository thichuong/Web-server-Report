//! Routes Module
//! 
//! This module organizes all route handlers for the Service Islands Architecture.
//! Routes are split into logical modules for better maintainability and organization.

pub mod static_files;
pub mod legacy;
pub mod homepage;
pub mod system;
pub mod dashboard;
pub mod crypto_reports;
pub mod api;
pub mod websocket;

use axum::Router;
use std::sync::Arc;

use crate::service_islands::ServiceIslands;

/// Create the main Service Islands router by combining all route modules
pub fn create_service_islands_router(service_islands: Arc<ServiceIslands>) -> Router {
    Router::new()
        // Static file serving
        .merge(static_files::configure_static_routes(service_islands.clone()))
        
        // Legacy compatibility routes
        .merge(legacy::configure_legacy_routes())
        
        // Homepage
        .merge(homepage::configure_homepage_route())
        
        // Health and system endpoints
        .merge(system::configure_system_routes())
        
        // Dashboard routes
        .merge(dashboard::configure_dashboard_routes())
        
        // Crypto Reports routes
        .merge(crypto_reports::configure_crypto_reports_routes())
        
        // API endpoints
        .merge(api::configure_api_routes())
        
        // WebSocket endpoint
        .merge(websocket::configure_websocket_routes())
        
        .with_state(service_islands)
}
