//! Picking a game to play.

mod common;

use axum::http::StatusCode;
use common::TestApp;
use memory_card::db::suggestions;
use memory_card::error::ApiError;
use serde_json::{Value, json};

const COLLECTION: &str = "game_title,play_status,platform,region,format,device\n\
    Tetris,backlog,Switch,eu,physical,\n\
    Tetris,backlog,PS4,na,digital,\n\
    Hades,backlog,PS4,na,digital,PS5\n\
    Zelda,completed,Switch,eu,physical,\n\
    Celeste,,Switch,ww,digital,\n\
    Mario Kart Wii,paused,Wii,eu,physical,\n";

async fn collection() -> TestApp {
    let app = TestApp::new().await;
    let (status, result) = app.post_csv("/api/import", COLLECTION).await;
    assert_eq!(status, StatusCode::OK, "{result}");
    app
}

async fn suggest(app: &TestApp, filter: Value) -> (StatusCode, Value) {
    app.json("POST", "/api/suggestions", Some(filter)).await
}

async fn title(app: &TestApp, filter: Value) -> String {
    let (status, body) = suggest(app, filter).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body["game"]["title"].as_str().unwrap().to_owned()
}

async fn game_id(app: &TestApp, title: &str) -> i64 {
    sqlx::query_scalar("SELECT id FROM games WHERE title = ?")
        .bind(title)
        .fetch_one(&app.pool)
        .await
        .unwrap()
}

#[tokio::test]
async fn suggests_owned_backlog_games_and_skips_recent_picks() {
    let app = collection().await;
    sqlx::query("INSERT INTO games (title, sort_title) VALUES ('Okami', 'okami')")
        .execute(&app.pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO play_progress (user_id, game_id, play_status)
         SELECT ?, id, 'backlog' FROM games WHERE title = 'Okami'",
    )
    .bind(common::MANAGER)
    .execute(&app.pool)
    .await
    .unwrap();
    let release: i64 = sqlx::query_scalar(
        "INSERT INTO releases (game_id, platform_id, region)
         SELECT id, 14, 'eu' FROM games WHERE title = 'Okami' RETURNING id",
    )
    .fetch_one(&app.pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO copies (release_id, status, format) VALUES (?, 'wanted', 'digital')")
        .bind(release)
        .execute(&app.pool)
        .await
        .unwrap();

    let first = title(&app, json!({})).await;
    let second = title(&app, json!({})).await;
    let mut picked = [first.clone(), second];
    picked.sort();
    assert_eq!(picked, ["Hades", "Tetris"]);
    // Both were picked today, so the older pick comes back.
    assert_eq!(title(&app, json!({})).await, first);
    assert_eq!(app.scalar("SELECT COUNT(*) FROM suggestions").await, 3);

    // The backlog is the manager's; someone else has none yet.
    let theirs = suggestions::suggest(&app.pool, "someone-else", Default::default()).await;
    assert!(
        matches!(theirs, Err(ApiError::NoMatch("no_candidates"))),
        "{theirs:?}"
    );
}

#[tokio::test]
async fn returns_the_oldest_recent_pick_and_prunes_old_history() {
    let app = collection().await;
    let (tetris, hades) = (game_id(&app, "Tetris").await, game_id(&app, "Hades").await);
    sqlx::query(
        "INSERT INTO suggestions (user_id, game_id, suggested_at) VALUES
            (?3, ?1, strftime('%Y-%m-%dT%H:%M:%fZ', 'now', '-40 days')),
            (?3, ?1, strftime('%Y-%m-%dT%H:%M:%fZ', 'now', '-2 hours')),
            (?3, ?2, strftime('%Y-%m-%dT%H:%M:%fZ', 'now', '-1 hours'))",
    )
    .bind(tetris)
    .bind(hades)
    .bind(common::MANAGER)
    .execute(&app.pool)
    .await
    .unwrap();

    assert_eq!(title(&app, json!({})).await, "Tetris");
    assert_eq!(
        app.scalar("SELECT COUNT(*) FROM suggestions WHERE suggested_at < strftime('%Y-%m-%dT%H:%M:%fZ', 'now', '-30 days')")
            .await,
        0
    );
}

#[tokio::test]
async fn keeps_only_each_persons_newest_picks() {
    let app = collection().await;
    let tetris = game_id(&app, "Tetris").await;
    sqlx::query("INSERT INTO users (id) VALUES ('someone-else')")
        .execute(&app.pool)
        .await
        .unwrap();
    sqlx::query(
        "WITH RECURSIVE n(i) AS (SELECT 1 UNION ALL SELECT i + 1 FROM n WHERE i < 1500)
         INSERT INTO suggestions (user_id, game_id)
         SELECT CASE WHEN i <= 10 THEN 'someone-else' ELSE ? END, ? FROM n",
    )
    .bind(common::MANAGER)
    .bind(tetris)
    .execute(&app.pool)
    .await
    .unwrap();

    assert_eq!(title(&app, json!({})).await, "Hades");
    assert_eq!(
        app.scalar("SELECT COUNT(*) FROM suggestions").await,
        suggestions::KEPT_MAX + 10
    );
    assert_eq!(
        app.scalar("SELECT game_id FROM suggestions ORDER BY id DESC LIMIT 1")
            .await,
        game_id(&app, "Hades").await
    );
}

#[tokio::test]
async fn filters_match_one_owned_copy() {
    let app = collection().await;
    let switch = app.platform_id("switch").await;

    assert_eq!(
        title(
            &app,
            json!({ "play_status": ["paused", "backlog"], "platform": ["wii"] })
        )
        .await,
        "Mario Kart Wii"
    );

    // Tetris has a Switch copy and a PS4 copy; only the Switch one matches.
    let (status, body) = suggest(&app, json!({ "platform": ["switch"] })).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["game"]["title"], "Tetris");
    let switch_copy: i64 = sqlx::query_scalar(
        "SELECT c.id FROM copies c JOIN releases r ON r.id = c.release_id
         JOIN games g ON g.id = r.game_id WHERE g.title = 'Tetris' AND r.platform_id = ?",
    )
    .bind(switch)
    .fetch_one(&app.pool)
    .await
    .unwrap();
    assert_eq!(body["copy_ids"], json!([switch_copy]));

    sqlx::query(
        "UPDATE releases SET local_max_players = 4
         WHERE platform_id = ? AND game_id = (SELECT id FROM games WHERE title = 'Tetris')",
    )
    .bind(switch)
    .execute(&app.pool)
    .await
    .unwrap();
    assert_eq!(
        title(&app, json!({ "min_local_players": 2 })).await,
        "Tetris"
    );

    sqlx::query(
        "INSERT INTO game_game_modes (game_id, game_mode_id)
         SELECT g.id, m.id FROM games g, game_modes m WHERE g.title = 'Hades' AND m.slug = 'co-op'",
    )
    .execute(&app.pool)
    .await
    .unwrap();
    assert_eq!(
        title(&app, json!({ "game_mode": ["co-op"] })).await,
        "Hades"
    );

    let (status, body) = suggest(&app, json!({ "platform": ["3ds"] })).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["code"], "no_candidates");
}

#[tokio::test]
async fn a_device_matches_its_copies_and_unassigned_copies_it_plays() {
    let app = collection().await;
    let (wii, wii_u) = (app.platform_id("wii").await, app.platform_id("wii-u").await);
    let (status, device) = app
        .json(
            "POST",
            "/api/devices",
            Some(json!({ "name": "Wii U", "kind": "console", "platform_ids": [wii_u, wii] })),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "{device}");

    let any = json!(["backlog", "paused", "completed"]);
    assert_eq!(
        title(
            &app,
            json!({ "play_status": any, "device": [device["slug"]] })
        )
        .await,
        "Mario Kart Wii"
    );

    // The import gave the PS5 the PS4 platform. Hades is on it; Tetris has a PS4 copy with no device.
    let mut picked = [
        title(&app, json!({ "device": ["ps5"] })).await,
        title(&app, json!({ "device": ["ps5"] })).await,
    ];
    picked.sort();
    assert_eq!(picked, ["Hades", "Tetris"]);
}

#[tokio::test]
async fn rejects_unknown_filters() {
    let app = collection().await;
    let (status, body) = suggest(&app, json!({ "platfrom": ["wii"] })).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["code"], "invalid_body");
    let (status, _) = suggest(&app, json!({ "platform": [1] })).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}
