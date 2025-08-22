//! Crypto Reports Routes
//! 
//! This module handles all cryptocurrency report-related routes including
//! report viewing, listing, and PDF template generation.

use axum::{
    routing::get,
    Router,
    response::{Html, IntoResponse, Response},
    extract::{Extension, Path, State, Query},
    http::StatusCode,
    Json
};
use std::sync::Arc;
use std::collections::HashMap;

use crate::service_islands::ServiceIslands;

/// Configure crypto reports routes
pub fn configure_crypto_reports_routes() -> Router<Arc<ServiceIslands>> {
    Router::new()
        .route("/crypto_report", get(crypto_index))
        .route("/crypto_report/:id", get(crypto_view_report))
        .route("/crypto_report/:id/pdf", get(crypto_report_pdf))
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
    
    // First fetch real-time market data from Layer 5 → Layer 2 for enhanced template context
    let realtime_data = match service_islands.crypto_reports.fetch_realtime_market_data().await {
        Ok(data) => {
            println!("✅ [Route] Got real-time data from Layer 5 for template context");
            Some(data)
        }
        Err(e) => {
            println!("⚠️ [Route] Failed to get real-time data: {}, using cached template only", e);
            None
        }
    };

    // Use AppState with Tera engine from Service Islands - Full L1/L2 caching
    match service_islands.crypto_reports.handlers.crypto_index_with_tera(&service_islands.app_state).await {
        Ok(html) => {
            println!("✅ [Route] Template rendered successfully from Layer 5");
            
            // TODO: In future, inject realtime_data into template context
            // For now, template uses database data + WebSocket will provide real-time updates
            if realtime_data.is_some() {
                println!("📊 [Route] Real-time data available for future template enhancement");
            }
            
            // Use create_cached_response helper like archive code
            service_islands.crypto_reports.handlers.create_cached_response(html, "service-islands")
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
    let realtime_data = match service_islands.crypto_reports.fetch_realtime_market_data().await {
        Ok(data) => {
            println!("✅ [Route] Got real-time data from Layer 5 for template context");
            Some(data)
        }
        Err(e) => {
            println!("⚠️ [Route] Failed to get real-time data: {}, using cached template only", e);
            None
        }
    };

    // Use Service Islands architecture to get specific report
    match service_islands.crypto_reports.handlers.crypto_report_by_id_with_tera(&service_islands.app_state, report_id).await {
        Ok(html) => {
            println!("✅ [Route] Report ID: {} template rendered successfully from Layer 5", report_id);
            
            // TODO: In future, inject realtime_data into template context
            // For now, template uses database data + WebSocket will provide real-time updates
            if realtime_data.is_some() {
                println!("📊 [Route] Real-time data available for future template enhancement");
            }
            
            // Create cached response like archive_old_code with proper headers
            Response::builder()
                .status(StatusCode::OK)
                .header("cache-control", "public, max-age=300") // 5min cache for individual reports
                .header("content-type", "text/html; charset=utf-8")
                .header("x-cache", "Layer5-Generated")
                .header("x-report-id", report_id.to_string())
                .body(html)
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

/// PDF template for crypto report by ID with Service Islands Integration
async fn crypto_report_pdf(
    Path(id): Path<String>,
    State(service_islands): State<Arc<ServiceIslands>>
) -> impl IntoResponse {
    println!("📄 [Route] crypto_report_pdf called for ID: {} - fetching from Service Islands Layer 5", id);
    
    // Parse report ID
    let report_id: i32 = match id.parse() {
        Ok(id) => id,
        Err(_) => {
            println!("❌ [Route] Invalid report ID format for PDF: {}", id);
            return (
                StatusCode::BAD_REQUEST,
                "Invalid report ID format"
            ).into_response();
        }
    };

    println!("📄 [Route] Requesting PDF for report ID: {}", report_id);

    // First fetch real-time market data from Layer 5 → Layer 2 for enhanced template context
    let realtime_data = match service_islands.crypto_reports.fetch_realtime_market_data().await {
        Ok(data) => {
            println!("✅ [Route] Got real-time data from Layer 5 for PDF template context");
            Some(data)
        }
        Err(e) => {
            println!("⚠️ [Route] Failed to get real-time data for PDF: {}, using cached template only", e);
            None
        }
    };

    // Use Service Islands architecture to get PDF template for specific report
    match service_islands.crypto_reports.handlers.crypto_report_pdf_with_tera(&service_islands.app_state, report_id).await {
        Ok(html) => {
            println!("✅ [Route] PDF template for report ID: {} rendered successfully from Layer 5", report_id);
            
            // TODO: In future, inject realtime_data into PDF template context
            // For now, template uses database data + WebSocket will provide real-time updates
            if realtime_data.is_some() {
                println!("📊 [Route] Real-time data available for future PDF template enhancement");
            }
            
            // Create PDF-optimized response with proper headers
            Response::builder()
                .status(StatusCode::OK)
                .header("cache-control", "public, max-age=300") // 5min cache for PDF templates
                .header("content-type", "text/html; charset=utf-8")
                .header("x-cache", "Layer5-Generated-PDF")
                .header("x-report-id", report_id.to_string())
                .header("x-template-type", "pdf-printable")
                .body(html)
                .unwrap()
                .into_response()
        }
        Err(e) => {
            eprintln!("❌ [Route] Failed to render PDF template for report ID: {} from Layer 5: {}", report_id, e);
            
            // Check if it's a not found case or other error
            let error_message = e.to_string();
            if error_message.contains("not found") || error_message.contains("Database error") {
                (
                    StatusCode::NOT_FOUND,
                    format!("Report #{} not found for PDF generation", report_id)
                ).into_response()
            } else {
                (
                    StatusCode::INTERNAL_SERVER_ERROR, 
                    format!("Failed to generate PDF for report #{}", report_id)
                ).into_response()
            }
        }
    }
}
