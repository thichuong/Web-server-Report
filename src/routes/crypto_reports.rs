//! Crypto Reports Routes
//! 
//! This module defines all HTTP routes for crypto reports functionality including
//! report viewing and listing.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Router,
    body::Body
};
use std::sync::Arc;
use std::collections::HashMap;

use crate::service_islands::ServiceIslands;
use crate::service_islands::layer5_business_logic::crypto_reports::handlers::CryptoHandlers;

/// Configure crypto reports routes
pub fn configure_crypto_reports_routes() -> Router<Arc<ServiceIslands>> {
    Router::new()
        .route("/crypto_report", get(crypto_index))
        .route("/crypto_report/:id", get(crypto_view_report))
        .route("/crypto_reports_list", get(crypto_reports_list))
}

/// Crypto reports index page - Delegates to Crypto Reports Island with Tera engine
async fn crypto_index(
    State(service_islands): State<Arc<ServiceIslands>>,
    Query(params): Query<HashMap<String, String>>
) -> Response {
    // Check if specific report ID is requested (like ?id=54)
    let report_id = params.get("id");
    if let Some(id) = report_id {
        println!("🚀 [Route] crypto_index called for report ID: {} - fetching from Service Islands Layer 5", id);
    } else {
        println!("🚀 [Route] crypto_index called for latest report - fetching from Service Islands Layer 5");
    }
    


    // Get pre-loaded chart modules content for optimal performance
    let chart_modules_content = service_islands.get_chart_modules_content();

    // Use AppState with Tera engine from Service Islands - Full L1/L2 caching
    match service_islands.crypto_reports.handlers.crypto_index_with_tera(
        &service_islands.app_state,
        Some(chart_modules_content) // Truyền pre-loaded chart modules
    ).await {
        Ok(compressed_data) => {
            println!("✅ [Route] Compressed template rendered successfully from Layer 5");
                        
            // Use create_compressed_response for compressed data
            CryptoHandlers::create_compressed_response(compressed_data)
        }
        Err(e) => {
            eprintln!("❌ [Route] Crypto index error: {}", e);
            "Internal server error".into_response()
        }
    }
}

/// List all crypto reports with pagination
async fn crypto_reports_list(
    Query(params): Query<HashMap<String, String>>,
    State(service_islands): State<Arc<ServiceIslands>>
) -> impl IntoResponse {
    println!("🚀 [Route] crypto_reports_list called - fetching from Service Islands Layer 5");
    
    // Parse pagination parameter
    let page: i64 = params.get("page").and_then(|p| p.parse().ok()).unwrap_or(1);
    println!("📄 [Route] Requesting page: {}", page);

    // Use Service Islands architecture to get reports list
    match service_islands.crypto_reports.handlers.crypto_reports_list_with_tera(&service_islands.app_state, page).await {
        Ok(html) => {
            println!("✅ [Route] Reports list template rendered successfully from Layer 5");
            
            // Create cached response like archive_old_code with proper headers
            Response::builder()
                .status(StatusCode::OK)
                .header("cache-control", "public, max-age=60")
                .header("content-type", "text/html; charset=utf-8")
                .header("x-cache", "Layer5-Generated")
                .body(html)
                .unwrap()
                .into_response()
        }
        Err(e) => {
            eprintln!("❌ [Route] Failed to render reports list template from Layer 5: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR, 
                "Failed to load reports list"
            ).into_response()
        }
    }
}

/// View specific crypto report by ID
async fn crypto_view_report(
    Path(id): Path<String>,
    State(service_islands): State<Arc<ServiceIslands>>
) -> impl IntoResponse {
    println!("🚀 [Route] crypto_view_report called for ID: {} - fetching from Service Islands Layer 5", id);
    
    // Parse report ID
    let report_id: i32 = match id.parse() {
        Ok(id) => id,
        Err(_) => {
            println!("❌ [Route] Invalid report ID format: {}", id);
            return (
                StatusCode::BAD_REQUEST,
                "Invalid report ID format"
            ).into_response();
        }
    };

    println!("📄 [Route] Requesting report ID: {}", report_id);

    // First fetch real-time market data from Layer 5 → Layer 2 for enhanced template context
    // let realtime_data = match service_islands.crypto_reports.fetch_realtime_market_data().await {
    //     Ok(data) => {
    //         println!("✅ [Route] Got real-time data from Layer 5 for template context");
    //         Some(data)
    //     }
    //     Err(e) => {
    //         println!("⚠️ [Route] Failed to get real-time data: {}, using cached template only", e);
    //         None
    //     }
    // };

    // Get pre-loaded chart modules content for optimal performance
    let chart_modules_content = service_islands.get_chart_modules_content();

    // Use Service Islands architecture to get specific report
    match service_islands.crypto_reports.handlers.crypto_report_by_id_with_tera(
        &service_islands.app_state, 
        report_id, 
        Some(chart_modules_content) // Truyền pre-loaded chart modules
    ).await {
        Ok(compressed_data) => {
            println!("✅ [Route] Report ID: {} compressed template rendered successfully from Layer 5", report_id);            
            
            // Create compressed response with proper headers
            Response::builder()
                .status(StatusCode::OK)
                .header("cache-control", "public, max-age=300") // 5min cache for individual reports
                .header("content-type", "text/html; charset=utf-8")
                .header("content-encoding", "gzip")
                .header("x-cache", "Layer5-Generated-Compressed")
                .header("x-report-id", report_id.to_string())
                .body(Body::from(compressed_data))
                .unwrap()
                .into_response()
        }
        Err(e) => {
            eprintln!("❌ [Route] Failed to render report ID: {} template from Layer 5: {}", report_id, e);
            
            // Check if it's a not found case or other error
            let error_message = e.to_string();
            if error_message.contains("not found") || error_message.contains("Database error") {
                (
                    StatusCode::NOT_FOUND,
                    format!("Report #{} not found", report_id)
                ).into_response()
            } else {
                (
                    StatusCode::INTERNAL_SERVER_ERROR, 
                    format!("Failed to load report #{}", report_id)
                ).into_response()
            }
        }
    }
}
