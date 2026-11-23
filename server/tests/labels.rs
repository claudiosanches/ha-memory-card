//! Labels people make up, put on games and filter the library by.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::{SUPERVISOR, TestApp};
use serde_json::{Value, json};

async fn label(app: &TestApp, name: &str, color: &str) -> i64 {
    let (status, label) = app
        .json(
            "POST",
            "/api/labels",
            Some(json!({"name": name, "color": color})),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "{label}");
    label["id"].as_i64().unwrap()
}

async fn game(app: &TestApp, title: &str) -> i64 {
    let (status, game) = app
        .json("POST", "/api/games", Some(json!({"title": title})))
        .await;
    assert_eq!(status, StatusCode::CREATED, "{game}");
    game["id"].as_i64().unwrap()
}

async fn titles(app: &TestApp, query: &str) -> Vec<String> {
    let (status, games) = app
        .json("GET", &format!("/api/games?ownership=any&{query}"), None)
        .await;
    assert_eq!(status, StatusCode::OK, "{games}");
    games
        .as_array()
        .unwrap()
        .iter()
        .map(|g| g["title"].as_str().unwrap().to_owned())
        .collect()
}

#[tokio::test]
async fn labels_are_named_colored_and_counted() {
    let app = TestApp::new().await;
    let couch = label(&app, "Couch co-op", "teal").await;

    let (status, body) = app
        .json(
            "POST",
            "/api/labels",
            Some(json!({"name": "couch CO-OP", "color": "red"})),
        )
        .await;
    assert_eq!(
        (status, body["code"].as_str()),
        (StatusCode::CONFLICT, Some("name_taken"))
    );
    let (status, body) = app
        .json(
            "POST",
            "/api/labels",
            Some(json!({"name": "Mario; Luigi", "color": "red"})),
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["code"], "label_separator", "{body}");

    let (status, renamed) = app
        .json(
            "PUT",
            &format!("/api/labels/{couch}"),
            Some(json!({"name": "Couch", "color": "grape"})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{renamed}");
    assert_eq!(renamed["slug"], "couch-co-op", "renaming keeps the slug");
    assert_eq!(renamed["color"], "grape");

    let okami = game(&app, "Okami").await;
    let (status, _) = app
        .json(
            "PUT",
            &format!("/api/games/{okami}"),
            Some(json!({"title": "Okami", "label_ids": [couch]})),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let (_, labels) = app.json("GET", "/api/labels", None).await;
    assert_eq!(labels[0]["game_count"], 1);
}

#[tokio::test]
async fn games_keep_labels_unless_the_update_names_them() {
    let app = TestApp::new().await;
    let couch = label(&app, "Couch co-op", "teal").await;
    let retro = label(&app, "Retro", "orange").await;
    let okami = game(&app, "Okami").await;
    let path = format!("/api/games/{okami}");

    let (status, body) = app
        .json(
            "PUT",
            &path,
            Some(json!({"title": "Okami", "label_ids": [retro, couch, retro]})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (_, detail) = app.json("GET", &path, None).await;
    let names: Vec<&str> = detail["labels"]
        .as_array()
        .unwrap()
        .iter()
        .map(|l| l["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["Couch co-op", "Retro"], "by name, once each");
    assert_eq!(detail["labels"][1]["color"], "orange");

    app.json("PUT", &path, Some(json!({"title": "Ōkami"})))
        .await;
    let (_, detail) = app.json("GET", &path, None).await;
    assert_eq!(
        detail["labels"].as_array().unwrap().len(),
        2,
        "left out, they stay"
    );

    let (status, body) = app
        .json(
            "PUT",
            &path,
            Some(json!({"title": "Okami", "label_ids": [999]})),
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["code"], "unknown_id");

    let (status, _) = app
        .json("DELETE", &format!("/api/labels/{retro}"), None)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (_, detail) = app.json("GET", &path, None).await;
    assert_eq!(
        detail["labels"].as_array().unwrap().len(),
        1,
        "the game stays"
    );
}

#[tokio::test]
async fn the_library_shows_games_with_any_picked_label() {
    let app = TestApp::new().await;
    let couch = label(&app, "Couch co-op", "teal").await;
    let retro = label(&app, "Retro", "orange").await;
    for (title, labels) in [
        ("Okami", vec![retro]),
        ("Overcooked", vec![couch]),
        ("Hades", vec![]),
    ] {
        let id = game(&app, title).await;
        app.json(
            "PUT",
            &format!("/api/games/{id}"),
            Some(json!({"title": title, "label_ids": labels})),
        )
        .await;
    }

    assert_eq!(titles(&app, "label=retro").await, ["Okami"]);
    assert_eq!(
        titles(&app, "label=RETRO,couch-co-op").await,
        ["Okami", "Overcooked"]
    );
    assert!(titles(&app, "label=unknown").await.is_empty());
}

#[tokio::test]
async fn only_managers_change_labels() {
    let app = TestApp::new().await;
    let request = Request::post("/api/labels")
        .header("x-remote-user-id", "viewer")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({"name": "Retro", "color": "gray"}).to_string(),
        ))
        .unwrap();
    let (status, _) = app.request(SUPERVISOR, request).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn imports_add_labels_and_exports_list_them() {
    let app = TestApp::new().await;
    label(&app, "Retro", "orange").await;
    let okami = game(&app, "Okami").await;
    app.json(
        "PUT",
        &format!("/api/games/{okami}"),
        Some(json!({"title": "Okami", "label_ids": []})),
    )
    .await;

    let csv = "game_title,tags,platform,region,format\n\
               Okami,retro; Favorites,,,\n\
               Hades,\"Favorites, Roguelike\",Switch,eu,digital\n\
               Hades,favorites,,,\n";
    let (status, preview) = app.post_csv("/api/import/preview", csv).await;
    assert_eq!(status, StatusCode::OK, "{preview}");
    assert_eq!(preview["new_labels"], json!(["Favorites", "Roguelike"]));
    assert_eq!(preview["summary"]["updated_games"], 1, "Okami gains labels");
    let (status, result) = app.post_csv("/api/import", csv).await;
    assert_eq!(status, StatusCode::OK, "{result}");

    let (_, labels) = app.json("GET", "/api/labels", None).await;
    let counts: Vec<(String, i64, String)> = labels
        .as_array()
        .unwrap()
        .iter()
        .map(|l: &Value| {
            (
                l["name"].as_str().unwrap().to_owned(),
                l["game_count"].as_i64().unwrap(),
                l["color"].as_str().unwrap().to_owned(),
            )
        })
        .collect();
    assert_eq!(
        counts,
        [
            ("Favorites".to_owned(), 2, "gray".to_owned()),
            ("Retro".to_owned(), 1, "orange".to_owned()),
            ("Roguelike".to_owned(), 1, "gray".to_owned()),
        ]
    );

    let (_, bytes) = app.get("/api/export.csv").await;
    let exported = String::from_utf8(bytes).unwrap();
    assert!(
        exported.contains(",Hades,,Favorites; Roguelike,Nintendo Switch,"),
        "{exported}"
    );
    assert!(exported.contains(",Okami,,Favorites; Retro,"), "{exported}");
}
