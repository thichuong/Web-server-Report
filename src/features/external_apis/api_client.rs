// src/features/external_apis/api_client.rs
//
// Generic HTTP client with retry logic and error handling

use anyhow::{Context, Result};
use reqwest::Client;
use serde::de::DeserializeOwned;
use std::time::{Duration, Instant};

use super::models::{ApiRequestContext, ApiResponse};
use super::rate_limiter::RateLimiter;

/// Generic API client with built-in retry logic and rate limiting
#[derive(Debug, Clone)]
pub struct ApiClient {
    client: Client,
    rate_limiter: RateLimiter,
}

impl ApiClient {
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("Mozilla/5.0 (compatible; RustWebServer/1.0)")
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client,
            rate_limiter: RateLimiter::new(),
        })
    }

    /// Perform GET request with automatic retry and rate limiting
    pub async fn get_with_retry<T>(
        &self,
        url: &str,
        endpoint_name: &str,
        max_retries: u32,
    ) -> Result<ApiResponse<T>>
    where
        T: DeserializeOwned,
    {
        let start_time = Instant::now();
        let mut attempt = 1;

        loop {
            // Rate limiting check
            self.rate_limiter.wait_if_needed(endpoint_name).await?;

            let context = ApiRequestContext {
                endpoint: endpoint_name.to_string(),
                attempt,
                max_retries,
                backoff_seconds: Self::calculate_backoff(attempt),
            };

            match self.perform_get_request::<T>(url, &context).await {
                Ok(data) => {
                    let response_time = start_time.elapsed().as_millis() as u64;
                    return Ok(ApiResponse {
                        success: true,
                        data: Some(data),
                        error: None,
                        response_time_ms: response_time,
                        from_cache: false,
                        timestamp: chrono::Utc::now(),
                    });
                }
                Err(e) => {
                    println!("❌ API request failed (attempt {}): {}", attempt, e);
                    
                    if attempt >= max_retries {
                        let response_time = start_time.elapsed().as_millis() as u64;
                        return Ok(ApiResponse {
                            success: false,
                            data: None,
                            error: Some(e.to_string()),
                            response_time_ms: response_time,
                            from_cache: false,
                            timestamp: chrono::Utc::now(),
                        });
                    }

                    // Handle rate limiting specially
                    if e.to_string().contains("429") || e.to_string().contains("Too Many Requests") {
                        let wait_time = Duration::from_secs(60); // Wait 1 minute for rate limits
                        println!("⏳ Rate limited, waiting {:?} before retry", wait_time);
                        tokio::time::sleep(wait_time).await;
                    } else {
                        let backoff = Duration::from_secs(Self::calculate_backoff(attempt));
                        println!("⏳ Waiting {:?} before retry {}", backoff, attempt + 1);
                        tokio::time::sleep(backoff).await;
                    }

                    attempt += 1;
                }
            }
        }
    }

    /// Perform a single GET request
    async fn perform_get_request<T>(&self, url: &str, context: &ApiRequestContext) -> Result<T>
    where
        T: DeserializeOwned,
    {
        println!("🔄 API request to {} (attempt {})", context.endpoint, context.attempt);

        let response = self
            .client
            .get(url)
            .header("Accept", "application/json")
            .send()
            .await
            .context("Network request failed")?;

        // Check response status
        if !response.status().is_success() {
            anyhow::bail!("API returned error status: {}", response.status());
        }

        let data: T = response
            .json()
            .await
            .context("Failed to parse JSON response")?;

        Ok(data)
    }

    /// Calculate exponential backoff delay
    fn calculate_backoff(attempt: u32) -> u64 {
        std::cmp::min(2_u64.pow(attempt.saturating_sub(1)), 60) // Max 60 seconds
    }

    /// Get rate limiter for external monitoring
    pub fn rate_limiter(&self) -> &RateLimiter {
        &self.rate_limiter
    }
}
