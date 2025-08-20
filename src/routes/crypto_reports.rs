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
        .route("/crypto_reports_list", get(crypto_reports_list))
        .route("/test-layer5-data", get(test_layer5_data_flow))  // 🔍 DEBUG: Test Layer 5 → Layer 2 data flow
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

/// List all crypto reports (placeholder)
async fn crypto_reports_list() -> impl IntoResponse {
    "List of crypto reports - not yet implemented from Service Islands".into_response()
}

/// 🔍 DEBUG: Test Layer 5 → Layer 2 data flow
/// 
/// This endpoint tests if Layer 5 (Crypto Reports Island) can successfully fetch data from Layer 2 (External APIs Island)
async fn test_layer5_data_flow(
    State(service_islands): State<Arc<ServiceIslands>>
) -> impl IntoResponse {
    println!("🔍 [DEBUG] Testing Layer 5 → Layer 2 data flow...");
    
    // Test fetching dashboard data through Layer 5
    match service_islands.crypto_reports.fetch_realtime_market_data().await {
        Ok(market_data) => {
            let response = serde_json::json!({
                "status": "success",
                "message": "Layer 5 successfully received data from Layer 2",
                "data": market_data,
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "data_flow": "Layer 5 (Business Logic) ← Layer 2 (External APIs)"
            });
            
            println!("✅ Layer 5 → Layer 2 data flow test successful!");
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            let error_response = serde_json::json!({
                "status": "error",
                "message": format!("Layer 5 failed to fetch data from Layer 2: {}", e),
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "data_flow": "Layer 5 (Business Logic) ✗ Layer 2 (External APIs)"
            });
            
            println!("❌ Layer 5 → Layer 2 data flow test failed: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)).into_response()
        }
    }
}

/// View specific crypto report by ID
async fn crypto_view_report(
    Path(id): Path<String>,
    State(_service_islands): State<Arc<ServiceIslands>>
) -> Html<String> {
    Html(format!(r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>Crypto Report #{}</title>
        <link rel="stylesheet" href="/shared_assets/css/report.css">
    </head>
    <body>
        <h1>📊 Crypto Report #{}</h1>
        <div class="report-container">
            <p>Report ID: {}</p>
            <p>Generated by Service Islands Architecture</p>
        </div>
    </body>
    </html>
    "#, id, id, id))
}

/// PDF template for reports
async fn pdf_template(
    Path(id): Path<String>,
    State(_service_islands): State<Arc<ServiceIslands>>
) -> Html<String> {
    Html(format!(r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>PDF Template #{}</title>
        <link rel="stylesheet" href="/shared_assets/css/pdf-template.css">
    </head>
    <body>
        <div class="pdf-template">
            <h1>PDF Report Template #{}</h1>
            <p>This is a PDF-ready template for report ID: {}</p>
        </div>
    </body>
    </html>
    "#, id, id, id))
}
