//! RSS Feed XML Generator for Layer 5 Business Logic
//!
//! Generates dynamic RSS 2.0 feed following the RSS 2.0 specification.
//! This module creates XML content for search engines and AI bots to discover new reports.
//!
//! Reference: <https://www.rssboard.org/rss-specification>
//!
//! Features:
//! - RFC 822 date formatting for pubDate
//! - HTML content extraction for descriptions
//! - XML entity escaping
//! - Atom namespace for self-referencing link

use chrono::{DateTime, FixedOffset, Offset, Utc};
use std::fmt::Write;
use tracing::info;

use super::error::{Layer5Error, Layer5Result};
use crate::service_islands::layer3_communication::data_communication::crypto_data_service::ReportRssData;

/// Base URL for the website
const BASE_URL: &str = "https://cryptodashboard.me";

/// Maximum characters for description extraction
const MAX_DESCRIPTION_LENGTH: usize = 300;

/// RSS Feed metadata
struct FeedMetadata {
    title: &'static str,
    link: &'static str,
    description: &'static str,
    language: &'static str,
}

impl Default for FeedMetadata {
    fn default() -> Self {
        Self {
            title: "CryptoDashboard - Báo cáo Thị trường Crypto",
            link: BASE_URL,
            description: "Báo cáo phân tích thị trường crypto hàng ngày với dữ liệu real-time từ Binance, CoinGecko và các nguồn uy tín",
            language: "vi-VN",
        }
    }
}

/// RSS Feed XML generator
pub struct RssCreator;

impl RssCreator {
    /// Generate complete RSS 2.0 XML feed from report data
    ///
    /// # Arguments
    /// * `reports` - Vector of `ReportRssData` from database
    ///
    /// # Returns
    /// # Returns
    /// Complete RSS 2.0 XML string
    ///
    /// # Errors
    ///
    /// Returns error if XML writing fails
    pub fn generate_rss_xml(reports: &[ReportRssData]) -> Layer5Result<String> {
        let metadata = FeedMetadata::default();
        let now = Utc::now();

        // Pre-calculate capacity to minimize allocations
        // Each item entry is approximately 500-700 bytes
        let estimated_capacity = 1000 + (reports.len() * 700);
        let mut xml = String::with_capacity(estimated_capacity);

        // XML declaration
        writeln!(xml, r#"<?xml version="1.0" encoding="UTF-8"?>"#)
            .map_err(|e| Layer5Error::Internal(format!("Failed to write XML header: {e}")))?;

        // RSS 2.0 opening tag with Atom namespace for self-referencing link
        writeln!(
            xml,
            r#"<rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom">"#
        )
        .map_err(|e| Layer5Error::Internal(format!("Failed to write rss tag: {e}")))?;

        // Channel opening
        writeln!(xml, "  <channel>")
            .map_err(|e| Layer5Error::Internal(format!("Failed to write channel: {e}")))?;

        // Channel metadata
        Self::write_channel_metadata(&mut xml, &metadata, &now)?;

        // Write items
        for report in reports {
            Self::write_item(&mut xml, report)?;
        }

        // Close channel and rss
        writeln!(xml, "  </channel>")
            .map_err(|e| Layer5Error::Internal(format!("Failed to close channel: {e}")))?;
        writeln!(xml, "</rss>")
            .map_err(|e| Layer5Error::Internal(format!("Failed to close rss: {e}")))?;

        info!(
            "📡 RSS feed generated successfully: {} items, {} bytes",
            reports.len(),
            xml.len()
        );

        Ok(xml)
    }

    /// Write channel metadata section
    fn write_channel_metadata(
        xml: &mut String,
        metadata: &FeedMetadata,
        now: &DateTime<Utc>,
    ) -> Layer5Result<()> {
        // Required elements
        writeln!(
            xml,
            "    <title>{}</title>",
            Self::escape_xml(metadata.title)
        )
        .map_err(|e| Layer5Error::Internal(format!("XML write error: {e}")))?;

        writeln!(xml, "    <link>{}</link>", metadata.link)
            .map_err(|e| Layer5Error::Internal(format!("XML write error: {e}")))?;

        writeln!(
            xml,
            "    <description>{}</description>",
            Self::escape_xml(metadata.description)
        )
        .map_err(|e| Layer5Error::Internal(format!("XML write error: {e}")))?;

        // Optional but recommended elements
        writeln!(xml, "    <language>{}</language>", metadata.language)
            .map_err(|e| Layer5Error::Internal(format!("XML write error: {e}")))?;

        writeln!(
            xml,
            "    <lastBuildDate>{}</lastBuildDate>",
            Self::format_rfc822_date(now)
        )
        .map_err(|e| Layer5Error::Internal(format!("XML write error: {e}")))?;

        // Atom self-referencing link (recommended for feed readers)
        writeln!(
            xml,
            r#"    <atom:link href="{BASE_URL}/rss.xml" rel="self" type="application/rss+xml"/>"#
        )
        .map_err(|e| Layer5Error::Internal(format!("XML write error: {e}")))?;

        // Generator
        writeln!(
            xml,
            "    <generator>CryptoDashboard Rust Web Server</generator>"
        )
        .map_err(|e| Layer5Error::Internal(format!("XML write error: {e}")))?;

        // TTL (time to live in minutes) - 60 minutes
        writeln!(xml, "    <ttl>60</ttl>")
            .map_err(|e| Layer5Error::Internal(format!("XML write error: {e}")))?;

        Ok(())
    }

    /// Write a single item entry
    fn write_item(xml: &mut String, report: &ReportRssData) -> Layer5Result<()> {
        writeln!(xml, "    <item>")
            .map_err(|e| Layer5Error::Internal(format!("XML write error: {e}")))?;

        // Title with date in Vietnamese timezone (UTC+7)
        // Safe: 7 * 3600 = 25200 seconds is well within the valid range (±86400 seconds)
        // Double fallback: first to UTC (offset 0), then to a compile-time verified UTC offset
        let vn_offset = FixedOffset::east_opt(7 * 3600)
            .or_else(|| FixedOffset::east_opt(0))
            .unwrap_or_else(|| Utc.fix());
        let vn_time = report.created_at.with_timezone(&vn_offset);
        let date_str = vn_time.format("%d/%m/%Y").to_string();
        let title = format!("Báo cáo Thị trường Crypto #{} - {}", report.id, date_str);

        writeln!(xml, "      <title>{}</title>", Self::escape_xml(&title))
            .map_err(|e| Layer5Error::Internal(format!("XML write error: {e}")))?;

        // Link
        let link = format!("{}/crypto_report/{}", BASE_URL, report.id);
        writeln!(xml, "      <link>{link}</link>")
            .map_err(|e| Layer5Error::Internal(format!("XML write error: {e}")))?;

        // GUID (globally unique identifier) - same as link, marked as permalink
        writeln!(xml, r#"      <guid isPermaLink="true">{link}</guid>"#)
            .map_err(|e| Layer5Error::Internal(format!("XML write error: {e}")))?;

        // Publication date in RFC 822 format
        writeln!(
            xml,
            "      <pubDate>{}</pubDate>",
            Self::format_rfc822_date(&report.created_at)
        )
        .map_err(|e| Layer5Error::Internal(format!("XML write error: {e}")))?;

        // Description - extract from HTML content
        let description = Self::extract_description(&report.html_content, MAX_DESCRIPTION_LENGTH);
        writeln!(
            xml,
            "      <description>{}</description>",
            Self::escape_xml(&description)
        )
        .map_err(|e| Layer5Error::Internal(format!("XML write error: {e}")))?;

        writeln!(xml, "    </item>")
            .map_err(|e| Layer5Error::Internal(format!("XML write error: {e}")))?;

        Ok(())
    }

    /// Format `DateTime` to RFC 822 standard for RSS pubDate
    ///
    /// Format: "Sun, 23 Nov 2025 14:00:00 +0700"
    /// RSS 2.0 requires dates in RFC 822 format
    fn format_rfc822_date(dt: &DateTime<Utc>) -> String {
        // Convert to Vietnam timezone (UTC+7) for display
        let vn_offset = FixedOffset::east_opt(7 * 3600).unwrap_or_else(|| {
            // Fallback to UTC if VN offset fails (should not happen with constant)
            #[allow(clippy::expect_used)]
            FixedOffset::east_opt(0).expect("UTC offset 0 should always be valid")
        });
        let vn_time = dt.with_timezone(&vn_offset);

        // RFC 822 format: "Sun, 23 Nov 2025 14:00:00 +0700"
        vn_time.format("%a, %d %b %Y %H:%M:%S %z").to_string()
    }

    /// Extract plain text description from HTML content
    ///
    /// Removes HTML tags and extracts first N characters for RSS description.
    /// Adds ellipsis if content is truncated.
    fn extract_description(html: &str, max_len: usize) -> String {
        // Simple HTML tag removal - strip all tags
        let mut result = String::with_capacity(max_len + 10);
        let mut in_tag = false;
        let mut char_count = 0;

        for c in html.chars() {
            if char_count >= max_len {
                break;
            }

            match c {
                '<' => in_tag = true,
                '>' => in_tag = false,
                _ if !in_tag => {
                    // Skip multiple whitespaces
                    if c.is_whitespace() {
                        if !result.ends_with(' ') && !result.is_empty() {
                            result.push(' ');
                            char_count += 1;
                        }
                    } else {
                        result.push(c);
                        char_count += 1;
                    }
                }
                _ => {}
            }
        }

        // Trim and add ellipsis if truncated
        let trimmed = result.trim().to_string();
        if html.len() > max_len && !trimmed.is_empty() {
            format!("{trimmed}...")
        } else {
            trimmed
        }
    }

    /// Escape special XML characters
    ///
    /// According to XML specification, these characters must be entity-escaped:
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
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_generate_rss_with_reports() {
        let reports = vec![
            ReportRssData {
                id: 1,
                html_content: "<div>Báo cáo thị trường crypto ngày hôm nay</div>".to_string(),
                created_at: Utc.with_ymd_and_hms(2025, 11, 23, 7, 0, 0).unwrap(),
            },
            ReportRssData {
                id: 2,
                html_content: "<p>Bitcoin tăng mạnh</p>".to_string(),
                created_at: Utc.with_ymd_and_hms(2025, 11, 22, 7, 0, 0).unwrap(),
            },
        ];

        let xml = RssCreator::generate_rss_xml(&reports).unwrap();

        // Verify XML structure
        assert!(xml.starts_with(r#"<?xml version="1.0" encoding="UTF-8"?>"#));
        assert!(xml.contains("<rss version=\"2.0\""));
        assert!(xml.contains("</rss>"));
        assert!(xml.contains("<channel>"));
        assert!(xml.contains("</channel>"));

        // Verify channel metadata
        assert!(xml.contains("<title>CryptoDashboard"));
        assert!(xml.contains("<language>vi-VN</language>"));
        assert!(xml.contains("<ttl>60</ttl>"));

        // Verify items
        assert!(xml.contains("<item>"));
        assert!(xml.contains("Báo cáo Thị trường Crypto #1"));
        assert!(xml.contains("Báo cáo Thị trường Crypto #2"));
        assert!(xml.contains("<link>https://cryptodashboard.me/crypto_report/1</link>"));
        assert!(xml.contains("<guid isPermaLink=\"true\">"));
    }

    #[test]
    fn test_generate_rss_empty_reports() {
        let xml = RssCreator::generate_rss_xml(&[]).unwrap();

        // Should still have valid channel
        assert!(xml.contains("<channel>"));
        assert!(xml.contains("<title>CryptoDashboard"));
        assert!(!xml.contains("<item>"));
    }

    #[test]
    fn test_format_rfc822_date() {
        let dt = Utc.with_ymd_and_hms(2025, 11, 23, 7, 0, 0).unwrap();
        let formatted = RssCreator::format_rfc822_date(&dt);

        // Should be in format: "Sun, 23 Nov 2025 14:00:00 +0700" (converted to VN timezone)
        assert!(formatted.contains("Nov 2025"));
        assert!(formatted.contains("+0700"));
    }

    #[test]
    fn test_extract_description() {
        let html = "<div><h1>Title</h1><p>This is a paragraph with some text.</p></div>";
        let desc = RssCreator::extract_description(html, 50);

        assert!(!desc.contains('<'));
        assert!(!desc.contains('>'));
        assert!(desc.contains("Title"));
        assert!(desc.contains("paragraph"));
    }

    #[test]
    fn test_extract_description_truncation() {
        let html = "<p>This is a very long text that should be truncated because it exceeds the maximum length</p>";
        let desc = RssCreator::extract_description(html, 30);

        assert!(desc.len() <= 35); // 30 + "..."
        assert!(desc.ends_with("..."));
    }

    #[test]
    fn test_escape_xml() {
        assert_eq!(RssCreator::escape_xml("test&value"), "test&amp;value");
        assert_eq!(RssCreator::escape_xml("a<b>c"), "a&lt;b&gt;c");
        assert_eq!(RssCreator::escape_xml("normal"), "normal");
        assert_eq!(RssCreator::escape_xml("\"quoted\""), "&quot;quoted&quot;");
    }
}
