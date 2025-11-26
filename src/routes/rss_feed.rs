//! RSS Feed Routes Module
//!
//! Handles RSS feed endpoint for search engines, AI bots, and feed readers:
//! - /rss.xml - RSS 2.0 feed with latest crypto reports
//!
//! These routes follow the Service Islands architecture (Layer 5 -> Layer 3 -> Layer 1)
//! and are optimized for daily content discovery by bots and crawlers.

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
use crate::service_islands::layer5_business_logic::shared::RssCreator;
use crate::service_islands::ServiceIslands;

/// Default number of reports to include in RSS feed
const RSS_FEED_LIMIT: i64 = 20;

/// Configure RSS feed routes
pub fn configure_rss_routes() -> Router<Arc<ServiceIslands>> {
    Router::new()
        .route("/rss.xml", get(rss_feed))
        .route("/rss", get(rss_feed)) // Alternative path without .xml extension
}

/// Generate and serve RSS 2.0 feed
///
/// Flow: Route Handler -> Layer 5 (`RssCreator`) -> Layer 3 (`CryptoDataService`)
///
/// Response:
/// - Content-Type: application/rss+xml; charset=utf-8
/// - Cache-Control: public, max-age=3600 (1 hour)
///
/// Features:
/// - Returns 20 most recent crypto reports
/// - RFC 822 date formatting for pubDate
/// - HTML content extraction for descriptions
/// - Atom namespace for self-referencing link
async fn rss_feed(State(service_islands): State<Arc<ServiceIslands>>) -> impl IntoResponse {
    info!("📡 Generating RSS feed");

    // Get legacy app state for database access
    let app_state = service_islands.get_legacy_app_state();

    // Layer 3: Fetch report data from database
    let data_service = CryptoDataService::new();
    let reports_result = data_service
        .fetch_rss_reports(&app_state, RSS_FEED_LIMIT)
        .await;

    match reports_result {
        Ok(reports) => {
            let report_count = reports.len();

            // Layer 5: Generate RSS XML
            match RssCreator::generate_rss_xml(&reports) {
                Ok(xml) => {
                    info!(
                        "✅ RSS feed generated successfully: {} items, {} bytes",
                        report_count,
                        xml.len()
                    );

                    Response::builder()
                        .status(StatusCode::OK)
                        .header(header::CONTENT_TYPE, "application/rss+xml; charset=utf-8")
                        .header(header::CACHE_CONTROL, "public, max-age=3600")
                        .header("X-Robots-Tag", "index, follow")
                        .body(Body::from(xml))
                        .unwrap_or_else(|e| {
                            error!("Failed to build RSS response: {}", e);
                            Response::builder()
                                .status(StatusCode::INTERNAL_SERVER_ERROR)
                                .body(Body::from("Failed to generate RSS feed"))
                                .unwrap_or_else(|_| Response::new(Body::from("Failed to generate RSS feed")))
                        })
                        .into_response()
                }
                Err(e) => {
                    error!("❌ Failed to generate RSS XML: {}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Failed to generate RSS feed",
                    )
                        .into_response()
                }
            }
        }
        Err(e) => {
            error!("❌ Failed to fetch reports for RSS feed: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to fetch RSS data",
            )
                .into_response()
        }
    }
}
