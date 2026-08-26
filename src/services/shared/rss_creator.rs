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
use crate::services::data_communication::crypto_data_service::ReportRssData;

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

        // Channel Image / Logo for RSS feed readers
        writeln!(xml, "    <image>")
            .map_err(|e| Layer5Error::Internal(format!("XML write error: {e}")))?;
        writeln!(
            xml,
            "      <url>{BASE_URL}/frontend/shared/assets/images/logo.svg</url>"
        )
        .map_err(|e| Layer5Error::Internal(format!("XML write error: {e}")))?;
        writeln!(
            xml,
            "      <title>{}</title>",
            Self::escape_xml(metadata.title)
        )
        .map_err(|e| Layer5Error::Internal(format!("XML write error: {e}")))?;
        writeln!(xml, "      <link>{}</link>", metadata.link)
            .map_err(|e| Layer5Error::Internal(format!("XML write error: {e}")))?;
        writeln!(xml, "      <width>144</width>")
            .map_err(|e| Layer5Error::Internal(format!("XML write error: {e}")))?;
        writeln!(xml, "      <height>144</height>")
            .map_err(|e| Layer5Error::Internal(format!("XML write error: {e}")))?;
        writeln!(xml, "    </image>")
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

        // Categories
        writeln!(xml, "      <category>Báo cáo thị trường</category>")
            .map_err(|e| Layer5Error::Internal(format!("XML write error: {e}")))?;
        writeln!(xml, "      <category>Cryptocurrency</category>")
            .map_err(|e| Layer5Error::Internal(format!("XML write error: {e}")))?;
        writeln!(xml, "      <category>Phân tích thị trường</category>")
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
            FixedOffset::east_opt(0).unwrap_or(Utc.fix())
        });
        let vn_time = dt.with_timezone(&vn_offset);

        // RFC 822 format: "Sun, 23 Nov 2025 14:00:00 +0700"
        vn_time.format("%a, %d %b %Y %H:%M:%S %z").to_string()
    }

    /// Extract plain text description from HTML content
    ///
    /// Removes HTML tags, filters out `<style>`, `<script>`, and comment blocks,
    /// and extracts first N characters for RSS description.
    /// Adds ellipsis if content is truncated.
    fn extract_description(html: &str, max_len: usize) -> String {
        let mut result = String::with_capacity(max_len + 10);
        let mut i = 0;
        let mut char_count = 0;
        let mut is_truncated = false;
        let len = html.len();

        while i < len {
            if char_count >= max_len {
                is_truncated = true;
                break;
            }

            let slice = &html[i..];

            // 1. Skip HTML Comments <!-- ... -->
            if slice.starts_with("<!--") {
                if let Some(end_pos) = slice.find("-->") {
                    i += end_pos + 3;
                } else {
                    break;
                }
                continue;
            }

            // 2. Skip <script ...>...</script>
            if slice.starts_with("<script") || slice.starts_with("<SCRIPT") {
                let lower = slice.to_ascii_lowercase();
                if let Some(end_pos) = lower.find("</script>") {
                    i += end_pos + 9;
                } else {
                    break;
                }
                continue;
            }

            // 3. Skip <style ...>...</style>
            if slice.starts_with("<style") || slice.starts_with("<STYLE") {
                let lower = slice.to_ascii_lowercase();
                if let Some(end_pos) = lower.find("</style>") {
                    i += end_pos + 8;
                } else {
                    break;
                }
                continue;
            }

            // 4. Skip generic HTML tags <...>
            if slice.starts_with('<') {
                if let Some(end_pos) = slice.find('>') {
                    i += end_pos + 1;
                    if !result.ends_with(' ') && !result.is_empty() {
                        result.push(' ');
                    }
                } else {
                    break;
                }
                continue;
            }

            // 5. Extract text character
            if let Some(c) = slice.chars().next() {
                i += c.len_utf8();
                if c.is_whitespace() {
                    if !result.ends_with(' ') && !result.is_empty() {
                        result.push(' ');
                        char_count += 1;
                    }
                } else {
                    result.push(c);
                    char_count += 1;
                }
            } else {
                break;
            }
        }

        let trimmed = result.trim().to_string();
        if is_truncated && !trimmed.is_empty() {
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
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_generate_rss_with_reports() -> Layer5Result<()> {
        let reports = vec![
            ReportRssData {
                id: 1,
                html_content: "<div>Báo cáo thị trường crypto ngày hôm nay</div>".to_string(),
                created_at: Utc
                    .with_ymd_and_hms(2025, 11, 23, 7, 0, 0)
                    .single()
                    .ok_or(Layer5Error::Internal("Invalid date".into()))?,
            },
            ReportRssData {
                id: 2,
                html_content: "<p>Bitcoin tăng mạnh</p>".to_string(),
                created_at: Utc
                    .with_ymd_and_hms(2025, 11, 22, 7, 0, 0)
                    .single()
                    .ok_or(Layer5Error::Internal("Invalid date".into()))?,
            },
        ];

        let xml = RssCreator::generate_rss_xml(&reports)?;

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
        assert!(xml.contains("<image>"));
        assert!(xml.contains("<url>https://cryptodashboard.me/frontend/shared/assets/images/logo.svg</url>"));

        // Verify items
        assert!(xml.contains("<item>"));
        assert!(xml.contains("Báo cáo Thị trường Crypto #1"));
        assert!(xml.contains("Báo cáo Thị trường Crypto #2"));
        assert!(xml.contains("<link>https://cryptodashboard.me/crypto_report/1</link>"));
        assert!(xml.contains("<guid isPermaLink=\"true\">"));
        assert!(xml.contains("<category>Báo cáo thị trường</category>"));
        assert!(xml.contains("<category>Cryptocurrency</category>"));

        Ok(())
    }

    #[test]
    fn test_generate_rss_empty_reports() -> Layer5Result<()> {
        let xml = RssCreator::generate_rss_xml(&[])?;

        // Should still have valid channel
        assert!(xml.contains("<channel>"));
        assert!(xml.contains("<title>CryptoDashboard"));
        assert!(xml.contains("<image>"));
        assert!(!xml.contains("<item>"));

        Ok(())
    }

    #[test]
    fn test_format_rfc822_date() -> Result<(), Box<dyn std::error::Error>> {
        let dt = Utc
            .with_ymd_and_hms(2025, 11, 23, 7, 0, 0)
            .single()
            .ok_or("Invalid date")?;
        let formatted = RssCreator::format_rfc822_date(&dt);

        // Should be in format: "Sun, 23 Nov 2025 14:00:00 +0700" (converted to VN timezone)
        assert!(formatted.contains("Nov 2025"));
        assert!(formatted.contains("+0700"));
        Ok(())
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
    fn test_extract_description_filters_script_style_comments() {
        let html = r#"
            <div>
                <style>body { color: red; } .hidden { display: none; }</style>
                <script>console.log("secret tracker"); const x = 10;</script>
                <!-- This is a developer comment that should not appear -->
                <h1>Báo cáo thị trường Crypto</h1>
                <p>Bitcoin và Ethereum ghi nhận đà tăng trưởng mạnh mẽ trong 24h qua.</p>
            </div>
        "#;
        let desc = RssCreator::extract_description(html, 300);

        assert!(!desc.contains("body {"));
        assert!(!desc.contains("console.log"));
        assert!(!desc.contains("developer comment"));
        assert!(desc.contains("Báo cáo thị trường Crypto"));
        assert!(desc.contains("Bitcoin và Ethereum"));
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
