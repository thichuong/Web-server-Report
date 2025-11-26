//! SEO Routes Module
//!
//! Handles SEO-related endpoints including:
//! - sitemap.xml generation
//!
//! These routes are designed for search engine optimization and follow
//! the Service Islands architecture (Layer 5 -> Layer 3 -> Layer 1).

use axum::{
    body::Body,
    extract::State,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use std::sync::Arc;
use tracing::{error, info};

use crate::service_islands::layer3_communication::data_communication::crypto_data_service::CryptoDataService;
use crate::service_islands::layer5_business_logic::shared::SitemapCreator;
use crate::service_islands::ServiceIslands;

/// Configure SEO routes
pub fn configure_seo_routes() -> Router<Arc<ServiceIslands>> {
    Router::new().route("/sitemap.xml", get(sitemap_xml))
}

/// Generate and serve sitemap.xml
///
/// Flow: Route Handler -> Layer 5 (`SitemapCreator`) -> Layer 3 (`CryptoDataService`)
///
/// Response:
/// - Content-Type: application/xml; charset=utf-8
/// - Cache-Control: public, max-age=3600 (1 hour)
async fn sitemap_xml(State(service_islands): State<Arc<ServiceIslands>>) -> impl IntoResponse {
    info!("Generating sitemap.xml");

    // Get legacy app state for database access
    let app_state = service_islands.get_legacy_app_state();

    // Layer 3: Fetch report data from database
    let data_service = CryptoDataService::new();
    let reports_result = data_service
        .fetch_all_report_ids_for_sitemap(&app_state)
        .await;

    match reports_result {
        Ok(reports) => {
            // Convert to tuple format expected by SitemapCreator
            let report_data: Vec<(i32, chrono::DateTime<chrono::Utc>)> =
                reports.into_iter().map(|r| (r.id, r.created_at)).collect();

            // Layer 5: Generate sitemap XML
            match SitemapCreator::generate_sitemap_xml(report_data) {
                Ok(xml) => {
                    info!("Sitemap.xml generated successfully ({} bytes)", xml.len());

                    Response::builder()
                        .status(StatusCode::OK)
                        .header(header::CONTENT_TYPE, "application/xml; charset=utf-8")
                        .header(header::CACHE_CONTROL, "public, max-age=3600")
                        .header("X-Robots-Tag", "noindex")
                        .body(Body::from(xml))
                        .unwrap_or_else(|e| {
                            error!("Failed to build sitemap response: {}", e);
                            Response::builder()
                                .status(StatusCode::INTERNAL_SERVER_ERROR)
                                .body(Body::from("Failed to generate sitemap"))
                                .unwrap_or_else(|_| Response::new(Body::from("Failed to generate sitemap")))
                        })
                        .into_response()
                }
                Err(e) => {
                    error!("Failed to generate sitemap XML: {}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Failed to generate sitemap",
                    )
                        .into_response()
                }
            }
        }
        Err(e) => {
            error!("Failed to fetch reports for sitemap: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to fetch sitemap data",
            )
                .into_response()
        }
    }
}
