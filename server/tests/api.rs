mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::{MANAGER, SUPERVISOR, TestApp};
use serde_json::Value;
use tower::ServiceExt;

#[tokio::test]
async fn status_reports_empty_collection() {
    let app = TestApp::new().await;
    let (status, body) = app.get("/api/status").await;
    assert_eq!(status, StatusCode::OK);

    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(json["igdb_configured"], false);
}

#[tokio::test]
async fn untrusted_peer_is_rejected() {
    let app = TestApp::new().await;
    let request = Request::get("/api/status").body(Body::empty()).unwrap();
    let (status, _) = app.request("192.168.1.20:40000", request).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn ipv4_mapped_supervisor_is_trusted() {
    let app = TestApp::new().await;
    let request = Request::get("/api/status").body(Body::empty()).unwrap();
    let (status, _) = app.request("[::ffff:172.30.32.2]:40000", request).await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn unknown_paths_are_not_found() {
    let app = TestApp::new().await;
    assert_eq!(app.get("/api/nope").await.0, StatusCode::NOT_FOUND);
    assert_eq!(app.get("/nope.js").await.0, StatusCode::NOT_FOUND);
}

#[test]
fn openapi_lists_status_route() {
    let spec = memory_card::openapi();
    assert!(spec.paths.paths.contains_key("/api/status"));
}

#[tokio::test]
async fn responses_carry_security_headers() {
    let app = TestApp::new().await;
    let covers = app.config.covers_dir().join("game-1");
    std::fs::create_dir_all(&covers).unwrap();
    std::fs::write(covers.join("cover.jpg"), b"\xFF\xD8\xFF").unwrap();

    let mut csp = Vec::new();
    for path in ["/", "/api/status", "/covers/game-1/cover.jpg"] {
        let request = Request::get(path)
            .header("x-remote-user-id", MANAGER)
            .body(Body::empty())
            .unwrap();
        let response = app.router_from(SUPERVISOR).oneshot(request).await.unwrap();
        let headers = response.headers();
        assert_eq!(headers["x-content-type-options"], "nosniff", "{path}");
        assert_eq!(headers["referrer-policy"], "no-referrer", "{path}");
        assert!(headers.contains_key("permissions-policy"), "{path}");
        csp.push(
            headers
                .get("content-security-policy")
                .map(|v| v.to_str().unwrap().to_owned()),
        );
    }
    let page = csp[0].as_deref().unwrap();
    assert!(
        page.contains("script-src 'self' 'wasm-unsafe-eval'"),
        "{page}"
    );
    assert!(page.contains("frame-ancestors 'self'"), "{page}");
    assert_eq!(csp[1], None);
    assert_eq!(csp[2].as_deref(), Some("default-src 'none'; sandbox"));
}
