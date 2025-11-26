//! Breadcrumbs Module for GEO Optimization
//!
//! Generates structured breadcrumb navigation and JSON-LD schema for:
//! - Internal linking optimization
//! - AI bots understanding (Grok, GPT, Claude)
//! - Search engine crawling
//!
//! Part of Layer 5 Business Logic - Rendering strategies

use crate::service_islands::layer3_communication::data_communication::ReportSummaryData;
use serde::{Deserialize, Serialize};

/// Base URL for the website (used in JSON-LD schema)
const BASE_URL: &str = "https://cryptodashboard.io";

/// Related report data for template rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedReportItem {
    pub id: i32,
    pub created_at: String,
    pub created_date_display: String,
    pub created_time_display: String,
    pub url: String,
}

/// Breadcrumb item for navigation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreadcrumbItem {
    pub name: String,
    pub name_vi: String,
    pub name_en: String,
    pub url: String,
    pub is_current: bool,
}

/// Generate breadcrumb items for a crypto report page
///
/// Creates a hierarchical breadcrumb trail:
/// Home > Crypto Reports > Report #ID
#[must_use] 
pub fn generate_breadcrumb_items(report_id: i32) -> Vec<BreadcrumbItem> {
    vec![
        BreadcrumbItem {
            name: "Trang chu".to_string(),
            name_vi: "Trang chu".to_string(),
            name_en: "Home".to_string(),
            url: "/".to_string(),
            is_current: false,
        },
        BreadcrumbItem {
            name: "Bao cao".to_string(),
            name_vi: "Bao cao".to_string(),
            name_en: "Reports".to_string(),
            url: "/crypto_reports_list".to_string(),
            is_current: false,
        },
        BreadcrumbItem {
            name: format!("Report #{report_id}"),
            name_vi: format!("Bao cao #{report_id}"),
            name_en: format!("Report #{report_id}"),
            url: format!("/crypto_report/{report_id}"),
            is_current: true,
        },
    ]
}

/// Generate JSON-LD `BreadcrumbList` schema for SEO
///
/// Creates Schema.org `BreadcrumbList` structured data that helps:
/// - Search engines understand page hierarchy
/// - AI bots navigate site structure
/// - Rich snippets in search results
#[must_use] 
pub fn generate_breadcrumbs_schema(report_id: i32) -> String {
    let items = generate_breadcrumb_items(report_id);

    let list_elements: Vec<serde_json::Value> = items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            serde_json::json!({
                "@type": "ListItem",
                "position": index + 1,
                "name": item.name_en,
                "item": format!("{}{}", BASE_URL, item.url)
            })
        })
        .collect();

    let schema = serde_json::json!({
        "@context": "https://schema.org",
        "@type": "BreadcrumbList",
        "itemListElement": list_elements
    });

    format!(
        r#"<script type="application/ld+json">
{}
</script>"#,
        serde_json::to_string_pretty(&schema).unwrap_or_default()
    )
}

/// Convert `ReportSummaryData` to `RelatedReportItem` for template
///
/// Formats dates to Vietnam timezone (UTC+7) for display
#[must_use] 
pub fn format_related_report(report: &ReportSummaryData) -> RelatedReportItem {
    let dt = report.created_at + chrono::Duration::hours(7);

    RelatedReportItem {
        id: report.id,
        created_at: report.created_at.to_rfc3339(),
        created_date_display: dt.format("%d/%m/%Y").to_string(),
        created_time_display: format!("{} UTC+7", dt.format("%H:%M")),
        url: format!("/crypto_report/{}", report.id),
    }
}

/// Format multiple related reports for template rendering
pub fn format_related_reports(reports: &[ReportSummaryData]) -> Vec<RelatedReportItem> {
    reports.iter().map(format_related_report).collect()
}

/// Generate complete breadcrumbs and related reports data for template
///
/// Returns a tuple containing:
/// - `breadcrumb_items`: Vec for navigation rendering
/// - `breadcrumbs_schema`: JSON-LD string for SEO
/// - `related_reports`: Vec for related content section
#[must_use] 
pub fn generate_breadcrumbs_and_related(
    report_id: i32,
    related_reports_data: &[ReportSummaryData],
) -> (Vec<BreadcrumbItem>, String, Vec<RelatedReportItem>) {
    let breadcrumb_items = generate_breadcrumb_items(report_id);
    let breadcrumbs_schema = generate_breadcrumbs_schema(report_id);
    let related_reports = format_related_reports(related_reports_data);

    (breadcrumb_items, breadcrumbs_schema, related_reports)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_breadcrumb_items() {
        let items = generate_breadcrumb_items(123);
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].url, "/");
        assert_eq!(items[1].url, "/crypto_reports_list");
        assert_eq!(items[2].url, "/crypto_report/123");
        assert!(items[2].is_current);
    }

    #[test]
    fn test_generate_breadcrumbs_schema() {
        let schema = generate_breadcrumbs_schema(456);
        assert!(schema.contains("BreadcrumbList"));
        assert!(schema.contains("Report #456"));
        assert!(schema.contains("https://cryptodashboard.io"));
    }
}
