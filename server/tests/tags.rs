//! Home Assistant tags: linking a copy and starting it with a scan.

mod common;

use axum::Router;
use axum::http::{HeaderMap, StatusCode};
use axum::routing::post;
use common::TestApp;
use memory_card::tags::{self, Scan, TagsStatus};
use serde_json::{Value, json};

/// A Home Assistant stand-in whose day is 2 January 2026.
async fn home_assistant() -> String {
    let stub = Router::new().route(
        "/api/template",
        post(|headers: HeaderMap| async move {
            if headers.get("authorization").and_then(|v| v.to_str().ok()) == Some("Bearer ha-token")
            {
                (StatusCode::OK, "2026-01-02")
            } else {
                (StatusCode::UNAUTHORIZED, "")
            }
        }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, stub).await.unwrap() });
    base
}

async fn app() -> TestApp {
    let url = home_assistant().await;
    TestApp::with_env(&[
        ("MEMORY_CARD_HA_URL", &url),
        ("MEMORY_CARD_HA_TOKEN", "ha-token"),
    ])
    .await
}

async fn copy_of(app: &TestApp, title: &str, status: &str) -> (i64, i64) {
    let switch = app.platform_id("switch").await;
    let (_, game) = app
        .json(
            "POST",
            "/api/games",
            Some(json!({"title": title, "play_status": "completed"})),
        )
        .await;
    let game = game["id"].as_i64().unwrap();
    let body =
        json!({"status": status, "platform_id": switch, "region": "eu", "format": "physical"});
    let (_, detail) = app
        .json("POST", &format!("/api/games/{game}/copies"), Some(body))
        .await;
    (
        game,
        detail["releases"][0]["copies"][0]["id"].as_i64().unwrap(),
    )
}

#[tokio::test]
async fn a_scanned_tag_starts_its_copy() {
    let app = app().await;
    let (game, copy) = copy_of(&app, "Tetris 99", "owned").await;

    let path = format!("/api/copies/{copy}/tag-link");
    let (status, body) = app.json("POST", &path, None).await;
    assert_eq!(
        (status, body["code"].clone()),
        (
            StatusCode::SERVICE_UNAVAILABLE,
            json!("home_assistant_unavailable")
        )
    );
    *app.state.tag_status.lock().unwrap() = TagsStatus::Connected;
    let (status, body) = app.json("POST", &path, None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(body["expires_at"].as_str().is_some());

    assert_eq!(
        tags::scanned(&app.state, "04-a1-b2", Some("phone-user")).await,
        Scan::Linked { copy_id: copy }
    );
    let (_, detail) = app.json("GET", &format!("/api/games/{game}"), None).await;
    assert_eq!(
        detail["releases"][0]["copies"][0]["tags"][0]["tag_id"],
        "04-a1-b2"
    );
    assert!(
        detail["sessions"].as_array().unwrap().is_empty(),
        "linking is not playing"
    );

    for _ in 0..2 {
        assert_eq!(
            tags::scanned(&app.state, "04-a1-b2", Some("phone-user")).await,
            Scan::Played { game_id: game }
        );
    }
    let (_, detail) = app.json("GET", &format!("/api/games/{game}"), None).await;
    let sessions = detail["sessions"].as_array().unwrap();
    assert_eq!(sessions.len(), 1, "one session a day");
    assert_eq!(
        sessions[0]["played_on"], "2026-01-02",
        "Home Assistant's day"
    );
    assert_eq!(sessions[0]["copy_id"], copy);
    assert_eq!(sessions[0]["created_by"]["id"], "phone-user");
    assert_eq!(detail["play_status"], "playing");

    assert_eq!(
        tags::scanned(&app.state, "unknown", None).await,
        Scan::Unknown
    );

    let (status, detail) = app.json("DELETE", "/api/tags/04-a1-b2", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(detail["releases"][0]["copies"][0]["tags"], json!([]));
    assert_eq!(
        tags::scanned(&app.state, "04-a1-b2", None).await,
        Scan::Unknown
    );
}

#[tokio::test]
async fn links_expire_and_cancel_and_wishes_do_not_play() {
    let app = app().await;
    *app.state.tag_status.lock().unwrap() = TagsStatus::Connected;
    let (game, copy) = copy_of(&app, "Silksong", "wanted").await;
    let path = format!("/api/copies/{copy}/tag-link");

    app.json("POST", &path, None).await;
    let (status, _) = app.json("DELETE", &path, None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert_eq!(tags::scanned(&app.state, "t1", None).await, Scan::Unknown);

    app.json("POST", &path, None).await;
    app.state.tag_link.lock().unwrap().as_mut().unwrap().until =
        time::OffsetDateTime::now_utc() - time::Duration::seconds(1);
    assert_eq!(tags::scanned(&app.state, "t1", None).await, Scan::Unknown);

    app.json("POST", &path, None).await;
    tags::scanned(&app.state, "t1", None).await;
    tags::scanned(&app.state, "t1", None).await;
    let (_, detail): (_, Value) = app.json("GET", &format!("/api/games/{game}"), None).await;
    assert!(detail["sessions"].as_array().unwrap().is_empty());
    assert_eq!(detail["play_status"], "completed");

    let (status, _) = app.json("POST", "/api/copies/999/tag-link", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
