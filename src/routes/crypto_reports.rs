//! Crypto Reports Routes
//!
//! This module defines all HTTP routes for crypto reports functionality including
//! report viewing and listing.

use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use crate::service_islands::layer5_business_logic::crypto_reports::handlers::CryptoHandlers;
use crate::service_islands::layer5_business_logic::crypto_reports::rendering::{
    generate_breadcrumbs_and_related, generate_complete_geo_metadata,
};
use crate::service_islands::ServiceIslands;

/// Configure crypto reports routes
pub fn configure_crypto_reports_routes() -> Router<Arc<ServiceIslands>> {
    Router::new()
        .route("/crypto_report", get(crypto_index))
        .route("/crypto_report/:id", get(crypto_view_report))
        .route("/crypto_reports_list", get(crypto_reports_list))
}

/// Detect preferred language from request
/// Priority: Query param > Cookie > Accept-Language header > Default (vi)
fn detect_preferred_language(
    query_params: &HashMap<String, String>,
    headers: &HeaderMap,
) -> Option<String> {
    // 1. Check query parameter (?lang=en or ?lang=vi)
    if let Some(lang) = query_params.get("lang") {
        let lang = lang.to_lowercase();
        if lang == "en" || lang == "vi" {
            debug!("🌐 [Language] Detected from query param: {}", lang);
            return Some(lang);
        }
    }

    // 2. Check Cookie header for preferred_language or language
    if let Some(cookie_header) = headers.get("cookie") {
        if let Ok(cookie_str) = cookie_header.to_str() {
            // Parse cookies manually
            for cookie in cookie_str.split(';') {
                let parts: Vec<&str> = cookie.trim().splitn(2, '=').collect();
                if parts.len() == 2 {
                    let (name, value) = (parts[0].trim(), parts[1].trim());
                    if name == "preferred_language" || name == "language" {
                        let lang = value.to_lowercase();
                        if lang == "en" || lang == "vi" {
                            debug!("🌐 [Language] Detected from cookie: {}", lang);
                            return Some(lang);
                        }
                    }
                }
            }
        }
    }

    // 3. Check Accept-Language header
    if let Some(accept_lang) = headers.get("accept-language") {
        if let Ok(lang_str) = accept_lang.to_str() {
            // Parse Accept-Language: "vi-VN,vi;q=0.9,en-US;q=0.8,en;q=0.7"
            for lang_tag in lang_str.split(',') {
                let lang = lang_tag.split(';').next().unwrap_or("").trim();
                if lang.starts_with("en") {
                    debug!("🌐 [Language] Detected from Accept-Language: en");
                    return Some("en".to_string());
                } else if lang.starts_with("vi") {
                    debug!("🌐 [Language] Detected from Accept-Language: vi");
                    return Some("vi".to_string());
                }
            }
        }
    }

    // 4. Default to Vietnamese
    debug!("🌐 [Language] Using default: vi");
    None // None means use default (vi) in generate_shadow_dom_content
}

/// List all crypto reports with pagination
async fn crypto_reports_list(
    Query(params): Query<HashMap<String, String>>,
    State(service_islands): State<Arc<ServiceIslands>>,
) -> impl IntoResponse {
    debug!("🚀 [Route] crypto_reports_list called - fetching from Service Islands Layer 5");

    // Parse pagination parameter
    let page: i64 = params.get("page").and_then(|p| p.parse().ok()).unwrap_or(1);
    debug!("📄 [Route] Requesting page: {}", page);

    // Use Service Islands architecture to get reports list (compressed)
    match service_islands
        .crypto_reports
        .handlers
        .crypto_reports_list_with_tera(&service_islands.get_legacy_app_state(), page)
        .await
    {
        Ok(compressed_data) => {
            let size_kb = compressed_data.len() / 1024;
            info!("✅ [Route] Reports list template rendered successfully from Layer 5 - compressed ({}KB)", size_kb);

            // Create compressed response with proper headers
            Response::builder()
                .status(StatusCode::OK)
                .header("cache-control", "public, max-age=60")
                .header("content-type", "text/html; charset=utf-8")
                .header("content-encoding", "gzip")
                .header("x-cache", "Layer5-Compressed")
                .body(Body::from(compressed_data))
                .unwrap_or_else(|e| {
                    warn!("⚠️ Failed to build reports list response: {}", e);
                    Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Body::from("Response build error"))
                        .unwrap() // This is guaranteed safe with literal body
                })
                .into_response()
        }
        Err(e) => {
            error!(
                "❌ [Route] Failed to render reports list template from Layer 5: {}",
                e
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to load reports list",
            )
                .into_response()
        }
    }
}

/// Crypto reports index page using Declarative Shadow DOM
/// Modern primary route for crypto reports
/// ✅ OPTIMIZED: Full caching support with language-specific cache keys
async fn crypto_index(
    State(service_islands): State<Arc<ServiceIslands>>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
) -> Response {
    debug!("🌓 [Route] crypto_index called - using Declarative Shadow DOM architecture");

    // Detect preferred language from request
    let preferred_language =
        detect_preferred_language(&params, &headers).unwrap_or_else(|| "vi".to_string());

    // Check if specific report ID is requested
    let report_id = params.get("id");
    let report_id_value = if let Some(id_str) = report_id {
        match id_str.parse::<i32>() {
            Ok(id) => id,
            Err(_) => {
                error!("❌ [Route] Invalid report ID format: {}", id_str);
                return (StatusCode::BAD_REQUEST, "Invalid report ID format").into_response();
            }
        }
    } else {
        -1 // Latest report sentinel
    };

    debug!(
        "🚀 [Route] crypto_index called for {} (language: {})",
        if report_id_value == -1 {
            "latest report".to_string()
        } else {
            format!("report ID: {}", report_id_value)
        },
        preferred_language
    );

    // STEP 1: Check cache for compressed DSD HTML
    let data_service = &service_islands.crypto_reports.report_creator.data_service;
    if let Ok(Some(cached_compressed)) = data_service
        .get_rendered_report_dsd_compressed(
            &service_islands.get_legacy_app_state(),
            report_id_value,
        )
        .await
    {
        info!(
            "✅ [Route] DSD cache HIT - returning compressed HTML for {} (language: {})",
            if report_id_value == -1 {
                "latest".to_string()
            } else {
                format!("#{}", report_id_value)
            },
            preferred_language
        );

        return Response::builder()
            .status(StatusCode::OK)
            .header("cache-control", "public, max-age=300")
            .header("content-type", "text/html; charset=utf-8")
            .header("content-encoding", "gzip")
            .header("x-render-mode", "declarative-shadow-dom")
            .header("x-cache", "HIT")
            .body(Body::from(cached_compressed))
            .unwrap_or_else(|e| {
                warn!("⚠️ Failed to build cached DSD response: {}", e);
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::from("Response build error"))
                    .unwrap()
            })
            .into_response();
    }

    debug!("🔍 [Route] DSD cache MISS - generating fresh HTML");

    // STEP 2: Fetch report from database (uses existing data cache)
    let report_result = if report_id_value == -1 {
        service_islands
            .crypto_reports
            .report_creator
            .fetch_and_cache_latest_report(&service_islands.get_legacy_app_state())
            .await
    } else {
        service_islands
            .crypto_reports
            .report_creator
            .fetch_and_cache_report_by_id(&service_islands.get_legacy_app_state(), report_id_value)
            .await
    };

    let report = match report_result {
        Ok(Some(report)) => report,
        Ok(None) => {
            warn!("⚠️ [Route] No report found for DSD view");
            return (StatusCode::NOT_FOUND, "Report not found").into_response();
        }
        Err(e) => {
            error!("❌ [Route] Database error fetching report for DSD: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
        }
    };

    // STEP 3: Generate shadow_dom_token
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    report.id.hash(&mut hasher);
    report.created_at.hash(&mut hasher);
    let shadow_dom_token = format!("sb_{:x}", hasher.finish());

    // STEP 4: Get chart modules content and generate shadow DOM content
    let chart_modules_content = service_islands.get_chart_modules_content();
    let sandboxed_report = service_islands
        .crypto_reports
        .report_creator
        .create_sandboxed_report(&report, Some(chart_modules_content.as_str()));
    let shadow_dom_content = service_islands
        .crypto_reports
        .report_creator
        .generate_shadow_dom_content(
            &sandboxed_report,
            Some(&preferred_language),
            Some(chart_modules_content.as_str()),
        );

    info!(
        "🌐 [Route] crypto_index rendering with language: {}",
        preferred_language
    );

    // STEP 5: Generate GEO metadata for AI bots (Grok, GPT, Claude)
    let (geo_meta_tags, geo_json_ld, geo_title) =
        generate_complete_geo_metadata(&report, Some(&preferred_language));
    debug!(
        "📊 [Route] GEO metadata generated for report {} - title: {}",
        report.id, geo_title
    );

    // STEP 5.1: Fetch related reports for internal linking (GEO optimization)
    let related_reports_data = match data_service
        .fetch_related_reports(
            &service_islands.get_legacy_app_state(),
            report.id,
            3, // Limit to 3 related reports
        )
        .await
    {
        Ok(reports) => reports,
        Err(e) => {
            warn!("⚠️ [Route] Failed to fetch related reports: {}", e);
            vec![] // Fallback to empty list on error
        }
    };

    // STEP 5.2: Generate breadcrumbs and related reports data
    let (breadcrumb_items, breadcrumbs_schema, related_reports) =
        generate_breadcrumbs_and_related(report.id, &related_reports_data);
    debug!(
        "📊 [Route] Breadcrumbs and {} related reports generated for report {}",
        related_reports.len(),
        report.id
    );

    // STEP 6: Render template with GEO metadata
    let mut context = tera::Context::new();
    context.insert("report", &report);
    context.insert("shadow_dom_token", &shadow_dom_token);
    context.insert("shadow_dom_content", &shadow_dom_content);
    context.insert("chart_modules_content", chart_modules_content.as_ref());
    context.insert(
        "websocket_url",
        &std::env::var("WEBSOCKET_SERVICE_URL")
            .unwrap_or_else(|_| "ws://localhost:8081/ws".to_string()),
    );
    // GEO metadata for AI optimization
    context.insert("geo_meta_tags", &geo_meta_tags);
    context.insert("geo_json_ld", &geo_json_ld);
    context.insert("geo_title", &geo_title);
    // Breadcrumbs and related reports for internal linking
    context.insert("breadcrumb_items", &breadcrumb_items);
    context.insert("breadcrumbs_schema", &breadcrumbs_schema);
    context.insert("related_reports", &related_reports);

    let app_state = service_islands.get_legacy_app_state();
    let html = match app_state
        .tera
        .render("crypto/routes/reports/view_dsd.html", &context)
    {
        Ok(html) => html,
        Err(e) => {
            error!("❌ [Route] Failed to render DSD template: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Template render error").into_response();
        }
    };

    // STEP 7: Compress HTML
    let compressed_data = match CryptoHandlers::compress_html_to_gzip(&html) {
        Ok(data) => data,
        Err(e) => {
            error!("❌ [Route] Failed to compress DSD HTML: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Compression error").into_response();
        }
    };

    // STEP 8: Cache the compressed HTML
    if let Err(e) = data_service
        .cache_rendered_report_dsd_compressed(
            &service_islands.get_legacy_app_state(),
            report_id_value,
            &compressed_data,
        )
        .await
    {
        warn!("⚠️ [Route] Failed to cache DSD compressed HTML: {}", e);
    }

    info!(
        "✅ [Route] DSD template rendered successfully for report {} (language: {})",
        report.id, preferred_language
    );

    // STEP 9: Return compressed response
    Response::builder()
        .status(StatusCode::OK)
        .header("cache-control", "public, max-age=300")
        .header("content-type", "text/html; charset=utf-8")
        .header("content-encoding", "gzip")
        .header("x-render-mode", "declarative-shadow-dom")
        .header("x-cache", "MISS")
        .body(Body::from(compressed_data))
        .unwrap_or_else(|e| {
            warn!("⚠️ Failed to build DSD response: {}", e);
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from("Response build error"))
                .unwrap()
        })
        .into_response()
}

/// View specific crypto report by ID using Declarative Shadow DOM
/// Modern primary route for viewing specific reports
/// ✅ OPTIMIZED: Full caching support with language-specific cache keys
async fn crypto_view_report(
    Path(id): Path<String>,
    State(service_islands): State<Arc<ServiceIslands>>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
) -> Response {
    debug!("🌓 [Route] crypto_view_report called for ID: {}", id);

    // Detect preferred language from request
    let preferred_language =
        detect_preferred_language(&params, &headers).unwrap_or_else(|| "vi".to_string());

    // Parse report ID
    let report_id: i32 = match id.parse() {
        Ok(id) => id,
        Err(_) => {
            error!("❌ [Route] Invalid report ID format: {}", id);
            return (StatusCode::BAD_REQUEST, "Invalid report ID format").into_response();
        }
    };

    debug!(
        "🚀 [Route] crypto_view_report called for report #{} (language: {})",
        report_id, preferred_language
    );

    // STEP 1: Check cache for compressed DSD HTML
    let data_service = &service_islands.crypto_reports.report_creator.data_service;
    if let Ok(Some(cached_compressed)) = data_service
        .get_rendered_report_dsd_compressed(&service_islands.get_legacy_app_state(), report_id)
        .await
    {
        info!(
            "✅ [Route] DSD cache HIT - returning compressed HTML for report #{} (language: {})",
            report_id, preferred_language
        );

        return Response::builder()
            .status(StatusCode::OK)
            .header("cache-control", "public, max-age=300")
            .header("content-type", "text/html; charset=utf-8")
            .header("content-encoding", "gzip")
            .header("x-render-mode", "declarative-shadow-dom")
            .header("x-report-id", report_id.to_string())
            .header("x-cache", "HIT")
            .body(Body::from(cached_compressed))
            .unwrap_or_else(|e| {
                warn!("⚠️ Failed to build cached DSD response: {}", e);
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::from("Response build error"))
                    .unwrap()
            })
            .into_response();
    }

    debug!(
        "🔍 [Route] DSD cache MISS - generating fresh HTML for report #{}",
        report_id
    );

    // STEP 2: Fetch report from database (uses existing data cache)
    let report = match service_islands
        .crypto_reports
        .report_creator
        .fetch_and_cache_report_by_id(&service_islands.get_legacy_app_state(), report_id)
        .await
    {
        Ok(Some(report)) => report,
        Ok(None) => {
            warn!("⚠️ [Route] Report {} not found for DSD view", report_id);
            return (
                StatusCode::NOT_FOUND,
                format!("Report #{} not found", report_id),
            )
                .into_response();
        }
        Err(e) => {
            error!(
                "❌ [Route] Database error fetching report {}: {}",
                report_id, e
            );
            return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
        }
    };

    // STEP 3: Generate shadow_dom_token
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    report.id.hash(&mut hasher);
    report.created_at.hash(&mut hasher);
    let shadow_dom_token = format!("sb_{:x}", hasher.finish());

    // STEP 4: Get chart modules content and generate shadow DOM content
    let chart_modules_content = service_islands.get_chart_modules_content();
    let sandboxed_report = service_islands
        .crypto_reports
        .report_creator
        .create_sandboxed_report(&report, Some(chart_modules_content.as_str()));
    let shadow_dom_content = service_islands
        .crypto_reports
        .report_creator
        .generate_shadow_dom_content(
            &sandboxed_report,
            Some(&preferred_language),
            Some(chart_modules_content.as_str()),
        );

    info!(
        "🌐 [Route] crypto_view_report rendering with language: {}",
        preferred_language
    );

    // STEP 5: Generate GEO metadata for AI bots (Grok, GPT, Claude)
    let (geo_meta_tags, geo_json_ld, geo_title) =
        generate_complete_geo_metadata(&report, Some(&preferred_language));
    debug!(
        "📊 [Route] GEO metadata generated for report {} - title: {}",
        report_id, geo_title
    );

    // STEP 5.1: Fetch related reports for internal linking (GEO optimization)
    let related_reports_data = match data_service
        .fetch_related_reports(
            &service_islands.get_legacy_app_state(),
            report_id,
            3, // Limit to 3 related reports
        )
        .await
    {
        Ok(reports) => reports,
        Err(e) => {
            warn!(
                "⚠️ [Route] Failed to fetch related reports for report {}: {}",
                report_id, e
            );
            vec![] // Fallback to empty list on error
        }
    };

    // STEP 5.2: Generate breadcrumbs and related reports data
    let (breadcrumb_items, breadcrumbs_schema, related_reports) =
        generate_breadcrumbs_and_related(report_id, &related_reports_data);
    debug!(
        "📊 [Route] Breadcrumbs and {} related reports generated for report {}",
        related_reports.len(),
        report_id
    );

    // STEP 6: Render template with GEO metadata
    let mut context = tera::Context::new();
    context.insert("report", &report);
    context.insert("shadow_dom_token", &shadow_dom_token);
    context.insert("shadow_dom_content", &shadow_dom_content);
    context.insert("chart_modules_content", chart_modules_content.as_ref());
    context.insert(
        "websocket_url",
        &std::env::var("WEBSOCKET_SERVICE_URL")
            .unwrap_or_else(|_| "ws://localhost:8081/ws".to_string()),
    );
    // GEO metadata for AI optimization
    context.insert("geo_meta_tags", &geo_meta_tags);
    context.insert("geo_json_ld", &geo_json_ld);
    context.insert("geo_title", &geo_title);
    // Breadcrumbs and related reports for internal linking
    context.insert("breadcrumb_items", &breadcrumb_items);
    context.insert("breadcrumbs_schema", &breadcrumbs_schema);
    context.insert("related_reports", &related_reports);

    let app_state = service_islands.get_legacy_app_state();
    let html = match app_state
        .tera
        .render("crypto/routes/reports/view_dsd.html", &context)
    {
        Ok(html) => html,
        Err(e) => {
            error!(
                "❌ [Route] Failed to render DSD template for report {}: {}",
                report_id, e
            );
            return (StatusCode::INTERNAL_SERVER_ERROR, "Template render error").into_response();
        }
    };

    // STEP 7: Compress HTML
    let compressed_data = match CryptoHandlers::compress_html_to_gzip(&html) {
        Ok(data) => data,
        Err(e) => {
            error!(
                "❌ [Route] Failed to compress DSD HTML for report {}: {}",
                report_id, e
            );
            return (StatusCode::INTERNAL_SERVER_ERROR, "Compression error").into_response();
        }
    };

    // STEP 8: Cache the compressed HTML
    if let Err(e) = data_service
        .cache_rendered_report_dsd_compressed(
            &service_islands.get_legacy_app_state(),
            report_id,
            &compressed_data,
        )
        .await
    {
        warn!(
            "⚠️ [Route] Failed to cache DSD compressed HTML for report {}: {}",
            report_id, e
        );
    }

    info!(
        "✅ [Route] DSD template rendered successfully for report {} (language: {})",
        report_id, preferred_language
    );

    // STEP 9: Return compressed response
    Response::builder()
        .status(StatusCode::OK)
        .header("cache-control", "public, max-age=300")
        .header("content-type", "text/html; charset=utf-8")
        .header("content-encoding", "gzip")
        .header("x-render-mode", "declarative-shadow-dom")
        .header("x-report-id", report_id.to_string())
        .header("x-cache", "MISS")
        .body(Body::from(compressed_data))
        .unwrap_or_else(|e| {
            warn!("⚠️ Failed to build DSD response: {}", e);
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from("Response build error"))
                .unwrap()
        })
        .into_response()
}
