//! UI Components - Dashboard user interface management
//!
//! Handles UI rendering, navigation, and interactive components
//! for the dashboard system.

use crate::features::shared_components::SharedComponents;
use axum::{
    extract::{Path, Query, State},
    response::{Html, IntoResponse, Response},
    http::StatusCode,
};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;

/// UI management component for dashboard interfaces
pub struct UiComponents {
    shared_components: Option<Arc<SharedComponents>>,
}

impl UiComponents {
    pub fn new(shared_components: &SharedComponents) -> Self {
        Self {
            shared_components: Some(Arc::new(shared_components.clone())),
        }
    }

    /// Render navigation components
    pub fn render_navigation(&self, current_route: &str) -> serde_json::Value {
        json!({
            "current_route": current_route,
            "nav_items": [
                {
                    "id": "dashboard",
                    "label": "Dashboard", 
                    "url": "/crypto",
                    "active": current_route == "dashboard"
                },
                {
                    "id": "reports",
                    "label": "Báo cáo",
                    "url": "/crypto/reports",
                    "active": current_route == "reports"
                },
                {
                    "id": "analytics", 
                    "label": "Phân tích",
                    "url": "/crypto/analytics",
                    "active": current_route == "analytics"
                }
            ]
        })
    }

    /// Create UI context for templates
    pub fn create_ui_context(&self, route: &str, additional_data: Option<serde_json::Value>) -> serde_json::Value {
        let mut context = json!({
            "current_route": route,
            "current_lang": "vi",
            "current_time": chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
            "navigation": self.render_navigation(route)
        });

        // Merge additional data if provided
        if let Some(data) = additional_data {
            if let (Some(context_obj), Some(data_obj)) = (context.as_object_mut(), data.as_object()) {
                for (key, value) in data_obj {
                    context_obj.insert(key.clone(), value.clone());
                }
            }
        }

        context
    }

    /// Create response with caching headers
    pub fn create_cached_response(&self, html: String, cache_type: &str) -> Response {
        let cache_header = match cache_type {
            "hit" => "public, max-age=300", // 5 minutes for cache hits
            "miss" => "public, max-age=60",  // 1 minute for cache misses
            "empty" => "no-cache",           // No cache for empty responses
            _ => "no-cache"
        };

        axum::response::Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "text/html; charset=utf-8")
            .header("cache-control", cache_header)
            .header("x-cache-status", cache_type)
            .body(html)
            .unwrap()
            .into_response()
    }

    /// Render chart modules for dashboard
    pub async fn serve_chart_modules(&self) -> impl IntoResponse {
        // Default chart modules content
        let content = r#"
// Chart modules for dashboard
console.log('Chart modules loaded');

// Initialize chart components
function initializeCharts() {
    // Chart initialization logic
}

// Export chart functions
window.chartModules = {
    initialize: initializeCharts
};
"#;

        axum::response::Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "application/javascript")
            .header("cache-control", "public, max-age=3600")
            .body(content.to_string())
            .unwrap()
    }

    /// Handle pagination logic
    pub fn build_pagination(&self, page: i64, total: i64, per_page: i64) -> serde_json::Value {
        let pages = if total == 0 { 1 } else { ((total as f64) / (per_page as f64)).ceil() as i64 };
        let offset = (page - 1) * per_page;
        
        // Build page numbers with ellipses
        let page_numbers = self.build_page_numbers(page, pages);
        
        let display_start = if total == 0 { 0 } else { offset + 1 };
        let display_end = offset + std::cmp::min(per_page, total - offset);

        json!({
            "total": total,
            "per_page": per_page,
            "page": page,
            "pages": pages,
            "has_prev": page > 1,
            "has_next": page < pages,
            "prev_num": if page > 1 { page - 1 } else { 1 },
            "next_num": if page < pages { page + 1 } else { pages },
            "page_numbers": page_numbers,
            "display_start": display_start,
            "display_end": display_end,
        })
    }

    /// Build page numbers with ellipses for pagination
    fn build_page_numbers(&self, current_page: i64, total_pages: i64) -> Vec<Option<i64>> {
        let mut page_numbers: Vec<Option<i64>> = Vec::new();
        
        if total_pages <= 10 {
            // Show all pages if <= 10
            for p in 1..=total_pages { 
                page_numbers.push(Some(p)); 
            }
        } else {
            // Show first 1-2, last 1-2, and current +/-2 with ellipses
            let mut added = std::collections::HashSet::new();
            let mut push = |vec: &mut Vec<Option<i64>>, v: i64, added: &mut std::collections::HashSet<i64>| {
                if !added.contains(&v) && v > 0 && v <= total_pages {
                    vec.push(Some(v));
                    added.insert(v);
                }
            };
            
            push(&mut page_numbers, 1, &mut added);
            push(&mut page_numbers, 2, &mut added);
            
            for v in (current_page-2)..=(current_page+2) { 
                if v > 2 && v < total_pages-1 { 
                    push(&mut page_numbers, v, &mut added); 
                } 
            }
            
            push(&mut page_numbers, total_pages-1, &mut added);
            push(&mut page_numbers, total_pages, &mut added);

            // Sort and insert None where gaps > 1
            let mut nums: Vec<i64> = page_numbers.iter().filter_map(|o| *o).collect();
            nums.sort();
            page_numbers.clear();
            
            let mut last: Option<i64> = None;
            for n in nums {
                if let Some(l) = last {
                    if n - l > 1 {
                        page_numbers.push(None); // Represents ellipsis
                    }
                }
                page_numbers.push(Some(n));
                last = Some(n);
            }
        }
        
        page_numbers
    }

    /// Format date for display
    pub fn format_display_date(&self, datetime: chrono::DateTime<chrono::Utc>) -> (String, String) {
        let dt = datetime + chrono::Duration::hours(7); // Convert to UTC+7
        let created_date = dt.format("%d/%m/%Y").to_string();
        let created_time = format!("{} UTC+7", dt.format("%H:%M:%S"));
        (created_date, created_time)
    }

    /// Create error response
    pub fn create_error_response(&self, status: StatusCode, message: &str) -> Response {
        (status, message).into_response()
    }

    /// Create HTML response
    pub fn create_html_response(&self, html: String) -> Response {
        Html(html).into_response()
    }

    /// Get chart modules content (placeholder)
    pub async fn get_chart_modules_content(&self) -> String {
        // This would typically fetch from shared components
        String::from(r#"
// Default chart modules content
console.log('Dashboard chart modules initialized');

// Chart configuration
const chartConfig = {
    responsive: true,
    maintainAspectRatio: false,
    animation: {
        duration: 800,
        easing: 'easeInOutQuart'
    }
};

// Export chart utilities
window.dashboardCharts = {
    config: chartConfig,
    init: function() {
        console.log('Dashboard charts ready');
    }
};
"#)
    }
}

impl Default for UiComponents {
    fn default() -> Self {
        Self {
            shared_components: None,
        }
    }
}
