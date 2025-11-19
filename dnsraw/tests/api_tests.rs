// Integration tests for the API endpoints
use actix_web::{App, test};
use serde_json::json;

/// Tests the PUT /blocklist endpoint with a mock HTTP server
///
/// Teaching moment: We use mockito to create a fake blocklist server
/// so we don't depend on external services in tests
#[tokio::test]
async fn test_update_blocklist_success() {
    // Step 1: Create a mock HTTP server
    // This simulates a real blocklist server without hitting the internet
    let mut server = mockito::Server::new_async().await;

    // Mock response: a simple blocklist with 3 domains
    let mock_blocklist = "||ads.example.com^\n||tracker.test.com^\n||malware.bad.com^";

    let mock = server
        .mock("GET", "/blocklist.txt")
        .with_status(200)
        .with_header("content-type", "text/plain")
        .with_body(mock_blocklist)
        .create_async()
        .await;

    // Step 2: Create the actix-web test app
    // This is like running the real server, but in-memory for testing
    let app =
        test::init_service(App::new().service(dnsraw::api::update_blocklist_endpoint())).await;

    // Step 3: Make a test HTTP request
    let mock_url = format!("{}/blocklist.txt", server.url());
    let req = test::TestRequest::put()
        .uri("/blocklist")
        .set_json(json!({
            "url": mock_url
        }))
        .to_request();

    // Step 4: Send the request and get response
    let resp = test::call_service(&app, req).await;

    // Step 5: Assert the response
    assert_eq!(resp.status(), 200, "Should return 200 OK");

    // Parse response body
    let body: serde_json::Value = test::read_body_json(resp).await;

    // Teaching moment: Assert on the structure of the response
    assert_eq!(body["message"], "Blocklist updated successfully");
    assert_eq!(body["url"], mock_url);
    assert_eq!(body["domains_loaded"], 3, "Should have loaded 3 domains");

    // Verify the mock was called
    mock.assert_async().await;
}

/// Tests that invalid URLs are rejected
///
/// Teaching moment: Testing error cases is as important as testing success!
#[tokio::test]
async fn test_update_blocklist_invalid_url() {
    let app =
        test::init_service(App::new().service(dnsraw::api::update_blocklist_endpoint())).await;

    let req = test::TestRequest::put()
        .uri("/blocklist")
        .set_json(json!({
            "url": "not-a-valid-url"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Should return 400 Bad Request
    assert_eq!(resp.status(), 400, "Should return 400 for invalid URL");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(
        body["error"]
            .as_str()
            .unwrap()
            .contains("Invalid URL format")
    );
}

/// Tests that network failures are handled gracefully
#[tokio::test]
async fn test_update_blocklist_download_failure() {
    // Create a mock server that returns 404
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("GET", "/nonexistent.txt")
        .with_status(404)
        .create_async()
        .await;

    let app =
        test::init_service(App::new().service(dnsraw::api::update_blocklist_endpoint())).await;

    let mock_url = format!("{}/nonexistent.txt", server.url());
    let req = test::TestRequest::put()
        .uri("/blocklist")
        .set_json(json!({
            "url": mock_url
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Should return error (400 or 500)
    assert!(
        resp.status().is_client_error() || resp.status().is_server_error(),
        "Should return error for failed download"
    );

    mock.assert_async().await;
}

/// Tests empty blocklist
#[tokio::test]
async fn test_update_blocklist_empty() {
    let mut server = mockito::Server::new_async().await;

    // Empty blocklist
    let mock = server
        .mock("GET", "/empty.txt")
        .with_status(200)
        .with_body("")
        .create_async()
        .await;

    let app =
        test::init_service(App::new().service(dnsraw::api::update_blocklist_endpoint())).await;

    let mock_url = format!("{}/empty.txt", server.url());
    let req = test::TestRequest::put()
        .uri("/blocklist")
        .set_json(json!({
            "url": mock_url
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(
        body["domains_loaded"], 0,
        "Empty blocklist should load 0 domains"
    );

    mock.assert_async().await;
}

/// Tests malformed JSON request
#[tokio::test]
async fn test_update_blocklist_malformed_json() {
    let app =
        test::init_service(App::new().service(dnsraw::api::update_blocklist_endpoint())).await;

    // Send request with wrong field name
    let req = test::TestRequest::put()
        .uri("/blocklist")
        .set_json(json!({
            "wrong_field": "https://example.com"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // actix-web should return 400 for deserialization errors
    assert_eq!(resp.status(), 400, "Should return 400 for malformed JSON");
}
