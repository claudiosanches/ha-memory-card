//! Duplicate groups and their labels.
// Tests set up and read rows with SQL.
#![allow(clippy::disallowed_methods)]

mod common;

use axum::http::StatusCode;
use common::TestApp;
use serde_json::{Value, json};

const COLLECTION: &str = "game_title,platform,region,format,device,store\n\
    Tetris,Switch,eu,physical,,\n\
    Tetris,Switch,eu,physical,,\n\
    Hades,Switch,eu,digital,Switch OLED,Nintendo eShop\n\
    Hades,PS4,na,physical,PS5,\n\
    Celeste,Switch,ww,digital,Switch OLED,\n\
    Celeste,Switch,ww,digital,Switch Lite,\n\
    Zelda,Switch,eu,physical,,\n";

#[tokio::test]
async fn groups_games_with_two_or_more_owned_copies() {
    let app = TestApp::new().await;
    let (status, result) = app.post_csv("/api/import", COLLECTION).await;
    assert_eq!(status, StatusCode::OK, "{result}");

    let (status, groups) = app.json("GET", "/api/duplicates", None).await;
    assert_eq!(status, StatusCode::OK, "{groups}");
    let summary: Vec<(String, Value, usize)> = groups
        .as_array()
        .unwrap()
        .iter()
        .map(|g| {
            (
                g["title"].as_str().unwrap().to_owned(),
                g["labels"].clone(),
                g["copies"].as_array().unwrap().len(),
            )
        })
        .collect();
    assert_eq!(
        summary,
        [
            (
                "Celeste".to_owned(),
                json!(["cross_device", "same_release"]),
                2
            ),
            (
                "Hades".to_owned(),
                json!([
                    "cross_platform",
                    "cross_region",
                    "cross_device",
                    "physical_digital"
                ]),
                2
            ),
            ("Tetris".to_owned(), json!(["same_release"]), 2),
        ]
    );
}

#[tokio::test]
async fn wanted_copies_do_not_count() {
    let app = TestApp::new().await;
    app.post_csv(
        "/api/import",
        "game_title,platform,region,format\nZelda,Switch,eu,physical\n",
    )
    .await;
    sqlx::query("INSERT INTO copies (release_id, status, format) SELECT id, 'wanted', 'digital' FROM releases")
        .execute(&app.pool)
        .await
        .unwrap();
    let (_, groups) = app.json("GET", "/api/duplicates", None).await;
    assert_eq!(groups, json!([]));
}
