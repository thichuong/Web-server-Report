//! Utility Functions Component
//! 
//! Provides common utility functions used across the application.
//! These functions handle data formatting, validation, and other shared operations.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use anyhow::Result;
use serde_json;

/// Utility Functions provides shared helper functions
pub struct UtilityFunctions {
    /// Function execution count
    execution_count: Arc<AtomicUsize>,
    /// Available functions registry
    available_functions: HashMap<String, String>,
}

impl UtilityFunctions {
    /// Initialize the Utility Functions component
    pub async fn new() -> Result<Self> {
        println!("🛠️  Initializing Utility Functions...");
        
        let available_functions = Self::register_functions().await?;
        
        println!("  🛠️  Registered {} utility functions", available_functions.len());
        
        Ok(Self {
            execution_count: Arc::new(AtomicUsize::new(0)),
            available_functions,
        })
    }
    
    /// Register all available utility functions
    async fn register_functions() -> Result<HashMap<String, String>> {
        let mut functions = HashMap::new();
        
        // Data formatting functions
        functions.insert("format_currency".to_string(), "Format number as currency ($1,234.56)".to_string());
        functions.insert("format_percentage".to_string(), "Format decimal as percentage (0.1234 -> 12.34%)".to_string());
        functions.insert("format_number".to_string(), "Format number with thousands separators".to_string());
        functions.insert("format_date".to_string(), "Format date string to various formats".to_string());
        
        // Validation functions
        functions.insert("validate_email".to_string(), "Validate email address format".to_string());
        functions.insert("validate_json".to_string(), "Validate JSON string format".to_string());
        functions.insert("validate_number".to_string(), "Validate number format and range".to_string());
        
        // Data processing functions
        functions.insert("calculate_percentage_change".to_string(), "Calculate percentage change between two values".to_string());
        functions.insert("generate_report_id".to_string(), "Generate unique report identifier".to_string());
        functions.insert("sanitize_html".to_string(), "Sanitize HTML content for security".to_string());
        functions.insert("truncate_text".to_string(), "Truncate text to specified length".to_string());
        
        // Crypto-specific functions
        functions.insert("format_crypto_amount".to_string(), "Format cryptocurrency amounts with proper decimals".to_string());
        functions.insert("calculate_portfolio_value".to_string(), "Calculate total portfolio value".to_string());
        
        Ok(functions)
    }
    
    /// Execute a utility function by name
    pub async fn execute_function(&self, function_name: &str, args: serde_json::Value) -> Result<serde_json::Value> {
        self.execution_count.fetch_add(1, Ordering::Relaxed);
        
        match function_name {
            "format_currency" => self.format_currency(args).await,
            "format_percentage" => self.format_percentage(args).await,
            "format_number" => self.format_number(args).await,
            "format_date" => self.format_date(args).await,
            "validate_email" => self.validate_email(args).await,
            "validate_json" => self.validate_json(args).await,
            "validate_number" => self.validate_number(args).await,
            "calculate_percentage_change" => self.calculate_percentage_change(args).await,
            "generate_report_id" => self.generate_report_id(args).await,
            "sanitize_html" => self.sanitize_html(args).await,
            "truncate_text" => self.truncate_text(args).await,
            "format_crypto_amount" => self.format_crypto_amount(args).await,
            "calculate_portfolio_value" => self.calculate_portfolio_value(args).await,
            _ => Err(anyhow::anyhow!("Unknown function: {}", function_name))
        }
    }
    
    // Utility function implementations
    
    /// Format number as currency
    async fn format_currency(&self, args: serde_json::Value) -> Result<serde_json::Value> {
        let value = args.get("value").and_then(|v| v.as_f64())
            .ok_or_else(|| anyhow::anyhow!("Missing or invalid 'value' parameter"))?;
        let symbol = args.get("symbol").and_then(|v| v.as_str()).unwrap_or("$");
        
        Ok(serde_json::json!({
            "result": format!("{}{:.2}", symbol, value)
        }))
    }
    
    /// Format decimal as percentage
    async fn format_percentage(&self, args: serde_json::Value) -> Result<serde_json::Value> {
        let value = args.get("value").and_then(|v| v.as_f64())
            .ok_or_else(|| anyhow::anyhow!("Missing or invalid 'value' parameter"))?;
        let decimals = args.get("decimals").and_then(|v| v.as_u64()).unwrap_or(2) as usize;
        
        Ok(serde_json::json!({
            "result": format!("{:.1$}%", value * 100.0, decimals)
        }))
    }
    
    /// Format number with thousands separators
    async fn format_number(&self, args: serde_json::Value) -> Result<serde_json::Value> {
        let value = args.get("value").and_then(|v| v.as_f64())
            .ok_or_else(|| anyhow::anyhow!("Missing or invalid 'value' parameter"))?;
        
        // Simple thousands separator formatting
        let formatted = if value >= 1_000_000.0 {
            format!("{:.1}M", value / 1_000_000.0)
        } else if value >= 1_000.0 {
            format!("{:.1}K", value / 1_000.0)
        } else {
            format!("{:.2}", value)
        };
        
        Ok(serde_json::json!({
            "result": formatted
        }))
    }
    
    /// Format date string
    async fn format_date(&self, args: serde_json::Value) -> Result<serde_json::Value> {
        let date_str = args.get("date").and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing or invalid 'date' parameter"))?;
        let format = args.get("format").and_then(|v| v.as_str()).unwrap_or("Y-m-d");
        
        // Simple date formatting - in real implementation, use chrono
        Ok(serde_json::json!({
            "result": format!("Formatted({}) {}", format, date_str)
        }))
    }
    
    /// Validate email address
    async fn validate_email(&self, args: serde_json::Value) -> Result<serde_json::Value> {
        let email = args.get("email").and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing or invalid 'email' parameter"))?;
        
        let is_valid = email.contains('@') && email.contains('.') && email.len() > 5;
        
        Ok(serde_json::json!({
            "result": is_valid
        }))
    }
    
    /// Validate JSON string
    async fn validate_json(&self, args: serde_json::Value) -> Result<serde_json::Value> {
        let json_str = args.get("json").and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing or invalid 'json' parameter"))?;
        
        let is_valid = serde_json::from_str::<serde_json::Value>(json_str).is_ok();
        
        Ok(serde_json::json!({
            "result": is_valid
        }))
    }
    
    /// Validate number
    async fn validate_number(&self, args: serde_json::Value) -> Result<serde_json::Value> {
        let value = args.get("value").and_then(|v| v.as_f64());
        let min = args.get("min").and_then(|v| v.as_f64());
        let max = args.get("max").and_then(|v| v.as_f64());
        
        if let Some(num) = value {
            let is_valid = match (min, max) {
                (Some(min_val), Some(max_val)) => num >= min_val && num <= max_val,
                (Some(min_val), None) => num >= min_val,
                (None, Some(max_val)) => num <= max_val,
                (None, None) => true,
            };
            
            Ok(serde_json::json!({
                "result": is_valid
            }))
        } else {
            Ok(serde_json::json!({
                "result": false
            }))
        }
    }
    
    /// Calculate percentage change
    async fn calculate_percentage_change(&self, args: serde_json::Value) -> Result<serde_json::Value> {
        let old_value = args.get("old_value").and_then(|v| v.as_f64())
            .ok_or_else(|| anyhow::anyhow!("Missing or invalid 'old_value' parameter"))?;
        let new_value = args.get("new_value").and_then(|v| v.as_f64())
            .ok_or_else(|| anyhow::anyhow!("Missing or invalid 'new_value' parameter"))?;
        
        let change = if old_value != 0.0 {
            ((new_value - old_value) / old_value) * 100.0
        } else {
            0.0
        };
        
        Ok(serde_json::json!({
            "result": change
        }))
    }
    
    /// Generate report ID
    async fn generate_report_id(&self, _args: serde_json::Value) -> Result<serde_json::Value> {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs();
        
        let report_id = format!("RPT-{}-{:04}", timestamp, rand::random::<u16>() % 10000);
        
        Ok(serde_json::json!({
            "result": report_id
        }))
    }
    
    /// Sanitize HTML content
    async fn sanitize_html(&self, args: serde_json::Value) -> Result<serde_json::Value> {
        let html = args.get("html").and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing or invalid 'html' parameter"))?;
        
        // Basic HTML sanitization - remove script tags and dangerous attributes
        let sanitized = html
            .replace("<script", "&lt;script")
            .replace("</script>", "&lt;/script&gt;")
            .replace("javascript:", "")
            .replace("on", "data-on");
        
        Ok(serde_json::json!({
            "result": sanitized
        }))
    }
    
    /// Truncate text
    async fn truncate_text(&self, args: serde_json::Value) -> Result<serde_json::Value> {
        let text = args.get("text").and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing or invalid 'text' parameter"))?;
        let max_length = args.get("max_length").and_then(|v| v.as_u64()).unwrap_or(100) as usize;
        
        let truncated = if text.len() > max_length {
            format!("{}...", &text[..max_length.saturating_sub(3)])
        } else {
            text.to_string()
        };
        
        Ok(serde_json::json!({
            "result": truncated
        }))
    }
    
    /// Format cryptocurrency amount
    async fn format_crypto_amount(&self, args: serde_json::Value) -> Result<serde_json::Value> {
        let amount = args.get("amount").and_then(|v| v.as_f64())
            .ok_or_else(|| anyhow::anyhow!("Missing or invalid 'amount' parameter"))?;
        let symbol = args.get("symbol").and_then(|v| v.as_str()).unwrap_or("BTC");
        
        let formatted = match symbol {
            "BTC" => format!("{:.8} {}", amount, symbol),
            "ETH" => format!("{:.6} {}", amount, symbol),
            _ => format!("{:.4} {}", amount, symbol),
        };
        
        Ok(serde_json::json!({
            "result": formatted
        }))
    }
    
    /// Calculate portfolio value
    async fn calculate_portfolio_value(&self, args: serde_json::Value) -> Result<serde_json::Value> {
        let holdings = args.get("holdings").and_then(|v| v.as_array())
            .ok_or_else(|| anyhow::anyhow!("Missing or invalid 'holdings' parameter"))?;
        
        let mut total_value = 0.0;
        
        for holding in holdings {
            if let (Some(amount), Some(price)) = (
                holding.get("amount").and_then(|v| v.as_f64()),
                holding.get("price").and_then(|v| v.as_f64())
            ) {
                total_value += amount * price;
            }
        }
        
        Ok(serde_json::json!({
            "result": total_value
        }))
    }
    
    /// Health check for utility functions
    pub async fn health_check(&self) -> bool {
        // Test a few core functions
        let test_args = serde_json::json!({
            "value": 1234.56
        });
        
        match self.format_currency(test_args).await {
            Ok(_) => {
                println!("✅ Utility Functions health check passed");
                true
            },
            Err(e) => {
                eprintln!("❌ Utility Functions health check failed: {}", e);
                false
            }
        }
    }
    
    /// Get utility functions statistics
    pub async fn get_statistics(&self) -> Result<serde_json::Value> {
        let total_executions = self.execution_count.load(Ordering::Relaxed);
        
        Ok(serde_json::json!({
            "component": "utility_functions",
            "total_executions": total_executions,
            "available_functions": self.available_functions.len(),
            "functions": self.available_functions
        }))
    }
    
    /// List all available functions
    pub async fn list_functions(&self) -> Vec<String> {
        self.available_functions.keys().cloned().collect()
    }
}
