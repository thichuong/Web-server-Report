//! Safe HTTP Response Builder Utilities
//!
//! Provides safe response construction without panic-inducing .`unwrap()` calls.
//! All response builders are guaranteed to succeed or return a safe fallback.

use axum::{
    body::Body,
    http::StatusCode,
    response::{IntoResponse, Response},
};

/// Cache control header values
pub mod cache_control {
    /// Short cache for dynamic content (15 seconds)
    pub const SHORT: &str = "public, max-age=15";
    /// Private cache for user-specific content (1 hour)
    pub const PRIVATE_LONG: &str = "private, max-age=3600";
    /// No cache for real-time data
    pub const NO_CACHE: &str = "no-cache, no-store, must-revalidate";
}

/// Common security headers for responses
pub mod security_headers {
    /// Content Security Policy for sandboxed content
    pub const CSP_SANDBOX: &str = "default-src 'self' 'unsafe-inline'; \
        script-src 'self' 'unsafe-inline' https://cdnjs.cloudflare.com https://fonts.googleapis.com https://cdn.tailwindcss.com; \
        style-src 'self' 'unsafe-inline' https://cdnjs.cloudflare.com https://fonts.googleapis.com; \
        font-src 'self' https://cdnjs.cloudflare.com https://fonts.gstatic.com; \
        img-src 'self' data: https:; connect-src 'self'";
}

/// Build a compressed HTML response with proper headers
///
/// This function is guaranteed to never panic. If response building fails
/// (which should never happen with valid inputs), it returns a safe error response.
#[inline]
#[must_use]
pub fn build_compressed_response(compressed_data: Vec<u8>, cache_status: &str) -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header("cache-control", cache_control::SHORT)
        .header("x-cache", cache_status)
        .header("content-type", "text/html; charset=utf-8")
        .header("content-encoding", "gzip")
        .body(Body::from(compressed_data))
        .unwrap_or_else(|_| fallback_error_response())
        .into_response()
}

/// Build a standard HTML response (uncompressed)
#[inline]
#[must_use]
pub fn build_html_response(html: String, cache_status: &str) -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header("cache-control", cache_control::SHORT)
        .header("x-cache", cache_status)
        .header("content-type", "text/html; charset=utf-8")
        .body(Body::from(html))
        .unwrap_or_else(|_| fallback_error_response())
        .into_response()
}

/// Build an error response with the given status code and message
#[inline]
#[must_use]
pub fn build_error_response(status: StatusCode, message: &str) -> Response {
    Response::builder()
        .status(status)
        .header("content-type", "text/plain; charset=utf-8")
        .body(Body::from(message.to_string()))
        .unwrap_or_else(|_| fallback_error_response())
        .into_response()
}

/// Build a forbidden response (403)
#[inline]
#[must_use]
pub fn build_forbidden_response(message: &str) -> Response {
    build_error_response(StatusCode::FORBIDDEN, message)
}

/// Build a not found response (404)
#[inline]
#[must_use]
pub fn build_not_found_response(message: &str) -> Response {
    build_error_response(StatusCode::NOT_FOUND, message)
}

/// Build a sandboxed HTML response with security headers
#[inline]
#[must_use]
pub fn build_sandboxed_response(html: String) -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "text/html; charset=utf-8")
        .header("x-frame-options", "SAMEORIGIN")
        .header("content-security-policy", security_headers::CSP_SANDBOX)
        .header("x-content-type-options", "nosniff")
        .header("cache-control", cache_control::PRIVATE_LONG)
        .header("access-control-allow-origin", "*")
        .header("access-control-allow-methods", "GET, POST, OPTIONS")
        .header("access-control-allow-headers", "Content-Type")
        .body(Body::from(html))
        .unwrap_or_else(|_| fallback_error_response())
        .into_response()
}

/// Build a Shadow DOM response with appropriate headers
#[inline]
#[must_use]
pub fn build_shadow_dom_response(html: String) -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "text/html; charset=utf-8")
        .header("x-content-type-options", "nosniff")
        .header("cache-control", cache_control::PRIVATE_LONG)
        .header("access-control-allow-origin", "*")
        .header("access-control-allow-methods", "GET, POST, OPTIONS")
        .header("access-control-allow-headers", "Content-Type")
        .body(Body::from(html))
        .unwrap_or_else(|_| fallback_error_response())
        .into_response()
}

/// Fallback error response - guaranteed to never fail
///
/// This is the ultimate fallback that uses only compile-time constants.
#[inline]
fn fallback_error_response() -> Response<Body> {
    // This construction is guaranteed safe because:
    // 1. Status code is a valid enum variant
    // 2. Body is a static string literal
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(Body::from("Internal Server Error"))
        .unwrap_or_else(|_| {
            let mut res = Response::new(Body::from("Internal Server Error"));
            *res.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
            res
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_compressed_response() {
        let data = vec![1, 2, 3, 4];
        let response = build_compressed_response(data, "HIT");
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn test_build_error_response() {
        let response = build_error_response(StatusCode::NOT_FOUND, "Not found");
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
