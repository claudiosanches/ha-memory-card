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
        assert_eq!(
            headers["permissions-policy"], "camera=(self), geolocation=(), microphone=()",
            "{path}"
        );
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

#[tokio::test]
async fn cross_site_writes_and_untyped_bodies_are_refused() {
    let app = TestApp::new().await;
    let post = |site: Option<&str>, content_type: &str| {
        let mut request = Request::post("/api/import/preview").header("content-type", content_type);
        if let Some(site) = site {
            request = request.header("sec-fetch-site", site);
        }
        request.body(Body::from("game_title\nCeleste\n")).unwrap()
    };

    for site in ["cross-site", "same-site"] {
        let (status, body) = app.request(SUPERVISOR, post(Some(site), "text/csv")).await;
        assert_eq!(status, StatusCode::FORBIDDEN, "{site}");
        let body: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(body["code"], "cross_site");
    }
    for site in [Some("same-origin"), Some("none"), None] {
        let (status, _) = app
            .request(SUPERVISOR, post(site, "text/csv; charset=utf-8"))
            .await;
        assert_eq!(status, StatusCode::OK, "{site:?}");
    }
    let request = Request::get("/api/status")
        .header("sec-fetch-site", "cross-site")
        .body(Body::empty())
        .unwrap();
    assert_eq!(app.request(SUPERVISOR, request).await.0, StatusCode::OK);

    for content_type in [
        "text/plain",
        "application/x-www-form-urlencoded",
        "multipart/form-data",
    ] {
        let (status, body) = app.request(SUPERVISOR, post(None, content_type)).await;
        assert_eq!(status, StatusCode::UNSUPPORTED_MEDIA_TYPE, "{content_type}");
        let body: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(body["code"], "unsupported_media_type");
    }
    let (status, _) = app
        .request(
            SUPERVISOR,
            Request::put("/api/games/1/cover")
                .header("content-type", "text/plain")
                .body(Body::from("x"))
                .unwrap(),
        )
        .await;
    assert_eq!(status, StatusCode::UNSUPPORTED_MEDIA_TYPE);
}

#[tokio::test]
async fn bodies_and_lists_have_limits() {
    let app = TestApp::new().await;
    let big = serde_json::json!({ "title": "x".repeat(1024 * 1024) });
    let (status, _) = app.json("POST", "/api/games", Some(big)).await;
    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);

    // An import may be larger than other bodies.
    let notes = "n".repeat(1500);
    let csv: String = std::iter::once("game_title,notes\n".to_owned())
        .chain((0..1500).map(|i| format!("Game {i},{notes}\n")))
        .collect();
    assert!(csv.len() > 2 * 1024 * 1024);
    let (status, _) = app.post_csv("/api/import/preview", &csv).await;
    assert_eq!(status, StatusCode::OK);
    let huge = "x".repeat(17 * 1024 * 1024);
    let (status, body) = app.post_csv("/api/import/preview", &huge).await;
    assert_eq!(
        (status, body["code"].as_str()),
        (StatusCode::PAYLOAD_TOO_LARGE, Some("too_large"))
    );

    let many: Vec<String> = (0..101).map(|i| format!("p{i}")).collect();
    let (status, _) = app
        .json(
            "GET",
            &format!("/api/games?platform={}", many.join(",")),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    let (status, _) = app
        .json(
            "GET",
            &format!("/api/games?platform={}", many[..100].join(",")),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = app
        .json(
            "POST",
            "/api/suggestions",
            Some(serde_json::json!({ "platform": many })),
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}
