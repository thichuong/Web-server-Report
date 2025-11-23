//! Sitemap XML Generator for Layer 5 Business Logic
//!
//! Generates dynamic sitemap.xml following the sitemap protocol 0.9 specification.
//! This module creates XML content for SEO purposes, including:
//! - Static pages (homepage, crypto_report index, reports list)
//! - Dynamic pages (individual crypto reports from database)
//!
//! Reference: https://www.sitemaps.org/protocol.html

use std::fmt::Write;
use chrono::{DateTime, Utc};
use tracing::info;

use super::error::{Layer5Error, Layer5Result};

/// Base URL for the website
const BASE_URL: &str = "https://cryptodashboard.me";

/// Represents a single URL entry in the sitemap
#[derive(Debug, Clone)]
pub struct SitemapEntry {
    /// Full URL location
    pub loc: String,
    /// Last modification date in W3C format (YYYY-MM-DD)
    pub lastmod: Option<String>,
    /// How frequently the page is likely to change
    pub changefreq: ChangeFrequency,
    /// Priority of this URL relative to other URLs (0.0 to 1.0)
    pub priority: f32,
}

/// Change frequency hints for search engines
#[derive(Debug, Clone, Copy)]
pub enum ChangeFrequency {
    Always,
    Hourly,
    Daily,
    Weekly,
    Monthly,
    Yearly,
    Never,
}

impl ChangeFrequency {
    /// Convert to sitemap XML value
    fn as_str(&self) -> &'static str {
        match self {
            Self::Always => "always",
            Self::Hourly => "hourly",
            Self::Daily => "daily",
            Self::Weekly => "weekly",
            Self::Monthly => "monthly",
            Self::Yearly => "yearly",
            Self::Never => "never",
        }
    }
}

/// Sitemap XML generator
pub struct SitemapCreator;

impl SitemapCreator {
    /// Generate complete sitemap XML from static and dynamic entries
    ///
    /// # Arguments
    /// * `report_data` - Vector of tuples (report_id, created_at) from database
    ///
    /// # Returns
    /// Complete sitemap XML string
    pub fn generate_sitemap_xml(
        report_data: Vec<(i32, DateTime<Utc>)>,
    ) -> Layer5Result<String> {
        let today = Utc::now().format("%Y-%m-%d").to_string();

        // Pre-calculate capacity to minimize allocations
        // Each URL entry is approximately 300-400 bytes
        let estimated_capacity = 500 + (report_data.len() * 400);
        let mut xml = String::with_capacity(estimated_capacity);

        // XML declaration and urlset opening tag
        writeln!(xml, r#"<?xml version="1.0" encoding="UTF-8"?>"#)
            .map_err(|e| Layer5Error::Internal(format!("Failed to write XML header: {}", e)))?;
        writeln!(xml, r#"<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">"#)
            .map_err(|e| Layer5Error::Internal(format!("Failed to write urlset: {}", e)))?;

        // Static entries
        let static_entries = Self::get_static_entries(&today);
        for entry in &static_entries {
            Self::write_url_entry(&mut xml, entry)?;
        }

        // Dynamic entries from database
        let dynamic_entries = Self::create_dynamic_entries(report_data);
        for entry in &dynamic_entries {
            Self::write_url_entry(&mut xml, entry)?;
        }

        // Close urlset
        writeln!(xml, "</urlset>")
            .map_err(|e| Layer5Error::Internal(format!("Failed to close urlset: {}", e)))?;

        let total_urls = static_entries.len() + dynamic_entries.len();
        info!(
            "Sitemap generated successfully: {} total URLs ({} static, {} dynamic)",
            total_urls,
            static_entries.len(),
            dynamic_entries.len()
        );

        Ok(xml)
    }

    /// Get static page entries
    fn get_static_entries(today: &str) -> Vec<SitemapEntry> {
        vec![
            // Homepage - highest priority
            SitemapEntry {
                loc: BASE_URL.to_string(),
                lastmod: Some(today.to_string()),
                changefreq: ChangeFrequency::Daily,
                priority: 1.0,
            },
            // Latest crypto report index
            SitemapEntry {
                loc: format!("{}/crypto_report", BASE_URL),
                lastmod: Some(today.to_string()),
                changefreq: ChangeFrequency::Daily,
                priority: 0.9,
            },
            // Reports list page
            SitemapEntry {
                loc: format!("{}/crypto_reports_list", BASE_URL),
                lastmod: Some(today.to_string()),
                changefreq: ChangeFrequency::Daily,
                priority: 0.8,
            },
        ]
    }

    /// Create dynamic entries from database report data
    fn create_dynamic_entries(report_data: Vec<(i32, DateTime<Utc>)>) -> Vec<SitemapEntry> {
        report_data
            .into_iter()
            .map(|(id, created_at)| {
                let lastmod = created_at.format("%Y-%m-%d").to_string();
                SitemapEntry {
                    loc: format!("{}/crypto_report/{}", BASE_URL, id),
                    lastmod: Some(lastmod),
                    changefreq: ChangeFrequency::Monthly,
                    priority: 0.7,
                }
            })
            .collect()
    }

    /// Write a single URL entry to the XML string
    fn write_url_entry(xml: &mut String, entry: &SitemapEntry) -> Layer5Result<()> {
        writeln!(xml, "  <url>")
            .map_err(|e| Layer5Error::Internal(format!("XML write error: {}", e)))?;

        // Location (required)
        writeln!(xml, "    <loc>{}</loc>", Self::escape_xml(&entry.loc))
            .map_err(|e| Layer5Error::Internal(format!("XML write error: {}", e)))?;

        // Last modification (optional)
        if let Some(ref lastmod) = entry.lastmod {
            writeln!(xml, "    <lastmod>{}</lastmod>", lastmod)
                .map_err(|e| Layer5Error::Internal(format!("XML write error: {}", e)))?;
        }

        // Change frequency (optional)
        writeln!(xml, "    <changefreq>{}</changefreq>", entry.changefreq.as_str())
            .map_err(|e| Layer5Error::Internal(format!("XML write error: {}", e)))?;

        // Priority (optional)
        writeln!(xml, "    <priority>{:.1}</priority>", entry.priority)
            .map_err(|e| Layer5Error::Internal(format!("XML write error: {}", e)))?;

        writeln!(xml, "  </url>")
            .map_err(|e| Layer5Error::Internal(format!("XML write error: {}", e)))?;

        Ok(())
    }

    /// Escape special XML characters in URLs
    ///
    /// According to sitemap protocol, URLs must be entity-escaped:
    /// - & -> &amp;
    /// - ' -> &apos;
    /// - " -> &quot;
    /// - > -> &gt;
    /// - < -> &lt;
    fn escape_xml(input: &str) -> String {
        let mut result = String::with_capacity(input.len());
        for c in input.chars() {
            match c {
                '&' => result.push_str("&amp;"),
                '\'' => result.push_str("&apos;"),
                '"' => result.push_str("&quot;"),
                '>' => result.push_str("&gt;"),
                '<' => result.push_str("&lt;"),
                _ => result.push(c),
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_generate_sitemap_with_reports() {
        let reports = vec![
            (1, Utc.with_ymd_and_hms(2024, 1, 15, 10, 0, 0).unwrap()),
            (2, Utc.with_ymd_and_hms(2024, 2, 20, 12, 30, 0).unwrap()),
        ];

        let xml = SitemapCreator::generate_sitemap_xml(reports).unwrap();

        // Verify XML structure
        assert!(xml.starts_with(r#"<?xml version="1.0" encoding="UTF-8"?>"#));
        assert!(xml.contains("<urlset"));
        assert!(xml.contains("</urlset>"));

        // Verify static URLs
        assert!(xml.contains("<loc>https://cryptodashboard.me</loc>"));
        assert!(xml.contains("<loc>https://cryptodashboard.me/crypto_report</loc>"));
        assert!(xml.contains("<loc>https://cryptodashboard.me/crypto_reports_list</loc>"));

        // Verify dynamic URLs
        assert!(xml.contains("<loc>https://cryptodashboard.me/crypto_report/1</loc>"));
        assert!(xml.contains("<loc>https://cryptodashboard.me/crypto_report/2</loc>"));

        // Verify lastmod format
        assert!(xml.contains("<lastmod>2024-01-15</lastmod>"));
        assert!(xml.contains("<lastmod>2024-02-20</lastmod>"));
    }

    #[test]
    fn test_generate_sitemap_empty_reports() {
        let xml = SitemapCreator::generate_sitemap_xml(vec![]).unwrap();

        // Should still have static URLs
        assert!(xml.contains("<loc>https://cryptodashboard.me</loc>"));
        assert!(xml.contains("<priority>1.0</priority>"));
    }

    #[test]
    fn test_escape_xml() {
        assert_eq!(SitemapCreator::escape_xml("test&value"), "test&amp;value");
        assert_eq!(SitemapCreator::escape_xml("a<b>c"), "a&lt;b&gt;c");
        assert_eq!(SitemapCreator::escape_xml("normal"), "normal");
    }

    #[test]
    fn test_change_frequency_as_str() {
        assert_eq!(ChangeFrequency::Daily.as_str(), "daily");
        assert_eq!(ChangeFrequency::Monthly.as_str(), "monthly");
        assert_eq!(ChangeFrequency::Always.as_str(), "always");
    }
}
