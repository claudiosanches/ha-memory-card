//! Photos of copies.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::{PHOTO as JPEG, TestApp, stored};
use serde_json::{Value, json};

async fn post_photo(app: &TestApp, copy: i64, bytes: &[u8]) -> (StatusCode, Value) {
    let request = Request::post(format!("/api/copies/{copy}/photos"))
        .header("content-type", "application/octet-stream")
        .body(Body::from(bytes.to_vec()))
        .unwrap();
    let (status, body) = app.request(common::SUPERVISOR, request).await;
    (status, serde_json::from_slice(&body).unwrap())
}

#[tokio::test]
async fn copies_keep_photos_until_they_go() {
    let app = TestApp::new().await;
    let switch = app.platform_id("switch").await;
    let (_, game) = app
        .json("POST", "/api/games", Some(json!({"title": "Okami"})))
        .await;
    let game = game["id"].as_i64().unwrap();
    let copy = json!({"platform_id": switch, "region": "eu", "format": "physical"});
    let (_, detail) = app
        .json("POST", &format!("/api/games/{game}/copies"), Some(copy))
        .await;
    let copy = detail["releases"][0]["copies"][0]["id"].as_i64().unwrap();

    let (status, detail) = post_photo(&app, copy, JPEG).await;
    assert_eq!(status, StatusCode::CREATED, "{detail}");
    let photos = &detail["releases"][0]["copies"][0]["photos"];
    let path = photos[0]["path"].as_str().unwrap().to_owned();
    assert!(path.starts_with(&format!("copy-{copy}/photo-")) && path.ends_with(".jpg"));
    let (status, bytes) = app.get(&format!("/covers/{path}")).await;
    assert_eq!((status, &bytes), (StatusCode::OK, &stored(JPEG)));
    let text = String::from_utf8_lossy(&bytes);
    assert!(
        !text.contains("PhoneCo") && !text.contains("secret place"),
        "metadata stripped"
    );

    let (status, body) = post_photo(&app, copy, b"plain text").await;
    assert_eq!(
        (status, body["code"].clone()),
        (StatusCode::UNPROCESSABLE_ENTITY, json!("invalid_image"))
    );
    let (status, _) = post_photo(&app, 999, JPEG).await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    post_photo(&app, copy, JPEG).await;
    let photo = photos[0]["id"].as_i64().unwrap();
    let (status, detail) = app
        .json("DELETE", &format!("/api/photos/{photo}"), None)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        detail["releases"][0]["copies"][0]["photos"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert!(
        !app.config.covers_dir().join(&path).exists(),
        "the file goes with it"
    );

    app.json("DELETE", &format!("/api/copies/{copy}"), None)
        .await;
    assert!(
        !app.config
            .covers_dir()
            .join(format!("copy-{copy}"))
            .exists()
    );
    assert_eq!(app.scalar("SELECT COUNT(*) FROM copy_photos").await, 0);
}

#[tokio::test]
async fn a_copy_holds_twenty_photos() {
    let app = TestApp::new().await;
    let release = app.release("Okami", "switch").await;
    let copy: i64 = sqlx::query_scalar(
        "INSERT INTO copies (release_id, format) VALUES (?, 'physical') RETURNING id",
    )
    .bind(release)
    .fetch_one(&app.pool)
    .await
    .unwrap();
    for _ in 0..20 {
        assert_eq!(post_photo(&app, copy, JPEG).await.0, StatusCode::CREATED);
    }
    assert_eq!(
        app.scalar("SELECT COUNT(DISTINCT path) FROM copy_photos")
            .await,
        20,
        "each upload gets its own file"
    );
    let (status, body) = post_photo(&app, copy, JPEG).await;
    assert_eq!(
        (status, body["code"].clone()),
        (StatusCode::CONFLICT, json!("too_many_photos"))
    );
}

#[tokio::test]
async fn uploads_at_once_cannot_pass_the_limit() {
    let app = TestApp::new().await;
    let release = app.release("Okami", "switch").await;
    let copy: i64 = sqlx::query_scalar(
        "INSERT INTO copies (release_id, format) VALUES (?, 'physical') RETURNING id",
    )
    .bind(release)
    .fetch_one(&app.pool)
    .await
    .unwrap();
    for n in 0..19 {
        sqlx::query("INSERT INTO copy_photos (copy_id, path) VALUES (?, ?)")
            .bind(copy)
            .bind(format!("copy-{copy}/old-{n}.jpg"))
            .execute(&app.pool)
            .await
            .unwrap();
    }

    let uploads = (0..4).map(|_| post_photo(&app, copy, JPEG));
    let statuses: Vec<StatusCode> = futures_util::future::join_all(uploads)
        .await
        .into_iter()
        .map(|(status, _)| status)
        .collect();
    assert_eq!(
        statuses
            .iter()
            .filter(|s| **s == StatusCode::CREATED)
            .count(),
        1,
        "{statuses:?}"
    );
    assert_eq!(app.scalar("SELECT COUNT(*) FROM copy_photos").await, 20);
    let dir = app.config.covers_dir().join(format!("copy-{copy}"));
    assert_eq!(
        std::fs::read_dir(dir).unwrap().count(),
        1,
        "refused uploads leave no file"
    );
}
