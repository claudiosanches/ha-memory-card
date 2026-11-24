//! Built-in and custom play statuses.

mod common;

use axum::http::StatusCode;
use common::TestApp;
use serde_json::{Value, json};

async fn keys(app: &TestApp) -> Vec<String> {
    let (status, list) = app.json("GET", "/api/play-statuses", None).await;
    assert_eq!(status, StatusCode::OK);
    list.as_array()
        .unwrap()
        .iter()
        .map(|s| s["key"].as_str().unwrap().to_owned())
        .collect()
}

async fn create(app: &TestApp, label: &str, counts_as: &str) -> (StatusCode, Value) {
    app.json(
        "POST",
        "/api/play-statuses",
        Some(json!({"label": label, "color": "teal", "counts_as": counts_as})),
    )
    .await
}

/// The test manager's status for the game.
async fn status_of(app: &TestApp, title: &str) -> Option<String> {
    sqlx::query_scalar(
        "SELECT p.play_status FROM games g
         LEFT JOIN play_progress p ON p.game_id = g.id AND p.user_id = ?
         WHERE g.title = ?",
    )
    .bind(common::MANAGER)
    .bind(title)
    .fetch_one(&app.pool)
    .await
    .unwrap()
}

#[tokio::test]
async fn lists_the_built_in_statuses_in_order() {
    let app = TestApp::new().await;
    assert_eq!(
        keys(&app).await,
        [
            "backlog",
            "playing",
            "paused",
            "completed",
            "dropped",
            "mastered"
        ]
    );
    let (_, list) = app.json("GET", "/api/play-statuses", None).await;
    assert_eq!(
        list[3],
        json!({"key": "completed", "label": null, "color": "green", "counts_as": "finished",
               "builtin": true, "game_count": 0})
    );
}

#[tokio::test]
async fn adds_a_status_and_uses_it_on_games() {
    let app = TestApp::new().await;
    let (status, created) = create(&app, "Want to replay", "backlog").await;
    assert_eq!(status, StatusCode::CREATED, "{created}");
    assert_eq!(created["key"], "want_to_replay");
    assert_eq!(created["builtin"], false);
    assert_eq!(keys(&app).await.last().unwrap(), "want_to_replay");

    let (_, game) = app
        .json(
            "POST",
            "/api/games",
            Some(json!({"title": "Okami", "play_status": "want_to_replay"})),
        )
        .await;
    assert_eq!(game["play_status"], "want_to_replay");
    let id = game["id"].as_i64().unwrap();

    let (status, body) = app
        .json(
            "PUT",
            &format!("/api/games/{id}/play-status"),
            Some(json!({"play_status": "someday"})),
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["code"], "unknown_play_status");

    let (_, list) = app
        .json(
            "GET",
            "/api/games?ownership=any&play_status=want_to_replay",
            None,
        )
        .await;
    assert_eq!(list.as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn rejects_names_that_read_like_another_status() {
    let app = TestApp::new().await;
    for label in ["Finished", "on hold", "None"] {
        let (status, body) = create(&app, label, "finished").await;
        assert_eq!(status, StatusCode::CONFLICT, "{label}");
        assert_eq!(body["code"], "name_taken");
    }
    create(&app, "Replaying", "playing").await;
    let (status, _) = create(&app, "re-playing", "playing").await;
    assert_eq!(status, StatusCode::CONFLICT);
    let (status, body) = create(&app, "  ", "playing").await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["field"], "label");
}

#[tokio::test]
async fn built_ins_can_be_renamed_but_not_regrouped_or_deleted() {
    let app = TestApp::new().await;
    let (status, body) = app
        .json(
            "PUT",
            "/api/play-statuses/paused",
            Some(json!({"label": "On hold", "color": "orange", "counts_as": "paused"})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["label"], "On hold");
    assert_eq!(body["color"], "orange");

    let (_, body) = app
        .json(
            "PUT",
            "/api/play-statuses/paused",
            Some(json!({"label": "", "color": "yellow", "counts_as": "paused"})),
        )
        .await;
    assert_eq!(body["label"], Value::Null);

    let (status, body) = app
        .json(
            "PUT",
            "/api/play-statuses/paused",
            Some(json!({"color": "yellow", "counts_as": "finished"})),
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["code"], "builtin_status");

    let (status, body) = app.json("DELETE", "/api/play-statuses/paused", None).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["code"], "builtin_status");
}

#[tokio::test]
async fn deleting_a_status_moves_its_games() {
    let app = TestApp::new().await;
    create(&app, "Replaying", "playing").await;
    create(&app, "Someday", "backlog").await;
    let csv = "game_title,play_status\nOkami,Replaying\nTunic,someday\n";
    let (status, result) = app.post_csv("/api/import", csv).await;
    assert_eq!(status, StatusCode::OK, "{result}");
    assert_eq!(status_of(&app, "Okami").await.as_deref(), Some("replaying"));

    let (status, _) = app
        .json(
            "DELETE",
            "/api/play-statuses/replaying?move_to=playing",
            None,
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert_eq!(status_of(&app, "Okami").await.as_deref(), Some("playing"));

    let (status, body) = app
        .json("DELETE", "/api/play-statuses/someday?move_to=someday", None)
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["field"], "move_to");
    let (status, _) = app.json("DELETE", "/api/play-statuses/someday", None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert_eq!(status_of(&app, "Tunic").await, None);
    assert!(!keys(&app).await.contains(&"someday".to_owned()));
}

#[tokio::test]
async fn reorders_statuses() {
    let app = TestApp::new().await;
    create(&app, "Someday", "backlog").await;
    let order = [
        "someday",
        "backlog",
        "playing",
        "paused",
        "completed",
        "dropped",
        "mastered",
    ];
    let (status, list) = app
        .json(
            "PUT",
            "/api/play-statuses/order",
            Some(json!({"keys": order})),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list[0]["key"], "someday");
    assert_eq!(keys(&app).await, order);

    let (status, body) = app
        .json(
            "PUT",
            "/api/play-statuses/order",
            Some(json!({"keys": ["backlog"]})),
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["code"], "invalid_order");
}

#[tokio::test]
async fn suggestions_use_every_status_that_counts_as_backlog() {
    let app = TestApp::new().await;
    create(&app, "Someday", "backlog").await;
    let csv = "game_title,play_status,platform,region,format\n\
        Tunic,Someday,Switch,eu,digital\n\
        Zelda,completed,Switch,eu,digital\n";
    let (status, result) = app.post_csv("/api/import", csv).await;
    assert_eq!(status, StatusCode::OK, "{result}");

    let (status, body) = app.json("POST", "/api/suggestions", Some(json!({}))).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["game"]["title"], "Tunic");
}

#[tokio::test]
async fn export_writes_status_keys() {
    let app = TestApp::new().await;
    create(&app, "Want to replay", "backlog").await;
    let (status, result) = app
        .post_csv(
            "/api/import",
            "game_title,play_status\nOkami,want to replay\n",
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{result}");
    let (status, body) = app.get("/api/export.csv").await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        String::from_utf8(body)
            .unwrap()
            .contains(",Okami,want_to_replay,")
    );
}
