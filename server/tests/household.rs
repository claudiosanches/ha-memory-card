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
    for sql in [
        "INSERT INTO play_progress (user_id, game_id, play_status) VALUES ('ana', ?, 'completed')",
        "INSERT INTO play_status_changes (user_id, game_id, play_status, source)
         VALUES ('ana', ?, 'completed', 'manual')",
    ] {
        sqlx::query(sql)
            .bind(tetris)
            .execute(&app.pool)
            .await
            .unwrap();
    }

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
    sqlx::query("UPDATE play_status_changes SET changed_at = '2001-01-01T00:00:00.000Z'")
        .execute(&app.pool)
        .await
        .unwrap();
}

async fn finished_all_time(app: &TestApp) -> i64 {
    let (_, all) = app.json("GET", "/api/household?period=all", None).await;
    all[0]["finished"].as_i64().unwrap()
}

async fn history(app: &TestApp, game: i64) -> Vec<(String, String)> {
    sqlx::query_as(
        "SELECT play_status, source FROM play_status_changes WHERE game_id = ? ORDER BY id",
    )
    .bind(game)
    .fetch_all(&app.pool)
    .await
    .unwrap()
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

#[tokio::test]
async fn a_finish_stays_counted_after_the_status_changes_again() {
    let app = TestApp::new().await;
    app.post_csv("/api/import", COLLECTION).await;
    let tetris = app
        .scalar("SELECT id FROM games WHERE title = 'Tetris'")
        .await;

    set_status(&app, tetris, "completed").await;
    changed_long_ago(&app).await;
    set_status(&app, tetris, "playing").await;
    assert_eq!(finished_all_time(&app).await, 1, "finished back in 2001");
    assert_eq!(finished_this_month(&app).await, 0);
    assert_eq!(
        history(&app, tetris).await,
        [
            ("backlog".into(), "csv".into()),
            ("completed".into(), "manual".into()),
            ("playing".into(), "manual".into()),
        ]
    );
}

#[tokio::test]
async fn a_status_changed_again_the_same_day_keeps_one_row() {
    let app = TestApp::new().await;
    app.post_csv("/api/import", COLLECTION).await;
    let tetris = app
        .scalar("SELECT id FROM games WHERE title = 'Tetris'")
        .await;
    changed_long_ago(&app).await;

    set_status(&app, tetris, "completed").await;
    set_status(&app, tetris, "playing").await;
    assert_eq!(finished_all_time(&app).await, 0, "completed by mistake");
    assert_eq!(
        history(&app, tetris).await,
        [
            ("backlog".into(), "csv".into()),
            ("playing".into(), "manual".into()),
        ]
    );

    // Clearing the status adds nothing.
    let (status, _) = app
        .json(
            "PUT",
            &format!("/api/games/{tetris}/play-status"),
            Some(json!({"play_status": null})),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(history(&app, tetris).await.len(), 2);
}

#[tokio::test]
async fn imported_and_logged_statuses_go_in_the_history() {
    let app = TestApp::new().await;
    let (status, result) = app
        .post_csv(
            "/api/import",
            "game_title,play_status,platform,region,format\n\
             Celeste,completed,Switch,eu,physical\n\
             Tetris,backlog,Switch,eu,physical\n",
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{result}");
    let tetris = app
        .scalar("SELECT id FROM games WHERE title = 'Tetris'")
        .await;
    changed_long_ago(&app).await;
    app.json(
        "POST",
        &format!("/api/games/{tetris}/sessions"),
        Some(json!({"played_on": "2001-01-02"})),
    )
    .await;

    assert_eq!(
        finished_all_time(&app).await,
        0,
        "an import restates statuses"
    );
    assert_eq!(
        history(&app, tetris).await,
        [
            ("backlog".into(), "csv".into()),
            ("playing".into(), "manual".into()),
        ],
        "a session moves a backlog game to playing"
    );
}

#[tokio::test]
async fn deleting_a_status_moves_or_drops_its_history() {
    let app = TestApp::new().await;
    app.post_csv("/api/import", COLLECTION).await;
    let tetris = app
        .scalar("SELECT id FROM games WHERE title = 'Tetris'")
        .await;
    for label in ["Speedrun", "Gifted"] {
        let (status, body) = app
            .json(
                "POST",
                "/api/play-statuses",
                Some(json!({"label": label, "color": "teal", "counts_as": "finished"})),
            )
            .await;
        assert_eq!(status, StatusCode::CREATED, "{body}");
    }
    set_status(&app, tetris, "speedrun").await;
    changed_long_ago(&app).await;
    set_status(&app, tetris, "gifted").await;
    changed_long_ago(&app).await;
    set_status(&app, tetris, "playing").await;

    let (status, _) = app
        .json(
            "DELETE",
            "/api/play-statuses/speedrun?move_to=completed",
            None,
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _) = app.json("DELETE", "/api/play-statuses/gifted", None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert_eq!(
        history(&app, tetris).await,
        [
            ("backlog".into(), "csv".into()),
            ("completed".into(), "manual".into()),
            ("playing".into(), "manual".into()),
        ]
    );
    assert_eq!(finished_all_time(&app).await, 1);
}
