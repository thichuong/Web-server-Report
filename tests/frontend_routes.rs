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
async fn test_homepage() {
    let app = get_app().await.expect("Failed to initialize app");

    let response = app
        .oneshot(
            Request::builder()
                .uri("/")
                .body(Body::empty())
                .expect("Failed to build request"),
        )
        .await
        .expect("Failed to get response");

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
#[ignore = "requires running database and Redis"]
async fn test_crypto_index() {
    let app = get_app().await.expect("Failed to initialize app");

    let response = app
        .oneshot(
            Request::builder()
                .uri("/crypto_report")
                .body(Body::empty())
                .expect("Failed to build request"),
        )
        .await
        .expect("Failed to get response");

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
#[ignore = "requires running database and Redis"]
async fn test_crypto_reports_list() {
    let app = get_app().await.expect("Failed to initialize app");

    let response = app
        .oneshot(
            Request::builder()
                .uri("/crypto_reports_list")
                .body(Body::empty())
                .expect("Failed to build request"),
        )
        .await
        .expect("Failed to get response");

    assert_eq!(response.status(), StatusCode::OK);
}
