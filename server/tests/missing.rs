//! The details the setup wizard asks for.

mod common;

use axum::http::StatusCode;
use common::TestApp;
use serde_json::{Value, json};

async fn new_game(app: &TestApp, title: &str) -> i64 {
    let (status, game) = app
        .json("POST", "/api/games", Some(json!({ "title": title })))
        .await;
    assert_eq!(status, StatusCode::CREATED, "{game}");
    game["id"].as_i64().unwrap()
}

async fn add_copy(app: &TestApp, game: i64, body: Value) {
    let (status, detail) = app
        .json("POST", &format!("/api/games/{game}/copies"), Some(body))
        .await;
    assert_eq!(status, StatusCode::CREATED, "{detail}");
}

#[tokio::test]
async fn lists_what_each_game_lacks() {
    let app = TestApp::new().await;
    let switch = app.platform_id("switch").await;
    let store: i64 = app
        .scalar("SELECT id FROM stores ORDER BY id LIMIT 1")
        .await;
    let device: i64 = sqlx::query_scalar(
        "INSERT INTO devices (name, kind) VALUES ('Switch', 'console') RETURNING id",
    )
    .fetch_one(&app.pool)
    .await
    .unwrap();
    let single_player: i64 = app
        .scalar("SELECT id FROM game_modes WHERE slug = 'single-player'")
        .await;

    let bare = new_game(&app, "Bare").await;
    let complete = new_game(&app, "Complete").await;
    add_copy(
        &app,
        complete,
        json!({ "platform_id": switch, "region": "eu", "format": "physical",
                "device_id": device, "store_id": store, "condition": "cib" }),
    )
    .await;
    let (status, _) = app
        .json(
            "PUT",
            &format!("/api/games/{complete}"),
            Some(json!({ "title": "Complete", "play_status": "backlog", "game_mode_ids": [single_player] })),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    sqlx::query("UPDATE games SET cover_path = 'game-1/cover.jpg' WHERE id = ?")
        .bind(complete)
        .execute(&app.pool)
        .await
        .unwrap();

    let copies = new_game(&app, "Copies").await;
    add_copy(&app, copies, json!({ "platform_id": switch, "region": "eu", "format": "physical", "device_id": device, "store_id": store })).await;
    add_copy(
        &app,
        copies,
        json!({ "platform_id": switch, "region": "eu", "format": "digital", "device_id": device }),
    )
    .await;

    let (status, list) = app.json("GET", "/api/missing-details", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        list,
        json!([
            {
                "game_id": bare,
                "title": "Bare",
                "fields": ["play_status", "cover", "game_modes", "players", "copies"],
            },
            {
                "game_id": copies,
                "title": "Copies",
                "fields": ["play_status", "cover", "game_modes", "players", "store", "condition"],
            },
        ])
    );
}

#[tokio::test]
async fn player_counts_on_a_release_count() {
    let app = TestApp::new().await;
    let release = app.release("Mario Kart", "switch").await;
    sqlx::query("UPDATE releases SET local_max_players = 4 WHERE id = ?")
        .bind(release)
        .execute(&app.pool)
        .await
        .unwrap();

    let (_, list) = app.json("GET", "/api/missing-details", None).await;
    let fields = list[0]["fields"].as_array().unwrap();
    assert!(!fields.contains(&json!("players")), "{fields:?}");
}

#[tokio::test]
async fn games_only_on_the_wishlist_are_left_out() {
    let app = TestApp::new().await;
    let switch = app.platform_id("switch").await;
    let game = new_game(&app, "Hades II").await;
    add_copy(
        &app,
        game,
        json!({ "status": "wanted", "platform_id": switch, "region": "ww", "format": "digital" }),
    )
    .await;
    let (_, list) = app.json("GET", "/api/missing-details", None).await;
    assert_eq!(list, json!([]));
}
