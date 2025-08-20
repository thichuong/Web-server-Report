// src/features/shared_components/utils/mod.rs
//
// Shared utility functions used across multiple features
// This module provides common functionality that doesn't belong to any specific business domain

pub mod chart_modules;

// Re-export commonly used functions
pub use chart_modules::{
    get_chart_modules_content,
    load_chart_modules_with_config, 
    ChartModulesConfig
};

/// Common utility functions for string manipulation
pub mod string_utils {
    /// Escape HTML special characters to prevent XSS
    pub fn escape_html(input: &str) -> String {
        input
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#x27;")
    }
    
    /// Truncate string to specified length with ellipsis
    pub fn truncate(input: &str, max_len: usize) -> String {
        if input.len() <= max_len {
            input.to_string()
        } else {
            format!("{}...", &input[..max_len.saturating_sub(3)])
        }
    }
    
    /// Convert snake_case to camelCase
    pub fn snake_to_camel(input: &str) -> String {
        let mut result = String::new();
        let mut capitalize_next = false;
        
        for ch in input.chars() {
            if ch == '_' {
                capitalize_next = true;
            } else if capitalize_next {
                result.push(ch.to_uppercase().next().unwrap_or(ch));
                capitalize_next = false;
            } else {
                result.push(ch);
            }
        }
        
        result
    }
}

/// Common utility functions for date/time handling
pub mod date_utils {
    use chrono::{DateTime, Utc, Duration};
    
    /// Format timestamp for display in Vietnam timezone (UTC+7)
    pub fn format_vietnam_time(dt: DateTime<Utc>) -> String {
        (dt + Duration::hours(7))
            .format("%d-%m-%Y %H:%M")
            .to_string()
    }
    
    /// Split Vietnam formatted time into date and time components
    pub fn split_vietnam_time(dt: DateTime<Utc>) -> (String, String) {
        let formatted = format_vietnam_time(dt);
        let parts: Vec<&str> = formatted.split(' ').collect();
        (
            parts.get(0).unwrap_or(&"").to_string(),
            parts.get(1).unwrap_or(&"").to_string(),
        )
    }
    
    /// Get relative time string (e.g., "2 hours ago")
    pub fn relative_time(dt: DateTime<Utc>) -> String {
        let now = Utc::now();
        let diff = now - dt;
        
        if diff.num_days() > 0 {
            format!("{} ngày trước", diff.num_days())
        } else if diff.num_hours() > 0 {
            format!("{} giờ trước", diff.num_hours())
        } else if diff.num_minutes() > 0 {
            format!("{} phút trước", diff.num_minutes())
        } else {
            "Vài giây trước".to_string()
        }
    }
}

/// Common utility functions for number formatting
pub mod number_utils {
    /// Format large numbers with appropriate suffixes (K, M, B)
    pub fn format_large_number(value: f64) -> String {
        if value >= 1_000_000_000.0 {
            format!("{:.1}B", value / 1_000_000_000.0)
        } else if value >= 1_000_000.0 {
            format!("{:.1}M", value / 1_000_000.0)
        } else if value >= 1_000.0 {
            format!("{:.1}K", value / 1_000.0)
        } else {
            format!("{:.0}", value)
        }
    }
    
    /// Format currency with appropriate commas and decimals
    pub fn format_currency(value: f64, symbol: &str) -> String {
        format!("{}{:.2}", symbol, value)
            .chars()
            .rev()
            .enumerate()
            .flat_map(|(i, c)| {
                if i != 0 && i % 3 == 0 && c.is_ascii_digit() {
                    vec![c, ',']
                } else {
                    vec![c]
                }
            })
            .rev()
            .collect()
    }
    
    /// Format percentage with + or - sign
    pub fn format_percentage(value: f64) -> String {
        if value >= 0.0 {
            format!("+{:.2}%", value)
        } else {
            format!("{:.2}%", value)
        }
    }
}

/// Error handling utilities
pub mod error_utils {
    /// Convert any error to user-friendly message
    pub fn user_friendly_error(error: &dyn std::error::Error) -> String {
        match error.to_string().as_str() {
            msg if msg.contains("connection") => "Lỗi kết nối mạng. Vui lòng thử lại.".to_string(),
            msg if msg.contains("timeout") => "Hết thời gian chờ. Vui lòng thử lại.".to_string(),
            msg if msg.contains("not found") => "Không tìm thấy dữ liệu yêu cầu.".to_string(),
            _ => "Đã xảy ra lỗi. Vui lòng thử lại sau.".to_string(),
        }
    }
}

/// Validation utilities
pub mod validation_utils {
    /// Validate report ID is positive integer
    pub fn validate_report_id(id: i32) -> Result<i32, String> {
        if id > 0 {
            Ok(id)
        } else {
            Err("ID báo cáo phải là số dương".to_string())
        }
    }
    
    /// Validate pagination parameters
    pub fn validate_pagination(page: i64, per_page: i64) -> Result<(i64, i64), String> {
        if page < 1 {
            return Err("Trang phải từ 1 trở lên".to_string());
        }
        if per_page < 1 || per_page > 100 {
            return Err("Số item mỗi trang phải từ 1-100".to_string());
        }
        Ok((page, per_page))
    }
}
