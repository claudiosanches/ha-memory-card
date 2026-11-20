mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::TestApp;
use serde_json::Value;

#[tokio::test]
async fn status_reports_empty_collection() {
    let app = TestApp::new().await;
    let (status, body) = app.get("/api/status").await;
    assert_eq!(status, StatusCode::OK);

    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(json["igdb_configured"], false);
    assert_eq!(json["games"], 0);
    assert_eq!(json["owned_copies"], 0);
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
