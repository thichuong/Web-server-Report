//! Crypto Reports HTTP Request Handlers
//!
//! This module contains all HTTP request handlers related to crypto reports functionality.
//! ONLY uses Template Engine - NO manual HTML creation

use axum::{
    body::Body,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use flate2::{Compression, write::GzEncoder};
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::error::Error as StdError;
use std::hash::{Hash, Hasher};
use std::io::Write;
use std::sync::{Arc, atomic::Ordering};
use tracing::{debug, error, info, warn};

use crate::services::crypto_reports::rendering::{
    generate_breadcrumbs_and_related, generate_complete_geo_metadata,
};

// Import from current state
use crate::state::AppState;

// Import from our specialized components
use super::report_creator::ReportCreator;
use super::template_orchestrator::TemplateOrchestrator;
use crate::services::shared::error::Layer5Result;

/// Rendered content ready for HTTP response
/// Decouples business logic from HTTP transport
pub struct RenderedContent {
    pub data: Vec<u8>,
    pub cache_control: &'static str,
    pub cache_status: &'static str,
}

impl IntoResponse for RenderedContent {
    fn into_response(self) -> Response {
        Response::builder()
            .status(StatusCode::OK)
            .header("cache-control", self.cache_control)
            .header("content-type", "text/html; charset=utf-8")
            .header("content-encoding", "gzip")
            .header("x-render-mode", "declarative-shadow-dom")
            .header("x-cache", self.cache_status)
            .body(Body::from(self.data))
            .unwrap_or_else(|_| Response::new(Body::from("Response build error")))
            .into_response()
    }
}

/// Crypto Handlers
///
/// Contains all HTTP request handlers for crypto reports-related operations.
/// These handlers manage crypto report generation and API interactions.
pub struct CryptoHandlers {
    pub report_creator: ReportCreator,
    pub template_orchestrator: TemplateOrchestrator,
}

impl Default for CryptoHandlers {
    fn default() -> Self {
        Self::new()
    }
}

impl CryptoHandlers {
    /// Create a new `CryptoHandlers` instance
    #[must_use]
    pub fn new() -> Self {
        let report_creator = ReportCreator::new();
        let template_orchestrator = TemplateOrchestrator::new(report_creator.clone());

        Self {
            report_creator,
            template_orchestrator,
        }
    }

    /// Health check for crypto handlers
    #[must_use]
    pub fn health_check(&self) -> bool {
        let report_creator_ok = self.report_creator.health_check();
        let template_orchestrator_ok = self.template_orchestrator.health_check();

        report_creator_ok && template_orchestrator_ok
    }

    /// Create cached response with proper headers
    #[allow(dead_code)]
    #[must_use]
    pub fn create_cached_response(&self, html: String, cache_status: &str) -> Response {
        Response::builder()
            .status(StatusCode::OK)
            .header("cache-control", "public, max-age=15")
            .header("content-type", "text/html; charset=utf-8")
            .header("x-cache", cache_status)
            .body(html)
            .unwrap_or_else(|e| {
                warn!("⚠️ Failed to build cached response: {}", e);
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body("Response build error".to_string())
                    .unwrap_or_else(|_| Response::new("Response build error".to_string()))
            })
            .into_response()
    }

    /// Create compressed HTTP response with proper headers
    #[must_use]
    pub fn create_compressed_response(compressed_data: Vec<u8>) -> Response {
        Response::builder()
            .status(StatusCode::OK)
            .header("cache-control", "public, max-age=15")
            .header("x-cache", "compressed")
            .header("content-type", "text/html; charset=utf-8")
            .header("content-encoding", "gzip")
            .body(Body::from(compressed_data))
            .unwrap_or_else(|e| {
                warn!("⚠️ Failed to build compressed response: {}", e);
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::from("Response build error"))
                    .unwrap_or_else(|_| Response::new(Body::from("Response build error")))
            })
            .into_response()
    }

    /// Compress HTML string to gzip format
    ///
    /// # Errors
    ///
    /// Returns error if compression fails
    pub fn compress_html_to_gzip(html: &str) -> Result<Vec<u8>, Box<dyn StdError + Send + Sync>> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(html.as_bytes())?;
        let compressed_data = encoder.finish()?;

        let original_size = html.len();
        let compressed_size = compressed_data.len();
        #[allow(clippy::cast_precision_loss)]
        let compression_ratio = (1.0 - (compressed_size as f64 / original_size as f64)) * 100.0;

        info!(
            "🗜️  CryptoHandlers: HTML compressed - Original: {}KB, Compressed: {}KB, Ratio: {:.1}%",
            original_size / 1024,
            compressed_size / 1024,
            compression_ratio
        );

        Ok(compressed_data)
    }

    /// Crypto Reports List handler - Paginated list of all reports
    ///
    /// Returns compressed data (Vec<u8>) for optimal transfer speed
    ///
    /// # Errors
    ///
    /// Returns error if database fetch fails
    pub async fn crypto_reports_list_with_tera(
        &self,
        state: &Arc<AppState>,
        page: i64,
    ) -> Layer5Result<RenderedContent> {
        info!(
            "📋 Layer 5: Nhận yêu cầu cho crypto reports list page {}",
            page
        );

        let request_count = state.request_counter.fetch_add(1, Ordering::Relaxed);

        if request_count.is_multiple_of(50) {
            info!(
                "Processed {} requests to crypto_reports_list",
                request_count
            );
        }

        let data_service = &self.report_creator.data_service;
        let per_page: i64 = 10;

        match data_service
            .fetch_reports_list_with_cache(state, page, per_page)
            .await
        {
            Ok(Some(compressed_data)) => {
                let size_kb = compressed_data.len() / 1024;
                info!(
                    "✅ Layer 5: Nhận compressed data từ Layer 3 cho reports list page {} ({}KB)",
                    page, size_kb
                );

                Ok(RenderedContent {
                    data: compressed_data,
                    cache_control: "public, max-age=60",
                    cache_status: "Layer5-Compressed",
                })
            }
            Ok(None) => {
                warn!(
                    "⚠️ Layer 5: Layer 3 trả về None cho reports list page {}",
                    page
                );
                Err(super::super::shared::error::Layer5Error::TemplateRender(
                    "No reports list data available".into(),
                ))
            }
            Err(e) => {
                error!(
                    "❌ Layer 5: Layer 3 error cho reports list page {}: {}",
                    page, e
                );
                Err(super::super::shared::error::Layer5Error::Internal(
                    e.to_string(),
                ))
            }
        }
    }

    /// Serve Shadow DOM content for Declarative Shadow DOM architecture
    ///
    /// Delegates to `ReportCreator` for actual Shadow DOM content generation.
    ///
    /// # Errors
    ///
    /// Returns error if report generation fails
    pub async fn serve_shadow_dom_content(
        &self,
        state: &Arc<AppState>,
        report_id: i32,
        shadow_dom_token: &str,
        language: Option<&str>,
    ) -> Result<axum::response::Response, Box<dyn StdError + Send + Sync>> {
        info!(
            "CryptoHandlers: Delegating Shadow DOM content request to ReportCreator for report {} with token {}",
            report_id, shadow_dom_token
        );

        self.report_creator
            .serve_shadow_dom_content(state, report_id, shadow_dom_token, language)
            .await
            .map_err(super::super::shared::error::Layer5Error::into_boxed)
    }

    /// Detect preferred language from request
    /// Priority: Query param > Cookie > Accept-Language header > Default (vi)
    pub fn detect_preferred_language(
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
        if let Some(cookie_header) = headers.get("cookie")
            && let Ok(cookie_str) = cookie_header.to_str()
        {
            for cookie in cookie_str.split(';') {
                let parts: Vec<&str> = cookie.trim().splitn(2, '=').collect();
                if let [name_part, value_part] = parts.as_slice() {
                    let (name, value) = (name_part.trim(), value_part.trim());
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

        // 3. Check Accept-Language header
        if let Some(accept_lang) = headers.get("accept-language")
            && let Ok(lang_str) = accept_lang.to_str()
        {
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

        // 4. Default to Vietnamese
        debug!("🌐 [Language] Using default: vi");
        None
    }

    /// Render Crypto Index DSD (Latest Report)
    /// Encapsulates all logic for the `crypto_index` route
    ///
    /// # Errors
    ///
    /// Returns error if database fetch or template rendering fails
    #[allow(clippy::too_many_lines)]
    pub async fn render_crypto_index_dsd(
        &self,
        state: &Arc<AppState>,
        params: &HashMap<String, String>,
        headers: &HeaderMap,
        report_id_opt: Option<i32>,
    ) -> Layer5Result<RenderedContent> {
        debug!(
            "🌓 [Handler] render_crypto_index_dsd called - using Declarative Shadow DOM architecture"
        );

        let report_id_value = report_id_opt.unwrap_or(-1);

        debug!(
            "🚀 [Handler] render_crypto_index_dsd called for {}",
            if report_id_value == -1 {
                "latest report".to_string()
            } else {
                format!("report ID: {report_id_value}")
            }
        );

        // STEP 1: Quick cache check with default language (Vietnamese)
        let data_service = &self.report_creator.data_service;
        let default_language = "vi";

        if let Ok(Some(cached_compressed)) = data_service
            .get_rendered_report_dsd_compressed(state, report_id_value, default_language)
            .await
        {
            info!(
                "✅ [Handler] DSD cache HIT (default language) - returning compressed HTML for {}",
                if report_id_value == -1 {
                    "latest".to_string()
                } else {
                    format!("#{report_id_value}")
                }
            );

            return Ok(RenderedContent {
                data: cached_compressed,
                cache_control: "public, max-age=300",
                cache_status: "HIT",
            });
        }

        // STEP 1.1: Cache miss with default language - now detect preferred language
        let preferred_language =
            Self::detect_preferred_language(params, headers).unwrap_or_else(|| "vi".to_string());

        debug!(
            "🔍 [Handler] DSD cache MISS (default language) - detected preferred language: {}",
            preferred_language
        );

        // STEP 1.2: If preferred language differs from default, try cache with preferred language
        if preferred_language != default_language
            && let Ok(Some(cached_compressed)) = data_service
                .get_rendered_report_dsd_compressed(state, report_id_value, &preferred_language)
                .await
        {
            info!(
                "✅ [Handler] DSD cache HIT (preferred language: {}) - returning compressed HTML for {}",
                preferred_language,
                if report_id_value == -1 {
                    "latest".to_string()
                } else {
                    format!("#{report_id_value}")
                }
            );

            return Ok(RenderedContent {
                data: cached_compressed,
                cache_control: "public, max-age=300",
                cache_status: "HIT",
            });
        }

        debug!("🔍 [Handler] DSD cache MISS - generating fresh HTML");

        // STEP 2: Fetch report from database
        let report_result = if report_id_value == -1 {
            self.report_creator
                .fetch_and_cache_latest_report(state)
                .await
        } else {
            self.report_creator
                .fetch_and_cache_report_by_id(state, report_id_value)
                .await
        };

        let report = match report_result {
            Ok(Some(report)) => report,
            Ok(None) => {
                warn!("⚠️ [Handler] No report found for DSD view");
                return Err(crate::services::shared::error::Layer5Error::NotFound(
                    "Report not found".to_string(),
                ));
            }
            Err(e) => {
                error!("❌ [Handler] Database error fetching report for DSD: {}", e);
                return Err(e.into());
            }
        };

        // STEP 3: Generate shadow_dom_token
        let mut hasher = DefaultHasher::new();
        report.id.hash(&mut hasher);
        report.created_at.hash(&mut hasher);
        let shadow_dom_token = format!("sb_{:x}", hasher.finish());

        // STEP 4: Generate shadow DOM content
        let sandboxed_report = self.report_creator.create_sandboxed_report(&report);
        let shadow_dom_content = self
            .report_creator
            .generate_shadow_dom_content(&sandboxed_report, Some(&preferred_language));

        info!(
            "🌐 [Handler] render_crypto_index_dsd rendering with language: {}",
            preferred_language
        );

        // STEP 5: Generate GEO metadata for AI bots
        let (geo_meta_tags, geo_json_ld, geo_title) =
            generate_complete_geo_metadata(&report, Some(&preferred_language));
        debug!(
            "📊 [Handler] GEO metadata generated for report {} - title: {}",
            report.id, geo_title
        );

        // STEP 5.1: Fetch related reports for internal linking
        let related_reports_data = match data_service
            .fetch_related_reports(state, report.id, 3)
            .await
        {
            Ok(reports) => reports,
            Err(e) => {
                warn!("⚠️ [Handler] Failed to fetch related reports: {}", e);
                vec![]
            }
        };

        // STEP 5.2: Generate breadcrumbs and related reports data
        let (breadcrumb_items, breadcrumbs_schema, related_reports) =
            generate_breadcrumbs_and_related(report.id, &related_reports_data);

        // STEP 6: Render template
        let mut context = tera::Context::new();
        context.insert("report", &report);
        context.insert("shadow_dom_token", &shadow_dom_token);
        context.insert("shadow_dom_content", &shadow_dom_content);
        context.insert(
            "websocket_url",
            &std::env::var("WEBSOCKET_SERVICE_URL")
                .unwrap_or_else(|_| "ws://localhost:8081/ws".to_string()),
        );
        // GEO metadata
        context.insert("geo_meta_tags", &geo_meta_tags);
        context.insert("geo_json_ld", &geo_json_ld);
        context.insert("geo_title", &geo_title);
        // Breadcrumbs and related reports
        context.insert("breadcrumb_items", &breadcrumb_items);
        context.insert("breadcrumbs_schema", &breadcrumbs_schema);
        context.insert("related_reports", &related_reports);

        let html = match state
            .tera
            .render("crypto/routes/reports/view.html", &context)
        {
            Ok(html) => html,
            Err(e) => {
                error!("❌ [Handler] Failed to render DSD template: {}", e);
                return Err(crate::services::shared::error::Layer5Error::TemplateRender(
                    e.to_string(),
                ));
            }
        };

        // STEP 7: Compress HTML
        let compressed_data = match Self::compress_html_to_gzip(&html) {
            Ok(data) => data,
            Err(e) => {
                error!("❌ [Handler] Failed to compress DSD HTML: {}", e);
                return Err(crate::services::shared::error::Layer5Error::Compression(
                    e.to_string(),
                ));
            }
        };

        // STEP 8: Cache response
        if let Err(e) = data_service
            .cache_rendered_report_dsd_compressed(
                state,
                report_id_value,
                &compressed_data,
                &preferred_language,
            )
            .await
        {
            warn!("⚠️ [Handler] Failed to cache DSD compressed content: {}", e);
        }

        info!("✅ [Handler] render_crypto_index_dsd completed successfully");

        // STEP 9: Return compressed response
        Ok(RenderedContent {
            data: compressed_data,
            cache_control: "public, max-age=300",
            cache_status: "MISS",
        })
    }

    /// Render Crypto Report by ID DSD
    /// Encapsulates all logic for the `crypto_view_report` route
    ///
    /// # Errors
    ///
    /// Returns error if database fetch or template rendering fails
    pub async fn render_crypto_report_dsd(
        &self,
        state: &Arc<AppState>,
        report_id: i32,
        params: &HashMap<String, String>,
        headers: &HeaderMap,
    ) -> Layer5Result<RenderedContent> {
        self.render_crypto_index_dsd(state, params, headers, Some(report_id))
            .await
    }
}
