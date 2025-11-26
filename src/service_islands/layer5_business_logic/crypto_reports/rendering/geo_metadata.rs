//! GEO Metadata Generator
//!
//! This module provides Generative Engine Optimization (GEO) metadata generation
//! for crypto reports. It helps AI bots (Grok, GPT, Claude) better understand
//! page content through:
//! - Dynamic Open Graph and Twitter Card meta tags
//! - JSON-LD structured data (Schema.org Article)
//! - Semantic HTML recommendations

use serde::Serialize;

use super::shared::Report;

/// Base URL for the website
const SITE_BASE_URL: &str = "https://cryptodashboard.me";

/// Default logo URL for publisher
const PUBLISHER_LOGO_URL: &str = "https://cryptodashboard.me/shared_assets/images/logo.png";

/// Default OG image URL
const DEFAULT_OG_IMAGE: &str = "https://cryptodashboard.me/shared_assets/images/image.jpg";

/// GEO Metadata container
///
/// Contains all metadata needed for SEO and AI bot optimization
#[derive(Debug, Clone, Serialize)]
pub struct GeoMetadata {
    /// Report ID
    pub report_id: i32,
    /// Dynamic page title
    pub title: String,
    /// Vietnamese title
    pub title_vi: String,
    /// English title
    pub title_en: String,
    /// Meta description
    pub description: String,
    /// Vietnamese description
    pub description_vi: String,
    /// English description
    pub description_en: String,
    /// Canonical URL
    pub canonical_url: String,
    /// ISO 8601 formatted date
    pub date_published: String,
    /// Human readable date (Vietnamese)
    pub date_display_vi: String,
    /// Human readable date (English)
    pub date_display_en: String,
    /// OG image URL
    pub og_image: String,
}

impl GeoMetadata {
    /// Create GEO metadata from a Report
    ///
    /// Generates all necessary metadata for SEO and AI optimization.
    ///
    /// # Arguments
    /// * `report` - The report to generate metadata for
    ///
    /// # Returns
    /// A `GeoMetadata` struct with all fields populated
    #[must_use] 
    pub fn from_report(report: &Report) -> Self {
        let report_id = report.id;
        let created_at = report.created_at;

        // Format dates
        let date_published = created_at.format("%Y-%m-%dT%H:%M:%SZ").to_string();

        // Vietnamese timezone (UTC+7)
        let vn_time = created_at + chrono::Duration::hours(7);
        let date_display_vi = vn_time.format("%d/%m/%Y %H:%M").to_string();
        let date_display_en = vn_time.format("%B %d, %Y at %H:%M").to_string();

        // Generate titles
        let title_vi = format!(
            "Phân Tích Thị Trường Crypto #{} - {}",
            report_id,
            vn_time.format("%d/%m/%Y")
        );
        let title_en = format!(
            "Crypto Market Analysis Report #{} - {}",
            report_id,
            vn_time.format("%Y-%m-%d")
        );
        let title = title_vi.clone(); // Default to Vietnamese

        // Generate descriptions
        let description_vi = format!(
            "Báo cáo phân tích thị trường tiền mã hóa #{report_id} với dữ liệu Bitcoin, Ethereum, \
            chỉ số kỹ thuật RSI/MACD và chỉ số Fear & Greed. Cập nhật {date_display_vi}."
        );
        let description_en = format!(
            "Crypto market analysis report #{report_id} featuring Bitcoin, Ethereum data, \
            RSI/MACD technical indicators, and Fear & Greed Index. Updated {date_display_en}."
        );
        let description = description_vi.clone();

        // Generate canonical URL
        let canonical_url = format!("{SITE_BASE_URL}/crypto_report/{report_id}");

        Self {
            report_id,
            title,
            title_vi,
            title_en,
            description,
            description_vi,
            description_en,
            canonical_url,
            date_published,
            date_display_vi,
            date_display_en,
            og_image: DEFAULT_OG_IMAGE.to_string(),
        }
    }
}

/// Generate Open Graph and Twitter Card meta tags as HTML string
///
/// Creates dynamic meta tags optimized for social sharing and AI bots.
///
/// # Arguments
/// * `metadata` - The `GeoMetadata` to render
/// * `language` - Optional language code ("vi" or "en"), defaults to "vi"
///
/// # Returns
/// HTML string containing all meta tags
///
/// # Performance
/// Uses `format!` macro with pre-calculated capacity for efficient string building
#[must_use] 
pub fn generate_meta_tags(metadata: &GeoMetadata, language: Option<&str>) -> String {
    let lang = language.unwrap_or("vi");

    let (title, description) = if lang == "en" {
        (&metadata.title_en, &metadata.description_en)
    } else {
        (&metadata.title_vi, &metadata.description_vi)
    };

    // Pre-calculate capacity for efficient allocation
    // Approximate size: ~2KB for all meta tags
    let mut html = String::with_capacity(2048);

    // Essential meta tags
    // Essential meta tags
    // Use write! to avoid intermediate string allocation
    let _ = std::fmt::Write::write_fmt(&mut html, format_args!(
        r#"<meta name="description" content="{description}" />
    <link rel="canonical" href="{canonical}" />

    <!-- Open Graph Meta Tags (Facebook, LinkedIn, Discord) -->
    <meta property="og:title" content="{title}" />
    <meta property="og:description" content="{description}" />
    <meta property="og:type" content="article" />
    <meta property="og:url" content="{canonical}" />
    <meta property="og:image" content="{og_image}" />
    <meta property="og:image:width" content="1200" />
    <meta property="og:image:height" content="630" />
    <meta property="og:site_name" content="CryptoDashboard" />
    <meta property="og:locale" content="{locale}" />
    <meta property="article:published_time" content="{published}" />
    <meta property="article:author" content="CryptoDashboard" />
    <meta property="article:section" content="Cryptocurrency" />
    <meta property="article:tag" content="Bitcoin" />
    <meta property="article:tag" content="Cryptocurrency" />
    <meta property="article:tag" content="Market Analysis" />

    <!-- Twitter Card Meta Tags (X/Twitter, Grok) -->
    <meta name="twitter:card" content="summary_large_image" />
    <meta name="twitter:title" content="{title}" />
    <meta name="twitter:description" content="{description}" />
    <meta name="twitter:image" content="{og_image}" />
    <meta name="twitter:site" content="@cryptodashboard" />
    <meta name="twitter:creator" content="@cryptodashboard" />

    <!-- Additional SEO Meta Tags -->
    <meta name="robots" content="index, follow, max-image-preview:large" />
    <meta name="author" content="CryptoDashboard" />
    <meta name="keywords" content="crypto, bitcoin, ethereum, market analysis, BTC, ETH, cryptocurrency, trading" />"#,
        description = escape_html_attr(description),
        title = escape_html_attr(title),
        canonical = &metadata.canonical_url,
        og_image = &metadata.og_image,
        locale = if lang == "en" { "en_US" } else { "vi_VN" },
        published = &metadata.date_published,
    ));

    html
}

/// Generate JSON-LD structured data for Schema.org Article
///
/// Creates structured data that helps AI bots and search engines
/// understand the content semantically.
///
/// # Arguments
/// * `metadata` - The `GeoMetadata` to render
/// * `language` - Optional language code ("vi" or "en"), defaults to "vi"
///
/// # Returns
/// HTML script tag containing JSON-LD data
///
/// # Schema Type
/// Uses `Article` with `FinancialAnalysis` as additional type for maximum compatibility
#[must_use] 
pub fn generate_json_ld(metadata: &GeoMetadata, language: Option<&str>) -> String {
    let lang = language.unwrap_or("vi");

    let (headline, description) = if lang == "en" {
        (&metadata.title_en, &metadata.description_en)
    } else {
        (&metadata.title_vi, &metadata.description_vi)
    };

    // Build JSON-LD using Serialize for proper escaping
    let json_ld = JsonLdArticle {
        context: "https://schema.org",
        type_field: "Article",
        additional_type: "FinancialAnalysis",
        headline: headline.clone(),
        description: description.clone(),
        date_published: metadata.date_published.clone(),
        date_modified: metadata.date_published.clone(),
        url: metadata.canonical_url.clone(),
        image: JsonLdImage {
            type_field: "ImageObject",
            url: metadata.og_image.clone(),
            width: 1200,
            height: 630,
        },
        author: JsonLdOrganization {
            type_field: "Organization",
            name: "CryptoDashboard".to_string(),
            url: SITE_BASE_URL.to_string(),
        },
        publisher: JsonLdPublisher {
            type_field: "Organization",
            name: "CryptoDashboard".to_string(),
            url: SITE_BASE_URL.to_string(),
            logo: JsonLdImage {
                type_field: "ImageObject",
                url: PUBLISHER_LOGO_URL.to_string(),
                width: 512,
                height: 512,
            },
        },
        main_entity_of_page: JsonLdWebPage {
            type_field: "WebPage",
            id: metadata.canonical_url.clone(),
        },
        in_language: if lang == "en" { "en-US" } else { "vi-VN" }.to_string(),
        about: vec![
            JsonLdThing {
                type_field: "Thing",
                name: "Bitcoin".to_string(),
            },
            JsonLdThing {
                type_field: "Thing",
                name: "Cryptocurrency".to_string(),
            },
            JsonLdThing {
                type_field: "Thing",
                name: "Market Analysis".to_string(),
            },
        ],
    };

    // Serialize to JSON with proper escaping
    match serde_json::to_string_pretty(&json_ld) {
        Ok(json_str) => format!(
            r#"<script type="application/ld+json">
{json_str}
    </script>"#
        ),
        Err(_) => String::new(), // Fallback: no JSON-LD if serialization fails
    }
}

/// Generate complete GEO metadata HTML (meta tags + JSON-LD)
///
/// Convenience function that combines both meta tags and JSON-LD
/// into a single HTML string for easy injection into templates.
///
/// # Arguments
/// * `report` - The report to generate metadata for
/// * `language` - Optional language code ("vi" or "en")
///
/// # Returns
/// Tuple of (`meta_tags_html`, `json_ld_html`, `dynamic_title`)
#[must_use] 
pub fn generate_complete_geo_metadata(
    report: &Report,
    language: Option<&str>,
) -> (String, String, String) {
    let metadata = GeoMetadata::from_report(report);
    let lang = language.unwrap_or("vi");

    let meta_tags = generate_meta_tags(&metadata, Some(lang));
    let json_ld = generate_json_ld(&metadata, Some(lang));
    let title = if lang == "en" {
        metadata.title_en
    } else {
        metadata.title_vi
    };

    (meta_tags, json_ld, title)
}

// ============================================================================
// Helper structs for JSON-LD serialization
// ============================================================================

#[derive(Serialize)]
struct JsonLdArticle {
    #[serde(rename = "@context")]
    context: &'static str,
    #[serde(rename = "@type")]
    type_field: &'static str,
    #[serde(rename = "additionalType")]
    additional_type: &'static str,
    headline: String,
    description: String,
    #[serde(rename = "datePublished")]
    date_published: String,
    #[serde(rename = "dateModified")]
    date_modified: String,
    url: String,
    image: JsonLdImage,
    author: JsonLdOrganization,
    publisher: JsonLdPublisher,
    #[serde(rename = "mainEntityOfPage")]
    main_entity_of_page: JsonLdWebPage,
    #[serde(rename = "inLanguage")]
    in_language: String,
    about: Vec<JsonLdThing>,
}

#[derive(Serialize)]
struct JsonLdImage {
    #[serde(rename = "@type")]
    type_field: &'static str,
    url: String,
    width: u32,
    height: u32,
}

#[derive(Serialize)]
struct JsonLdOrganization {
    #[serde(rename = "@type")]
    type_field: &'static str,
    name: String,
    url: String,
}

#[derive(Serialize)]
struct JsonLdPublisher {
    #[serde(rename = "@type")]
    type_field: &'static str,
    name: String,
    url: String,
    logo: JsonLdImage,
}

#[derive(Serialize)]
struct JsonLdWebPage {
    #[serde(rename = "@type")]
    type_field: &'static str,
    #[serde(rename = "@id")]
    id: String,
}

#[derive(Serialize)]
struct JsonLdThing {
    #[serde(rename = "@type")]
    type_field: &'static str,
    name: String,
}

// ============================================================================
// Utility functions
// ============================================================================

/// Escape HTML attribute values
///
/// Escapes special characters to prevent XSS in meta tag content attributes.
#[inline]
fn escape_html_attr(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_report() -> Report {
        Report {
            id: 123,
            html_content: "<p>Test content</p>".to_string(),
            css_content: None,
            js_content: None,
            html_content_en: None,
            js_content_en: None,
            created_at: Utc::now(),
        }
    }

    #[test]
    fn test_geo_metadata_from_report() {
        let report = create_test_report();
        let metadata = GeoMetadata::from_report(&report);

        assert_eq!(metadata.report_id, 123);
        assert!(metadata.title_vi.contains("#123"));
        assert!(metadata.title_en.contains("#123"));
        assert!(metadata.canonical_url.contains("/crypto_report/123"));
    }

    #[test]
    fn test_generate_meta_tags_vi() {
        let report = create_test_report();
        let metadata = GeoMetadata::from_report(&report);
        let html = generate_meta_tags(&metadata, Some("vi"));

        assert!(html.contains("og:title"));
        assert!(html.contains("twitter:card"));
        assert!(html.contains("summary_large_image"));
        assert!(html.contains("vi_VN"));
    }

    #[test]
    fn test_generate_meta_tags_en() {
        let report = create_test_report();
        let metadata = GeoMetadata::from_report(&report);
        let html = generate_meta_tags(&metadata, Some("en"));

        assert!(html.contains("en_US"));
        assert!(html.contains("Crypto Market Analysis"));
    }

    #[test]
    fn test_generate_json_ld() {
        let report = create_test_report();
        let metadata = GeoMetadata::from_report(&report);
        let html = generate_json_ld(&metadata, Some("vi"));

        assert!(html.contains("application/ld+json"));
        assert!(html.contains("@context"));
        assert!(html.contains("schema.org"));
        assert!(html.contains("Article"));
    }

    #[test]
    fn test_escape_html_attr() {
        assert_eq!(
            escape_html_attr("Test & \"quote\""),
            "Test &amp; &quot;quote&quot;"
        );
        assert_eq!(escape_html_attr("<script>"), "&lt;script&gt;");
    }
}
