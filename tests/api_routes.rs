use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt; // for `oneshot`

use std::sync::Arc;
use web_server_report::routes::create_service_islands_router;
use web_server_report::ServiceIslands;

#[tokio::test]
#[ignore = "requires running database and Redis"]
async fn test_homepage_route() -> Result<(), Box<dyn std::error::Error>> {
    // This test requires a running database and Redis instance, so it is ignored by default.
    // To run it, ensure services are up and set env vars, then run `cargo test -- --ignored`

    // Initialize Service Islands (might fail if services are down)
    let service_islands = ServiceIslands::initialize()
        .map_err(|e| format!("Failed to initialize Service Islands: {e}"))?;
    let app = create_service_islands_router(Arc::new(service_islands));

    // Create a request
    let request = Request::builder().uri("/").body(Body::empty())?;

    // Call the service
    let response = app
        .oneshot(request)
        .await
        .map_err(|e| format!("Service oneshot failed: {e}"))?;

    // Assertions
    assert_eq!(response.status(), StatusCode::OK);

    // Check if response is compressed
    // Note: Depends on whether `create_compressed_response` is called which depends on logic inside handler
    Ok(())
}
