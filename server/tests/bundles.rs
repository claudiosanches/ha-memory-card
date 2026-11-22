//! Copies that came inside another copy.

mod common;

use axum::http::StatusCode;
use common::TestApp;
use serde_json::{Value, json};

async fn new_game(app: &TestApp, title: &str) -> i64 {
    let (_, game) = app
        .json("POST", "/api/games", Some(json!({"title": title})))
        .await;
    game["id"].as_i64().unwrap()
}

fn first_copy(detail: &Value) -> &Value {
    &detail["releases"][0]["copies"][0]
}

#[tokio::test]
async fn bundle_contents_follow_the_bundle_copy() {
    let app = TestApp::new().await;
    let switch = app.platform_id("switch").await;
    let wii_u = app.platform_id("wii-u").await;
    let bundle = new_game(&app, "Super Mario 3D All-Stars").await;
    let sunshine = new_game(&app, "Super Mario Sunshine").await;
    let galaxy = new_game(&app, "Super Mario Galaxy").await;
    let copy = json!({"platform_id": switch, "region": "eu", "format": "physical",
                      "condition": "cib", "price": "59.99", "currency": "EUR"});
    let (_, detail) = app
        .json("POST", &format!("/api/games/{bundle}/copies"), Some(copy))
        .await;
    let bundle_copy = first_copy(&detail)["id"].as_i64().unwrap();

    let contents = format!("/api/copies/{bundle_copy}/contents");
    for game in [sunshine, galaxy] {
        let (status, detail) = app
            .json("POST", &contents, Some(json!({"game_id": game})))
            .await;
        assert_eq!(status, StatusCode::CREATED, "{detail}");
    }
    let (_, detail) = app.json("GET", &format!("/api/games/{bundle}"), None).await;
    let links = &first_copy(&detail)["contents"];
    assert_eq!(links[0]["title"], "Super Mario Galaxy");
    assert_eq!(links[1]["game_id"], sunshine);

    let (_, detail) = app
        .json("GET", &format!("/api/games/{sunshine}"), None)
        .await;
    let inside = first_copy(&detail);
    assert_eq!(detail["releases"][0]["platform_id"], switch);
    assert_eq!(detail["releases"][0]["region"], "eu");
    assert_eq!(inside["condition"], "cib");
    assert_eq!(
        inside["price"],
        Value::Null,
        "the price stays on the bundle"
    );
    assert_eq!(inside["bundle"]["copy_id"], bundle_copy);
    assert_eq!(inside["bundle"]["title"], "Super Mario 3D All-Stars");
    let inside_id = inside["id"].as_i64().unwrap();

    // Editing a copy inside a bundle changes only its notes.
    let (status, detail) = app
        .json(
            "PUT",
            &format!("/api/copies/{inside_id}"),
            Some(json!({"platform_id": wii_u, "region": "na", "format": "digital", "notes": "Shine Get"})),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(first_copy(&detail)["notes"], "Shine Get");
    assert_eq!(first_copy(&detail)["format"], "physical");
    assert_eq!(detail["releases"][0]["platform_id"], switch);

    // Editing the bundle copy moves and updates what is inside.
    let (status, _) = app
        .json(
            "PUT",
            &format!("/api/copies/{bundle_copy}"),
            Some(json!({"status": "wanted", "platform_id": wii_u, "region": "na", "format": "physical", "condition": "sealed"})),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let (_, detail) = app
        .json("GET", &format!("/api/games/{sunshine}"), None)
        .await;
    let moved = detail["releases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| !r["copies"].as_array().unwrap().is_empty())
        .unwrap();
    assert_eq!(moved["platform_id"], wii_u);
    assert_eq!(moved["region"], "na");
    assert_eq!(moved["copies"][0]["status"], "wanted");
    assert_eq!(moved["copies"][0]["condition"], "sealed");
    assert_eq!(moved["copies"][0]["notes"], "Shine Get");

    // A bundle counts once among copies; each game inside counts as a game.
    let got =
        json!({"status": "owned", "platform_id": wii_u, "region": "na", "format": "physical"});
    app.json("PUT", &format!("/api/copies/{bundle_copy}"), Some(got))
        .await;
    let (_, stats) = app.json("GET", "/api/stats", None).await;
    assert_eq!(stats["copies"], 1);
    assert_eq!(stats["games"], 3);

    // Deleting the bundle copy takes its contents with it.
    app.json("DELETE", &format!("/api/copies/{bundle_copy}"), None)
        .await;
    assert_eq!(app.scalar("SELECT COUNT(*) FROM copies").await, 0);
}

#[tokio::test]
async fn bundle_contents_are_checked() {
    let app = TestApp::new().await;
    let switch = app.platform_id("switch").await;
    let bundle = new_game(&app, "Collection").await;
    let inner = new_game(&app, "Inner").await;
    let (_, detail) = app
        .json(
            "POST",
            &format!("/api/games/{bundle}/copies"),
            Some(json!({"platform_id": switch, "region": "ww", "format": "digital"})),
        )
        .await;
    let bundle_copy = first_copy(&detail)["id"].as_i64().unwrap();
    let path = format!("/api/copies/{bundle_copy}/contents");
    app.json("POST", &path, Some(json!({"game_id": inner})))
        .await;

    let (status, body) = app
        .json("POST", &path, Some(json!({"game_id": inner})))
        .await;
    assert_eq!(
        (status, body["code"].clone()),
        (StatusCode::CONFLICT, json!("already_in_bundle"))
    );
    let (status, body) = app
        .json("POST", &path, Some(json!({"game_id": bundle})))
        .await;
    assert_eq!(
        (status, body["code"].clone()),
        (StatusCode::UNPROCESSABLE_ENTITY, json!("same_game"))
    );
    let (status, body) = app.json("POST", &path, Some(json!({"game_id": 999}))).await;
    assert_eq!(
        (status, body["code"].clone()),
        (StatusCode::UNPROCESSABLE_ENTITY, json!("unknown_game"))
    );
    let (status, _) = app
        .json(
            "POST",
            "/api/copies/999/contents",
            Some(json!({"game_id": inner})),
        )
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let inner_copy: i64 = app
        .scalar("SELECT id FROM copies WHERE bundle_id IS NOT NULL")
        .await;
    let (status, body) = app
        .json(
            "POST",
            &format!("/api/copies/{inner_copy}/contents"),
            Some(json!({"game_id": bundle})),
        )
        .await;
    assert_eq!(
        (status, body["code"].clone()),
        (StatusCode::CONFLICT, json!("nested_bundle"))
    );
}

fn row_issues(preview: &Value, line: u64) -> Vec<(String, String, bool)> {
    preview["rows"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["line"] == line)
        .flat_map(|row| row["issues"].as_array().unwrap().clone())
        .map(|i| {
            let text = |v: &Value| v.as_str().unwrap().to_owned();
            (
                text(&i["column"]),
                text(&i["code"]),
                i["error"].as_bool().unwrap(),
            )
        })
        .collect()
}

#[tokio::test]
async fn csv_rows_put_copies_in_bundles() {
    let app = TestApp::new().await;
    let header =
        "copy_id,game_title,platform,region,format,condition,price,currency,bundle_copy_id";
    let (status, result) = app
        .post_csv(
            "/api/import",
            &format!(
                "{header}\n\
                 ,Super Mario 3D All-Stars,Switch,eu,physical,cib,59.99,EUR,\n\
                 ,Super Mario Sunshine,Switch,eu,physical,,,,\n"
            ),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{result}");
    let copy_of = async |title: &str| -> i64 {
        sqlx::query_scalar(
            "SELECT c.id FROM copies c JOIN releases r ON r.id = c.release_id
             JOIN games g ON g.id = r.game_id WHERE g.title = ?",
        )
        .bind(title)
        .fetch_one(&app.pool)
        .await
        .unwrap()
    };
    let bundle = copy_of("Super Mario 3D All-Stars").await;
    let sunshine = copy_of("Super Mario Sunshine").await;
    let sunshine_now = async || -> (Option<String>, Option<i64>) {
        sqlx::query_as("SELECT condition, bundle_id FROM copies WHERE id = ?")
            .bind(sunshine)
            .fetch_one(&app.pool)
            .await
            .unwrap()
    };

    let csv = format!(
        "{header}\n\
         {sunshine},Super Mario Sunshine,Wii U,na,physical,,10,USD,{bundle}\n"
    );
    let (_, preview) = app.post_csv("/api/import/preview", &csv).await;
    assert_eq!(
        row_issues(&preview, 2),
        [("price".into(), "ignored_in_bundle".into(), false)]
    );
    let (status, result) = app.post_csv("/api/import", &csv).await;
    assert_eq!(status, StatusCode::OK, "{result}");
    let (platform, condition, price, in_bundle): (String, Option<String>, Option<i64>, i64) =
        sqlx::query_as(
            "SELECT p.slug, c.condition, c.price, c.bundle_id FROM copies c
             JOIN releases r ON r.id = c.release_id JOIN platforms p ON p.id = r.platform_id
             WHERE c.id = ?",
        )
        .bind(sunshine)
        .fetch_one(&app.pool)
        .await
        .unwrap();
    assert_eq!(
        (platform.as_str(), condition.as_deref(), price, in_bundle),
        ("switch", Some("cib"), None, bundle),
        "it follows the bundle copy"
    );

    let (_, bytes) = app.get("/api/export.csv").await;
    let exported = String::from_utf8(bytes).unwrap();
    let row = exported
        .lines()
        .find(|line| line.contains("Super Mario Sunshine"))
        .unwrap();
    assert!(row.ends_with(&format!(",{bundle}")), "{row}");
    let (_, preview) = app.post_csv("/api/import/preview", &exported).await;
    assert_eq!(preview["summary"]["unchanged_copies"], 2, "{preview}");

    let edited = exported.replacen(",cib,", ",loose,", 1);
    app.post_csv("/api/import", &edited).await;
    assert_eq!(
        sunshine_now().await.0.as_deref(),
        Some("loose"),
        "contents follow an edited bundle row"
    );

    let bad = format!(
        "{header}\n\
         ,Galaxy,Switch,eu,physical,,,,999\n\
         ,Galaxy,Switch,eu,physical,,,,{sunshine}\n\
         {bundle},Super Mario 3D All-Stars,Switch,eu,physical,,,,{bundle}\n\
         ,Super Mario 3D All-Stars,Switch,eu,physical,,,,{bundle}\n"
    );
    let (_, preview) = app.post_csv("/api/import/preview", &bad).await;
    let error = |code: &str| vec![("bundle_copy_id".to_owned(), code.to_owned(), true)];
    assert_eq!(row_issues(&preview, 2), error("unknown_bundle"));
    assert_eq!(row_issues(&preview, 3), error("nested_bundle"));
    assert_eq!(row_issues(&preview, 4), error("nested_bundle"));
    assert_eq!(row_issues(&preview, 5), error("same_game"));

    let csv = format!("{header}\n{sunshine},Super Mario Sunshine,Switch,eu,physical,loose,,,\n");
    app.post_csv("/api/import", &csv).await;
    assert_eq!(sunshine_now().await.1, None, "a blank cell takes it out");
}
