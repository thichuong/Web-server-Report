//! Crypto Reports Handlers - HTTP request handlers for crypto reports endpoints
//!
//! Provides Axum route handlers that integrate with Crypto Reports Service Island
//! components for serving report creation, PDF generation, and data endpoints.

use crate::features::crypto_reports::{CryptoReports, PdfGenerator, ReportCreator, DataManager};
use crate::state::AppState;
use axum::{
    extract::{Path, Query, State},
    response::{Html, IntoResponse, Response},
    routing::{get, post, Router},
    http::StatusCode,
    Json,
};
use std::collections::HashMap;
use std::sync::{Arc, atomic::Ordering};
use serde_json::json;

/// Create crypto reports routes
pub fn crypto_reports_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/crypto-reports", get(get_reports_list))
        .route("/api/crypto-reports/:id", get(get_report_by_id))
        .route("/api/crypto-reports", post(create_new_report))
        .route("/api/crypto-reports/:id/pdf", get(generate_pdf_report))
        .route("/api/crypto-reports/statistics", get(get_report_statistics))
        .route("/api/crypto-reports/market-data", get(get_processed_market_data))
        .route("/crypto-reports/:id/pdf-template", get(pdf_template_page))
}

/// Get reports list with pagination
pub async fn get_reports_list(
    Query(params): Query<HashMap<String, String>>,
    State(state): State<Arc<AppState>>
) -> Response {
    // Increment request counter
    state.request_counter.fetch_add(1, Ordering::Relaxed);
    
    let page = params.get("page").and_then(|p| p.parse().ok()).unwrap_or(1);
    let limit = params.get("limit").and_then(|l| l.parse().ok()).unwrap_or(10);
    
    // Create data manager for report operations
    let data_manager = DataManager::default();
    
    // Mock report list generation
    let reports = match generate_mock_reports_list(page, limit).await {
        Ok(reports) => reports,
        Err(e) => {
            eprintln!("Error generating reports list: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch reports").into_response();
        }
    };

    Json(json!({
        "success": true,
        "data": {
            "reports": reports,
            "pagination": {
                "page": page,
                "limit": limit,
                "total": 100, // Mock total
                "has_next": page * limit < 100
            }
        }
    })).into_response()
}

/// Get specific report by ID
pub async fn get_report_by_id(
    Path(id): Path<i32>,
    State(state): State<Arc<AppState>>
) -> Response {
    // Increment request counter
    state.request_counter.fetch_add(1, Ordering::Relaxed);
    
    let data_manager = DataManager::default();
    
    match data_manager.get_report_with_cache(id).await {
        Ok(Some(report)) => {
            Json(json!({
                "success": true,
                "data": {
                    "report": report
                }
            })).into_response()
        }
        Ok(None) => {
            Json(json!({
                "success": false,
                "error": "Report not found"
            })).into_response()
        }
        Err(e) => {
            eprintln!("Error fetching report {}: {}", id, e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch report").into_response()
        }
    }
}

/// Create new report
pub async fn create_new_report(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>
) -> Response {
    // Increment request counter
    state.request_counter.fetch_add(1, Ordering::Relaxed);
    
    let report_creator = ReportCreator::default();
    
    match report_creator.create_new_report(payload).await {
        Ok(report) => {
            Json(json!({
                "success": true,
                "data": {
                    "report": report,
                    "message": "Report created successfully"
                }
            })).into_response()
        }
        Err(e) => {
            eprintln!("Error creating report: {}", e);
            Json(json!({
                "success": false,
                "error": "Failed to create report"
            })).into_response()
        }
    }
}

/// Generate PDF report
pub async fn generate_pdf_report(
    Path(id): Path<i32>,
    State(state): State<Arc<AppState>>
) -> Response {
    // Increment request counter
    state.request_counter.fetch_add(1, Ordering::Relaxed);
    
    let pdf_generator = PdfGenerator::default();
    
    match pdf_generator.generate_pdf_template(id).await {
        Ok(html_content) => {
            // Return HTML content that can be used for PDF generation
            axum::response::Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "text/html; charset=utf-8")
                .header("cache-control", "private, max-age=300") // 5 minutes cache
                .body(html_content)
                .unwrap()
                .into_response()
        }
        Err(e) => {
            eprintln!("Error generating PDF for report {}: {}", id, e);
            if e.to_string().contains("not found") {
                (StatusCode::NOT_FOUND, "Report not found").into_response()
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, "Failed to generate PDF").into_response()
            }
        }
    }
}

/// Get report statistics
pub async fn get_report_statistics(State(state): State<Arc<AppState>>) -> Response {
    // Increment request counter
    state.request_counter.fetch_add(1, Ordering::Relaxed);
    
    let data_manager = DataManager::default();
    
    match data_manager.get_report_statistics().await {
        Ok(statistics) => {
            Json(json!({
                "success": true,
                "data": {
                    "statistics": statistics,
                    "processing_metrics": data_manager.get_processing_metrics()
                }
            })).into_response()
        }
        Err(e) => {
            eprintln!("Error fetching statistics: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch statistics").into_response()
        }
    }
}

/// Get processed market data
pub async fn get_processed_market_data(State(state): State<Arc<AppState>>) -> Response {
    // Increment request counter
    state.request_counter.fetch_add(1, Ordering::Relaxed);
    
    let data_manager = DataManager::default();
    
    // Mock market data
    let mock_market_data = json!({
        "bitcoin_price": 45230.50,
        "ethereum_price": 3123.75,
        "total_market_cap": 1_800_000_000_000.0,
        "fear_greed_index": 68
    });
    
    match data_manager.process_market_data(mock_market_data).await {
        Ok(processed_data) => {
            Json(json!({
                "success": true,
                "data": {
                    "market_data": processed_data
                }
            })).into_response()
        }
        Err(e) => {
            eprintln!("Error processing market data: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to process market data").into_response()
        }
    }
}

/// PDF template page handler
pub async fn pdf_template_page(
    Path(id): Path<i32>,
    State(state): State<Arc<AppState>>
) -> Response {
    // Increment request counter
    state.request_counter.fetch_add(1, Ordering::Relaxed);
    
    let pdf_generator = PdfGenerator::default();
    
    match pdf_generator.generate_pdf_template(id).await {
        Ok(html_content) => Html(html_content).into_response(),
        Err(e) => {
            eprintln!("Error generating PDF template for report {}: {}", id, e);
            if e.to_string().contains("not found") {
                Html(format!(r#"
                    <!DOCTYPE html>
                    <html>
                    <head><title>Report Not Found</title></head>
                    <body>
                        <div style="text-align: center; padding: 2rem;">
                            <h1>Report #{} Not Found</h1>
                            <p>The requested report could not be found.</p>
                            <a href="/crypto">Back to Dashboard</a>
                        </div>
                    </body>
                    </html>
                "#, id)).into_response()
            } else {
                Html(r#"
                    <!DOCTYPE html>
                    <html>
                    <head><title>Error</title></head>
                    <body>
                        <div style="text-align: center; padding: 2rem;">
                            <h1>Error Generating Report</h1>
                            <p>An error occurred while generating the report.</p>
                            <a href="/crypto">Back to Dashboard</a>
                        </div>
                    </body>
                    </html>
                "#).into_response()
            }
        }
    }
}

// Helper functions

/// Generate mock reports list for API responses
async fn generate_mock_reports_list(page: i32, limit: i32) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error + Send + Sync>> {
    let mut reports = Vec::new();
    let start_id = ((page - 1) * limit) + 1;
    
    for i in 0..limit {
        let report_id = start_id + i;
        let created_at = chrono::Utc::now() - chrono::Duration::days(i as i64);
        
        reports.push(json!({
            "id": report_id,
            "title": format!("Crypto Market Report #{}", report_id),
            "summary": format!("Comprehensive market analysis for report {}", report_id),
            "created_at": created_at.to_rfc3339(),
            "status": "published",
            "views": ((report_id * 7) % 500) + 50, // Mock view count
            "pdf_available": true
        }));
    }
    
    Ok(reports)
}
