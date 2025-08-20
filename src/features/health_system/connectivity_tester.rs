//! Connectivity Tester - SSL and external service connectivity testing
//!
//! Tests connectivity to external services and validates SSL certificates
//! to ensure external API integrations are functioning properly.

use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::timeout;

/// External service connectivity testing
pub struct ConnectivityTester {
    client: reqwest::Client,
    test_endpoints: HashMap<String, String>,
}

impl ConnectivityTester {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .expect("Failed to create HTTP client for connectivity testing");

        let mut test_endpoints = HashMap::new();
        test_endpoints.insert("coingecko".to_string(), "https://api.coingecko.com/api/v3/ping".to_string());
        test_endpoints.insert("fear_greed".to_string(), "https://api.alternative.me/fng/".to_string());
        test_endpoints.insert("taapi".to_string(), "https://api.taapi.io/".to_string());

        Self {
            client,
            test_endpoints,
        }
    }

    /// Test connectivity to all configured external services
    pub async fn test_all_services(&self) -> Value {
        let mut results = HashMap::new();

        for (name, url) in &self.test_endpoints {
            let result = self.test_single_service(name, url).await;
            results.insert(name.clone(), result);
        }

        json!(results)
    }

    /// Test connectivity to a single external service
    pub async fn test_single_service(&self, name: &str, url: &str) -> Value {
        let test_timeout = Duration::from_secs(5);
        
        let result = timeout(test_timeout, async {
            match self.client.get(url).send().await {
                Ok(response) => {
                    let status_code = response.status().as_u16();
                    let is_success = response.status().is_success();
                    
                    json!({
                        "status": if is_success { "healthy" } else { "error" },
                        "status_code": status_code,
                        "response_time_ms": 0, // TODO: Measure actual response time
                        "ssl_valid": true, // Reqwest validates SSL by default
                        "url": url
                    })
                }
                Err(err) => {
                    let error_type = if err.is_timeout() {
                        "timeout"
                    } else if err.is_connect() {
                        "connection_failed"
                    } else if err.is_request() {
                        "request_error"
                    } else {
                        "unknown_error"
                    };

                    json!({
                        "status": "error",
                        "error_type": error_type,
                        "error": err.to_string(),
                        "url": url
                    })
                }
            }
        }).await;

        match result {
            Ok(test_result) => test_result,
            Err(_) => {
                json!({
                    "status": "timeout",
                    "error": "Request timeout (5s)",
                    "url": url
                })
            }
        }
    }

    /// Test SSL connectivity specifically
    pub async fn test_ssl_connectivity(&self) -> Value {
        let mut results = HashMap::new();

        for (name, url) in &self.test_endpoints {
            let result = self.test_ssl_for_service(name, url).await;
            results.insert(name.clone(), result);
        }

        json!(results)
    }

    /// Test SSL connectivity for a specific service
    async fn test_ssl_for_service(&self, name: &str, url: &str) -> Value {
        // For HTTPS URLs, reqwest automatically validates SSL certificates
        if url.starts_with("https://") {
            match timeout(Duration::from_secs(5), self.client.get(url).send()).await {
                Ok(Ok(response)) => {
                    json!({
                        "ssl_valid": true,
                        "status": "connected",
                        "status_code": response.status().as_u16(),
                        "url": url
                    })
                }
                Ok(Err(err)) => {
                    let ssl_error = err.to_string().contains("certificate") || 
                                  err.to_string().contains("SSL") || 
                                  err.to_string().contains("TLS");
                    
                    json!({
                        "ssl_valid": !ssl_error,
                        "status": "error",
                        "error": err.to_string(),
                        "url": url
                    })
                }
                Err(_) => {
                    json!({
                        "ssl_valid": false,
                        "status": "timeout",
                        "error": "Connection timeout",
                        "url": url
                    })
                }
            }
        } else {
            json!({
                "ssl_valid": false,
                "status": "not_https",
                "message": "URL is not HTTPS",
                "url": url
            })
        }
    }

    /// Add a new test endpoint
    pub fn add_test_endpoint(&mut self, name: String, url: String) {
        self.test_endpoints.insert(name, url);
    }

    /// Remove a test endpoint
    pub fn remove_test_endpoint(&mut self, name: &str) -> Option<String> {
        self.test_endpoints.remove(name)
    }

    /// Get all configured test endpoints
    pub fn get_test_endpoints(&self) -> &HashMap<String, String> {
        &self.test_endpoints
    }

    /// Test basic internet connectivity
    pub async fn test_internet_connectivity(&self) -> Value {
        // Test with a reliable endpoint
        let result = self.test_single_service("internet", "https://www.google.com").await;
        
        json!({
            "internet_accessible": result["status"] == "healthy",
            "test_result": result
        })
    }
}

impl Default for ConnectivityTester {
    fn default() -> Self {
        Self::new()
    }
}
