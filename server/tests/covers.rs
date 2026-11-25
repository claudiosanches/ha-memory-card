//! Uploaded covers for games and releases.
// Tests read rows with SQL.
#![allow(clippy::disallowed_methods)]

mod common;

use axum::http::StatusCode;
use common::{COVER as JPEG, PNG, TestApp, stored};
use serde_json::Value;

fn files(app: &TestApp, dir: &str) -> Vec<String> {
    match std::fs::read_dir(app.config.covers_dir().join(dir)) {
        Ok(entries) => entries
            .map(|e| e.unwrap().file_name().into_string().unwrap())
            .collect(),
        Err(_) => Vec::new(),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn uploads_at_once_keep_the_file_the_game_points_to() {
    let app = TestApp::new().await;
    app.release("Okami", "switch").await;
    let game: i64 = app.scalar("SELECT id FROM games").await;
    let path = format!("/api/games/{game}/cover");

    let uploads = (0..16).map(|_| app.put_bytes(&path, JPEG));
    for (status, detail) in futures_util::future::join_all(uploads).await {
        assert_eq!(status, StatusCode::OK, "{detail}");
    }

    let stored_path: String = sqlx::query_scalar("SELECT cover_path FROM games")
        .fetch_one(&app.pool)
        .await
        .unwrap();
    let dir = format!("game-{game}");
    assert_eq!(
        files(&app, &dir),
        [stored_path.strip_prefix(&format!("{dir}/")).unwrap()]
    );
}

#[tokio::test]
async fn uploads_replace_the_game_cover() {
    let app = TestApp::new().await;
    let release = app.release("Okami", "switch").await;
    let game: i64 = app.scalar("SELECT id FROM games").await;

    let (status, detail) = app
        .put_bytes(&format!("/api/games/{game}/cover"), PNG)
        .await;
    assert_eq!(status, StatusCode::OK, "{detail}");
    assert_eq!(detail["cover_uploaded"], true);
    let first = detail["cover_path"].as_str().unwrap().to_owned();
    assert!(first.starts_with(&format!("game-{game}/upload-")) && first.ends_with(".png"));
    let (status, bytes) = app.get(&format!("/covers/{first}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(bytes, stored(PNG));

    // A second upload replaces the first file.
    let (_, detail) = app
        .put_bytes(&format!("/api/games/{game}/cover"), JPEG)
        .await;
    let second = detail["cover_path"].as_str().unwrap();
    assert!(second.ends_with(".jpg"));
    assert_eq!(files(&app, &format!("game-{game}")).len(), 1);

    let (status, detail) = app
        .put_bytes(&format!("/api/releases/{release}/cover"), PNG)
        .await;
    assert_eq!(status, StatusCode::OK, "{detail}");
    assert_eq!(detail["releases"][0]["cover_uploaded"], true);

    let (status, detail) = app
        .json("DELETE", &format!("/api/releases/{release}/cover"), None)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(detail["releases"][0]["cover_path"], Value::Null);
    assert_eq!(detail["releases"][0]["cover_uploaded"], false);
    assert!(files(&app, &format!("release-{release}")).is_empty());

    let (_, detail) = app
        .json("DELETE", &format!("/api/games/{game}/cover"), None)
        .await;
    assert_eq!(detail["cover_path"], Value::Null);
    assert!(files(&app, &format!("game-{game}")).is_empty());
}

#[tokio::test]
async fn rejects_files_that_are_not_images() {
    let app = TestApp::new().await;
    app.release("Okami", "switch").await;
    let game: i64 = app.scalar("SELECT id FROM games").await;

    let (status, body) = app
        .put_bytes(
            &format!("/api/games/{game}/cover"),
            b"<svg onload=alert(1)>",
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["code"], "invalid_image");

    let mut big = PNG.to_vec();
    big.resize(10 * 1024 * 1024 + 1, 0);
    let (status, body) = app
        .put_bytes(&format!("/api/games/{game}/cover"), &big)
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["code"], "image_too_large");

    let (status, _) = app.put_bytes("/api/games/999/cover", PNG).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(files(&app, "game-999").is_empty());
    assert!(files(&app, &format!("game-{game}")).is_empty());
}
