//! Home Assistant tags: linking a copy and starting it with a scan.
// Tests set up and read rows with SQL.
#![allow(clippy::disallowed_methods)]

mod common;

use axum::Router;
use axum::http::{HeaderMap, StatusCode};
use axum::routing::post;
use common::{MANAGER, TestApp};
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
    app_with(&[]).await
}

async fn app_with(vars: &[(&str, &str)]) -> TestApp {
    let url = home_assistant().await;
    let mut all = vec![
        ("MEMORY_CARD_HA_URL", url.as_str()),
        ("MEMORY_CARD_HA_TOKEN", "ha-token"),
    ];
    all.extend_from_slice(vars);
    TestApp::with_env(&all).await
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
        tags::scanned(&app.state, "04-a1-b2", Some(MANAGER), None).await,
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
            tags::scanned(&app.state, "04-a1-b2", Some(MANAGER), None).await,
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
    assert_eq!(detail["play_status"], "playing");

    assert_eq!(
        tags::scanned(&app.state, "unknown", None, None).await,
        Scan::Unknown
    );

    let (status, detail) = app.json("DELETE", "/api/tags/04-a1-b2", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(detail["releases"][0]["copies"][0]["tags"], json!([]));
    assert_eq!(
        tags::scanned(&app.state, "04-a1-b2", None, None).await,
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
    assert_eq!(
        tags::scanned(&app.state, "t1", None, None).await,
        Scan::Unknown
    );

    app.json("POST", &path, None).await;
    app.state.tag_link.lock().unwrap().as_mut().unwrap().until =
        time::OffsetDateTime::now_utc() - time::Duration::seconds(1);
    assert_eq!(
        tags::scanned(&app.state, "t1", None, None).await,
        Scan::Unknown
    );

    app.json("POST", &path, None).await;
    tags::scanned(&app.state, "t1", None, None).await;
    tags::scanned(&app.state, "t1", Some(MANAGER), None).await;
    let (_, detail): (_, Value) = app.json("GET", &format!("/api/games/{game}"), None).await;
    assert!(detail["sessions"].as_array().unwrap().is_empty());
    assert_eq!(detail["play_status"], "completed");

    let (status, _) = app.json("POST", "/api/copies/999/tag-link", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn a_finished_game_can_keep_its_status_on_a_scan() {
    let url = home_assistant().await;
    let app = TestApp::with_env(&[
        ("MEMORY_CARD_HA_URL", &url),
        ("MEMORY_CARD_HA_TOKEN", "ha-token"),
        ("MEMORY_CARD_TAG_SCAN_FINISHED", "keep"),
    ])
    .await;
    *app.state.tag_status.lock().unwrap() = TagsStatus::Connected;
    let (game, copy) = copy_of(&app, "Celeste", "owned").await;
    app.json("POST", &format!("/api/copies/{copy}/tag-link"), None)
        .await;
    tags::scanned(&app.state, "t2", None, None).await;

    assert_eq!(
        tags::scanned(&app.state, "t2", Some(MANAGER), None).await,
        Scan::Played { game_id: game }
    );
    let (_, detail) = app.json("GET", &format!("/api/games/{game}"), None).await;
    assert_eq!(detail["sessions"].as_array().unwrap().len(), 1);
    assert_eq!(detail["play_status"], "completed");

    app.json(
        "PUT",
        &format!("/api/games/{game}/play-status"),
        Some(json!({"play_status": "paused"})),
    )
    .await;
    tags::scanned(&app.state, "t2", Some(MANAGER), None).await;
    let (_, detail) = app.json("GET", &format!("/api/games/{game}"), None).await;
    assert_eq!(
        detail["play_status"], "playing",
        "only finished games keep it"
    );
}

#[tokio::test]
async fn scans_respect_who_may_link_and_log_play() {
    let app = app().await;
    *app.state.tag_status.lock().unwrap() = TagsStatus::Connected;
    let (game, copy) = copy_of(&app, "Hades", "owned").await;
    app.json("POST", &format!("/api/copies/{copy}/tag-link"), None)
        .await;

    // Another user's scan leaves the manager's link waiting.
    assert_eq!(
        tags::scanned(&app.state, "t3", Some("viewer"), None).await,
        Scan::Unknown
    );
    assert!(app.state.tag_link.lock().unwrap().is_some());
    assert_eq!(
        tags::scanned(&app.state, "t3", None, None).await,
        Scan::Linked { copy_id: copy }
    );

    // Viewers and readers with no user cannot log play by default.
    assert_eq!(
        tags::scanned(&app.state, "t3", Some("viewer"), None).await,
        Scan::Unknown
    );
    assert_eq!(
        tags::scanned(&app.state, "t3", None, None).await,
        Scan::Unknown
    );
    assert_eq!(
        tags::scanned(&app.state, "t3", Some("bad id!"), None).await,
        Scan::Unknown
    );
    let long = "t".repeat(129);
    assert_eq!(
        tags::scanned(&app.state, &long, Some(MANAGER), None).await,
        Scan::Unknown
    );
    assert_eq!(app.scalar("SELECT COUNT(*) FROM play_sessions").await, 0);
    assert_eq!(
        app.scalar("SELECT COUNT(*) FROM users WHERE id = 'bad id!'")
            .await,
        0
    );

    assert_eq!(
        tags::scanned(&app.state, "t3", Some(MANAGER), None).await,
        Scan::Played { game_id: game }
    );
}

#[tokio::test]
async fn viewers_log_play_from_tags_only_when_allowed_and_managers_exist() {
    let app = app_with(&[("MEMORY_CARD_VIEWERS_TRACK_PROGRESS", "true")]).await;
    let (game, copy) = copy_of(&app, "Celeste", "owned").await;
    sqlx::query("INSERT INTO nfc_tags (tag_id, copy_id) VALUES ('t4', ?)")
        .bind(copy)
        .execute(&app.pool)
        .await
        .unwrap();
    assert_eq!(
        tags::scanned(&app.state, "t4", Some("viewer-1"), None).await,
        Scan::Played { game_id: game }
    );
    assert_eq!(
        app.scalar("SELECT COUNT(*) FROM play_sessions WHERE user_id = 'viewer-1'")
            .await,
        1
    );
    // A scan with no person has no play to log.
    assert_eq!(
        tags::scanned(&app.state, "t4", None, None).await,
        Scan::Unknown
    );
    assert_eq!(app.scalar("SELECT COUNT(*) FROM play_sessions").await, 1);

    let locked = app_with(&[
        ("MEMORY_CARD_VIEWERS_TRACK_PROGRESS", "true"),
        ("MEMORY_CARD_MANAGERS", ""),
    ])
    .await;
    let release = locked.release("Celeste", "switch").await;
    let copy: i64 = sqlx::query_scalar(
        "INSERT INTO copies (release_id, format) VALUES (?, 'physical') RETURNING id",
    )
    .bind(release)
    .fetch_one(&locked.pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO nfc_tags (tag_id, copy_id) VALUES ('t5', ?)")
        .bind(copy)
        .execute(&locked.pool)
        .await
        .unwrap();
    assert_eq!(
        tags::scanned(&locked.state, "t5", Some(MANAGER), None).await,
        Scan::Unknown
    );
    assert_eq!(locked.scalar("SELECT COUNT(*) FROM play_sessions").await, 0);
}

#[tokio::test]
async fn readers_log_play_for_the_person_the_options_name() {
    let app = app_with(&[
        ("MEMORY_CARD_TAG_READERS", "reader-1=ana"),
        ("MEMORY_CARD_VIEWERS_TRACK_PROGRESS", "true"),
    ])
    .await;
    let (game, copy) = copy_of(&app, "Celeste", "owned").await;
    sqlx::query("INSERT INTO nfc_tags (tag_id, copy_id) VALUES ('t6', ?)")
        .bind(copy)
        .execute(&app.pool)
        .await
        .unwrap();
    let sessions = |who: &'static str| {
        let app = &app;
        async move {
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM play_sessions WHERE user_id = ?")
                .bind(who)
                .fetch_one(&app.pool)
                .await
                .unwrap()
        }
    };

    assert_eq!(
        tags::scanned(&app.state, "t6", None, Some("reader-1")).await,
        Scan::Played { game_id: game }
    );
    assert_eq!(sessions("ana").await, 1);
    assert_eq!(
        tags::scanned(&app.state, "t6", None, Some("reader-2")).await,
        Scan::Unknown,
        "a reader the options do not name"
    );
    // The user Home Assistant names comes before the reader's person.
    assert_eq!(
        tags::scanned(&app.state, "t6", Some(MANAGER), Some("reader-1")).await,
        Scan::Played { game_id: game }
    );
    assert_eq!(sessions(MANAGER).await, 1);
    assert_eq!(sessions("ana").await, 1);
}
