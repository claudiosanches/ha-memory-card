//! Everyone's play side by side.
// Tests set up and read rows with SQL.
#![allow(clippy::disallowed_methods)]

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::{SUPERVISOR, TestApp};
use serde_json::{Value, json};

const COLLECTION: &str = "game_title,play_status,platform,region,format\n\
    Hades,playing,PS4,na,digital\n\
    Tetris,backlog,Switch,eu,physical\n";

#[tokio::test]
async fn compares_everyone_who_keeps_play() {
    let app = TestApp::new().await;
    let (status, result) = app.post_csv("/api/import", COLLECTION).await;
    assert_eq!(status, StatusCode::OK, "{result}");
    let hades = app
        .scalar("SELECT id FROM games WHERE title = 'Hades'")
        .await;
    let tetris = app
        .scalar("SELECT id FROM games WHERE title = 'Tetris'")
        .await;
    let today: String = sqlx::query_scalar("SELECT date('now')")
        .fetch_one(&app.pool)
        .await
        .unwrap();

    for (day, minutes) in [
        (today.as_str(), 60),
        (today.as_str(), 30),
        ("2001-01-01", 500),
    ] {
        app.json(
            "POST",
            &format!("/api/games/{hades}/sessions"),
            Some(json!({"played_on": day, "minutes": minutes})),
        )
        .await;
    }
    sqlx::query("INSERT INTO users (id, display_name) VALUES ('ana', 'Ana')")
        .execute(&app.pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO play_sessions (user_id, game_id, played_on, minutes) VALUES ('ana', ?, date('now'), 120)",
    )
    .bind(tetris)
    .execute(&app.pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO play_progress (user_id, game_id, play_status) VALUES ('ana', ?, 'completed')",
    )
    .bind(tetris)
    .execute(&app.pool)
    .await
    .unwrap();

    let (status, month) = app.json("GET", "/api/household", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        month,
        json!([
            {
                "person": {"id": "ana", "name": "Ana"},
                "sessions": 1, "minutes": 120, "days": 1, "games": 1, "finished": 1,
                "top_game": {"game_id": tetris, "title": "Tetris", "cover_path": null, "sessions": 1},
            },
            {
                "person": {"id": common::MANAGER, "name": null},
                "sessions": 2, "minutes": 90, "days": 1, "games": 1, "finished": 0,
                "top_game": {"game_id": hades, "title": "Hades", "cover_path": null, "sessions": 2},
            },
        ])
    );

    let (_, all) = app.json("GET", "/api/household?period=all", None).await;
    assert_eq!(
        all[0]["person"]["id"],
        common::MANAGER,
        "500 minutes back in 2001"
    );
    assert_eq!(all[0]["minutes"], 590);
    assert_eq!(all[0]["days"], 2);

    let (status, error) = app.json("GET", "/api/household?period=week", None).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(error["code"], "invalid_filter");

    // Everyone may look.
    let request = Request::get("/api/household")
        .header("x-remote-user-id", "a-viewer")
        .body(Body::empty())
        .unwrap();
    let (status, body) = app.raw(SUPERVISOR, request).await;
    assert_eq!(status, StatusCode::OK);
    let body: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(body.as_array().unwrap().len(), 2);
}

async fn finished_this_month(app: &TestApp) -> i64 {
    let (_, month) = app.json("GET", "/api/household", None).await;
    month[0]["finished"].as_i64().unwrap()
}

async fn set_status(app: &TestApp, game: i64, status: &str) {
    let (code, body) = app
        .json(
            "PUT",
            &format!("/api/games/{game}/play-status"),
            Some(json!({"play_status": status})),
        )
        .await;
    assert_eq!(code, StatusCode::OK, "{body}");
}

async fn changed_long_ago(app: &TestApp) {
    sqlx::query("UPDATE play_progress SET updated_at = '2001-01-01T00:00:00.000Z'")
        .execute(&app.pool)
        .await
        .unwrap();
}

#[tokio::test]
async fn a_game_counts_as_finished_when_its_owner_finishes_it() {
    let app = TestApp::new().await;
    app.post_csv("/api/import", COLLECTION).await;
    let hades = app
        .scalar("SELECT id FROM games WHERE title = 'Hades'")
        .await;
    let tetris = app
        .scalar("SELECT id FROM games WHERE title = 'Tetris'")
        .await;
    // Play this month puts the manager on the page.
    sqlx::query(
        "INSERT INTO play_sessions (user_id, game_id, played_on) VALUES (?, ?, date('now'))",
    )
    .bind(common::MANAGER)
    .bind(hades)
    .execute(&app.pool)
    .await
    .unwrap();
    app.json(
        "POST",
        "/api/play-statuses",
        Some(json!({"label": "Someday", "color": "teal", "counts_as": "backlog"})),
    )
    .await;

    set_status(&app, tetris, "completed").await;
    assert_eq!(finished_this_month(&app).await, 1);

    changed_long_ago(&app).await;
    set_status(&app, tetris, "completed").await;
    assert_eq!(
        finished_this_month(&app).await,
        0,
        "the same status saved again"
    );

    set_status(&app, hades, "someday").await;
    changed_long_ago(&app).await;
    let (status, _) = app
        .json(
            "DELETE",
            "/api/play-statuses/someday?move_to=completed",
            None,
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert_eq!(
        finished_this_month(&app).await,
        0,
        "a manager moving a deleted status"
    );
}
