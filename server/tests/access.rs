mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::{MANAGER, SUPERVISOR, TestApp};
use serde_json::{Value, json};

const VIEWER: &str = "fedcba9876543210fedcba9876543210";

/// Sends a request as `user` (or nobody) and parses the JSON reply, if any.
async fn send(
    app: &TestApp,
    peer: &str,
    method: &str,
    path: &str,
    user: Option<&str>,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(path);
    if let Some(user) = user {
        builder = builder
            .header("x-remote-user-id", user)
            .header("x-remote-user-name", "sam")
            .header("x-remote-user-display-name", "Sam");
    }
    let request = match body {
        Some(body) => builder
            .header("content-type", "application/json")
            .body(Body::from(body.to_string())),
        None => builder.body(Body::empty()),
    }
    .unwrap();
    let (status, bytes) = app.raw(peer, request).await;
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

#[tokio::test]
async fn nobody_manages_until_managers_are_configured() {
    let app = TestApp::with_env(&[("MEMORY_CARD_MANAGERS", "")]).await;

    let (status, me) = send(&app, SUPERVISOR, "GET", "/api/me", Some(MANAGER), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(me["role"], "viewer");
    assert_eq!(me["managers_configured"], false);

    let (status, body) = send(
        &app,
        SUPERVISOR,
        "POST",
        "/api/stores",
        Some(MANAGER),
        Some(json!({ "name": "Amazon" })),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["code"], "manager_required");
}

#[tokio::test]
async fn me_names_the_user_and_role() {
    let app = TestApp::new().await;

    let (status, me) = send(&app, SUPERVISOR, "GET", "/api/me", Some(MANAGER), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        me,
        json!({
            "user": { "id": MANAGER, "name": "sam", "display_name": "Sam" },
            "role": "manager",
            "managers_configured": true,
            "can_track_progress": true,
        })
    );

    let (_, me) = send(&app, SUPERVISOR, "GET", "/api/me", Some(VIEWER), None).await;
    assert_eq!(me["role"], "viewer");
}

#[tokio::test]
async fn missing_identity_is_rejected() {
    let app = TestApp::new().await;

    for path in ["/api/games", "/api/me", "/covers/game-1/cover.jpg"] {
        let (status, body) = send(&app, SUPERVISOR, "GET", path, None, None).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{path}");
        assert_eq!(body["code"], "identity_required", "{path}");
    }
    let (status, _) = send(
        &app,
        SUPERVISOR,
        "GET",
        "/api/games",
        Some("not valid"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // The UI still loads, to explain the problem.
    let (status, _) = app.get("/").await;
    assert_ne!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn identity_does_not_admit_an_untrusted_peer() {
    let app = TestApp::new().await;
    let (status, _) = send(
        &app,
        "10.0.0.5:1234",
        "GET",
        "/api/games",
        Some(MANAGER),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn viewers_read_but_change_nothing() {
    let app = TestApp::new().await;
    let release = app.release("Tunic", "switch").await;
    let game: i64 = sqlx::query_scalar("SELECT game_id FROM releases WHERE id = ?")
        .bind(release)
        .fetch_one(&app.pool)
        .await
        .unwrap();

    let (status, games) = send(
        &app,
        SUPERVISOR,
        "GET",
        "/api/games?ownership=any",
        Some(VIEWER),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(games.as_array().unwrap().len(), 1);

    let (status, body) = send(
        &app,
        SUPERVISOR,
        "PUT",
        &format!("/api/games/{game}/play-status"),
        Some(VIEWER),
        Some(json!({ "play_status": "playing" })),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["code"], "manager_required");
    assert_eq!(
        app.scalar("SELECT COUNT(*) FROM games WHERE play_status IS NOT NULL")
            .await,
        0
    );

    let (status, _) = send(
        &app,
        SUPERVISOR,
        "PUT",
        &format!("/api/games/{game}/play-status"),
        Some(MANAGER),
        Some(json!({ "play_status": "playing" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
}

/// Walks every route in the API document, so a new route cannot skip the check.
#[tokio::test]
async fn every_route_checks_the_role() {
    let app = TestApp::new().await;
    let manager_reads = [
        "/api/export.csv",
        "/api/import/template.csv",
        "/api/igdb/search",
        "/api/igdb/titles",
        "/api/games/{id}/igdb-contents",
        "/api/igdb/matches",
    ];

    let openapi = memory_card::openapi();
    let mut checked = 0;
    for (path, item) in &openapi.paths.paths {
        let operations = [
            ("GET", item.get.is_some()),
            ("POST", item.post.is_some()),
            ("PUT", item.put.is_some()),
            ("DELETE", item.delete.is_some()),
            ("PATCH", item.patch.is_some()),
        ];
        for (method, _) in operations.into_iter().filter(|(_, present)| *present) {
            let uri = path.replace("{id}", "1").replace("{key}", "custom");
            let body = (method != "GET").then(|| json!({}));
            let viewer_allowed = match method {
                "GET" => !manager_reads.contains(&path.as_str()),
                _ => false,
            };

            let (status, _) =
                send(&app, SUPERVISOR, method, &uri, Some(VIEWER), body.clone()).await;
            if viewer_allowed {
                assert!(
                    status != StatusCode::FORBIDDEN && status != StatusCode::UNAUTHORIZED,
                    "viewer {method} {path}: {status}"
                );
            } else {
                assert_eq!(status, StatusCode::FORBIDDEN, "viewer {method} {path}");
            }

            let (status, _) = send(&app, SUPERVISOR, method, &uri, Some(MANAGER), body).await;
            assert!(
                status != StatusCode::FORBIDDEN && status != StatusCode::UNAUTHORIZED,
                "manager {method} {path}: {status}"
            );
            checked += 1;
        }
    }
    assert!(checked > 40, "only {checked} operations found");
}

#[tokio::test]
async fn viewers_track_progress_when_the_option_allows_it() {
    let app = TestApp::with_env(&[("MEMORY_CARD_VIEWERS_TRACK_PROGRESS", "true")]).await;
    let release = app.release("Tunic", "switch").await;
    let game: i64 = sqlx::query_scalar("SELECT game_id FROM releases WHERE id = ?")
        .bind(release)
        .fetch_one(&app.pool)
        .await
        .unwrap();

    let (_, me) = send(&app, SUPERVISOR, "GET", "/api/me", Some(VIEWER), None).await;
    assert_eq!(me["role"], "viewer");
    assert_eq!(me["can_track_progress"], true);

    let uri = format!("/api/games/{game}/play-status");
    let playing = json!({ "play_status": "playing" });
    let (status, body) = send(&app, SUPERVISOR, "PUT", &uri, Some(VIEWER), Some(playing)).await;
    assert_eq!(status, StatusCode::OK, "{body}");

    let uri = format!("/api/games/{game}/sessions");
    let today = json!({ "played_on": "2026-11-21" });
    let (status, body) = send(&app, SUPERVISOR, "POST", &uri, Some(VIEWER), Some(today)).await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    let session = body["sessions"][0]["id"].as_i64().unwrap();
    let uri = format!("/api/sessions/{session}");
    let moved = json!({ "played_on": "2026-11-20" });
    let (status, body) = send(&app, SUPERVISOR, "PUT", &uri, Some(VIEWER), Some(moved)).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = send(&app, SUPERVISOR, "DELETE", &uri, Some(VIEWER), None).await;
    assert_eq!(status, StatusCode::OK, "{body}");

    let (status, _) = send(
        &app,
        SUPERVISOR,
        "PUT",
        &format!("/api/games/{game}"),
        Some(VIEWER),
        Some(json!({ "title": "Renamed" })),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}
