use wi_api::Wi;
use wiremock::{
    matchers::{header, method, path},
    Mock, MockServer, ResponseTemplate,
};

#[test]
fn new_creates_client() {
    // Smoke test: construction and session scoping don't panic
    let wi = Wi::new("test-key");
    let _session = wi.session("my-instance");
}

#[test]
fn builder_accepts_api_key() {
    let wi = Wi::builder("custom-key").build();
    let _session = wi.session("s");
}

#[test]
fn session_clone_is_independent() {
    let wi = Wi::new("k");
    let s1 = wi.session("session-a");
    let s2 = wi.session("session-b");
    // Both sessions are usable; cloning the Wi client is cheap (Arc)
    let _s1b = s1.clone();
    let _s2b = s2.clone();
}

/// Verify that the builder trims a trailing slash on base_url so requests
/// don't produce double slashes like `//sessions/...`.
#[tokio::test]
async fn builder_trims_trailing_slash() {
    let server = MockServer::start().await;

    // MockServer::uri() has no trailing slash — we add one to test trimming
    let url_with_slash = format!("{}/", server.uri());

    Mock::given(method("GET"))
        .and(path("/sessions/s/status"))
        .and(header("x-api-key", "k"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "connected": false,
            "phone": null
        })))
        .expect(1)
        .mount(&server)
        .await;

    let wi = Wi::builder("k").base_url(url_with_slash).build();
    wi.session("s").status().await.unwrap();

    // If the slash wasn't trimmed, the path would be `//sessions/s/status`
    // and wiremock would not match `/sessions/s/status`, causing expect(1) to fail.
    server.verify().await;
}

/// Verify that builder().base_url() directs requests to the custom server.
#[tokio::test]
async fn builder_custom_base_url_routes_requests() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/sessions/x/status"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "connected": true,
            "phone": "5511999"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let wi = Wi::builder("k").base_url(server.uri()).build();
    let status = wi.session("x").status().await.unwrap();
    assert!(status.connected);
    server.verify().await;
}

/// Verify that x-api-key is sent on every request.
#[tokio::test]
async fn api_key_header_sent_on_every_request() {
    let server = MockServer::start().await;

    // Two different endpoints, same key required
    Mock::given(method("GET"))
        .and(path("/sessions/s/status"))
        .and(header("x-api-key", "my-secret-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "connected": false, "phone": null
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/sessions/s/qr"))
        .and(header("x-api-key", "my-secret-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "qr": null
        })))
        .mount(&server)
        .await;

    let wi = Wi::builder("my-secret-key").base_url(server.uri()).build();
    let sess = wi.session("s");
    sess.status().await.unwrap();
    sess.qr().await.unwrap();
}
