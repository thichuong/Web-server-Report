use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt; // for `oneshot`

use web_server_report::routes::create_service_islands_router;
use web_server_report::ServiceIslands;
use std::sync::Arc;

#[tokio::test]
#[ignore]
async fn test_homepage_route() {
    // This test requires a running database and Redis instance, so it is ignored by default.
    // To run it, ensure services are up and set env vars, then run `cargo test -- --ignored`
    
    // Initialize Service Islands (might fail if services are down)
    let service_islands = ServiceIslands::initialize().expect("Failed to initialize Service Islands");
    let app = create_service_islands_router(Arc::new(service_islands));

    // Create a request
    let request = Request::builder()
        .uri("/")
        .body(Body::empty())
        .unwrap();

    // Call the service
    let response = app.oneshot(request).await.unwrap();

    // Assertions
    assert_eq!(response.status(), StatusCode::OK);
    
    // Check if response is compressed
    // Note: Depends on whether `create_compressed_response` is called which depends on logic inside handler
}
