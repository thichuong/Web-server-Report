use serde::Serialize;
use sqlx::FromRow;

#[derive(FromRow, Serialize, Debug, Clone)]
pub struct Report {
    pub id: i32,
    pub html_content: String,
    pub css_content: Option<String>,
    pub js_content: Option<String>,
    pub html_content_en: Option<String>,
    pub js_content_en: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(FromRow, Serialize)]
pub struct ReportSummary {
    pub id: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize)]
pub struct ReportListItem {
    pub id: i32,
    pub created_date: String,
    pub created_time: String,
}
