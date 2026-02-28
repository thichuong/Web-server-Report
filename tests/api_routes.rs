#![allow(clippy::expect_used)]

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use std::sync::Arc;
use tower::ServiceExt; // for `oneshot`
use web_server_report::routes::create_router;
use web_server_report::state::AppState;

async fn get_app() -> Option<axum::Router> {
    dotenvy::dotenv().ok();
    let state = AppState::new().await.ok()?;
    Some(create_router(Arc::new(state)))
}

#[tokio::test]
#[ignore = "requires running database and Redis"]
async fn test_health_check() {
    let app = get_app().await.expect("Failed to initialize app");

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/health")
                .body(Body::empty())
                .expect("Failed to build request"),
        )
        .await
        .expect("Failed to get response");

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
#[ignore = "requires running database and Redis"]
async fn test_dashboard_data() {
    let app = get_app().await.expect("Failed to initialize app");

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/dashboard/data")
                .body(Body::empty())
                .expect("Failed to build request"),
        )
        .await
        .expect("Failed to get response");

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
#[ignore = "requires running database and Redis"]
async fn test_crypto_dashboard_summary() {
    let app = get_app().await.expect("Failed to initialize app");

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/crypto/dashboard-summary")
                .body(Body::empty())
                .expect("Failed to build request"),
        )
        .await
        .expect("Failed to get response");

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
#[ignore = "requires running database and Redis"]
async fn test_websocket_stats_redirect() {
    let app = get_app().await.expect("Failed to initialize app");

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/websocket/stats")
                .body(Body::empty())
                .expect("Failed to build request"),
        )
        .await
        .expect("Failed to get response");

    assert_eq!(response.status(), StatusCode::OK);
}
