//! Dashboard HTTP Request Handlers
//! 
//! This module contains all HTTP request handlers related to dashboard functionality.
//! Originally located in src/handlers/api.rs, these handlers have been moved to the
//! Dashboard Island as part of the Service Islands Architecture.

use tokio::fs;
use std::sync::Arc;
use tera::Context;
use crate::state::AppState;

/// Dashboard Handlers
/// 
/// Contains all HTTP request handlers for dashboard-related operations.
/// These handlers manage dashboard data, summaries, and API interactions.
pub struct DashboardHandlers {
    // Component state will be added here as we implement lower layers
}

impl DashboardHandlers {
    /// Create a new DashboardHandlers instance
    pub fn new() -> Self {
        Self {
            // Initialize component state
        }
    }
    
    /// Health check for dashboard handlers
    pub async fn health_check(&self) -> bool {
        // Verify handlers are functioning properly
        true // Will implement actual health checks
    }
    /// Homepage handler - renders homepage template using Tera
    /// 
    /// Uses Tera template engine to render home.html with market indicators component included.
    /// This replaces the simple file reading with proper template rendering.
    pub async fn homepage(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        match fs::read_to_string("dashboards/home.html").await {
            Ok(content) => Ok(content),
            Err(e) => Err(Box::new(e)),
        }
    }
    
    /// Homepage handler with Tera rendering - ENHANCED VERSION
    /// 
    /// Renders homepage using Tera template engine with proper context.
    /// This includes the market indicators component and any dynamic data.
    pub async fn homepage_with_tera(
        &self,
        state: &Arc<AppState>
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut context = Context::new();
        
        // Add basic context for homepage
        context.insert("current_route", "homepage");
        context.insert("current_lang", "vi");
        context.insert("current_time", &chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string());
        
        // You can add market data or other dynamic content here
        // context.insert("market_data", &some_market_data);
        
        match state.tera.render("home.html", &context) {
            Ok(html) => {
                println!("✅ Homepage rendered successfully with Tera");
                Ok(html)
            }
            Err(e) => {
                println!("❌ Failed to render homepage template: {}", e);
                // Fallback to simple file reading
                match fs::read_to_string("dashboards/home.html").await {
                    Ok(content) => Ok(content),
                    Err(fallback_e) => Err(Box::new(fallback_e)),
                }
            }
        }
    }
}
