//! Store accounts and the digital copies they own.

mod common;

use axum::http::StatusCode;
use common::{MANAGER, TestApp};
use serde_json::{Value, json};

async fn new_game(app: &TestApp, title: &str) -> i64 {
    let (status, game) = app
        .json("POST", "/api/games", Some(json!({"title": title})))
        .await;
    assert_eq!(status, StatusCode::CREATED, "{game}");
    game["id"].as_i64().unwrap()
}

async fn new_account(app: &TestApp, body: Value) -> Value {
    let (status, account) = app.json("POST", "/api/accounts", Some(body)).await;
    assert_eq!(status, StatusCode::CREATED, "{account}");
    account
}

async fn steam_id(app: &TestApp) -> i64 {
    let (_, stores) = app.json("GET", "/api/stores", None).await;
    stores
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["name"] == "Steam")
        .unwrap()["id"]
        .as_i64()
        .unwrap()
}

#[tokio::test]
async fn accounts_are_added_changed_and_deleted() {
    let app = TestApp::new().await;
    let pc = app.platform_id("pc").await;
    let steam = steam_id(&app).await;
    let account = new_account(
        &app,
        json!({"name": "Steam account", "store_id": steam, "user_id": MANAGER, "platform_ids": [pc]}),
    )
    .await;
    let id = account["id"].as_i64().unwrap();
    assert_eq!(account["slug"], "steam-account");
    assert_eq!(account["store_id"], steam);
    assert_eq!(account["person"]["id"], MANAGER);
    assert_eq!(account["platform_ids"], json!([pc]));

    let (status, updated) = app
        .json(
            "PUT",
            &format!("/api/accounts/{id}"),
            Some(json!({"name": "Steam", "active": false})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{updated}");
    assert_eq!(updated["slug"], "steam-account", "slug is stable");
    assert_eq!(updated["active"], false);
    assert_eq!(updated["person"], Value::Null);
    assert_eq!(updated["platform_ids"], json!([]));

    let (status, _) = app
        .json("DELETE", &format!("/api/accounts/{id}"), None)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _) = app
        .json("DELETE", &format!("/api/accounts/{id}"), None)
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn accounts_refuse_what_does_not_exist() {
    let app = TestApp::new().await;
    for (body, code, field) in [
        (
            json!({"name": "A", "store_id": 9999}),
            "unknown_store",
            "store_id",
        ),
        (
            json!({"name": "A", "platform_ids": [9999]}),
            "unknown_platform",
            "platform_ids",
        ),
        (
            json!({"name": "A", "user_id": "not a user"}),
            "invalid_user",
            "user_id",
        ),
        (
            json!({"name": "A", "user_id": "fedcba9876543210fedcba9876543210"}),
            "unknown_user",
            "user_id",
        ),
    ] {
        let (status, error) = app.json("POST", "/api/accounts", Some(body)).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{code}");
        assert_eq!(error, json!({"code": code, "field": field}));
    }
    assert_eq!(app.scalar("SELECT COUNT(*) FROM accounts").await, 0);
    assert_eq!(
        app.scalar("SELECT COUNT(*) FROM users WHERE id = 'fedcba9876543210fedcba9876543210'")
            .await,
        0,
        "naming an owner adds no person"
    );
}

#[tokio::test]
async fn people_lists_who_memory_card_knows() {
    let app = TestApp::new().await;
    new_game(&app, "Celeste").await;
    let (status, people) = app.json("GET", "/api/people", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(people, json!([{"id": MANAGER, "name": null}]));
}

#[tokio::test]
async fn digital_copies_name_their_account() {
    let app = TestApp::new().await;
    let pc = app.platform_id("pc").await;
    let account = new_account(&app, json!({"name": "Steam account"})).await;
    let account_id = account["id"].as_i64().unwrap();
    let game = new_game(&app, "Hades").await;

    let (status, detail) = app
        .json(
            "POST",
            &format!("/api/games/{game}/copies"),
            Some(json!({"platform_id": pc, "region": "ww", "format": "digital", "account_id": account_id})),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "{detail}");
    let copy = &detail["releases"][0]["copies"][0];
    assert_eq!(copy["account_id"], account_id);
    let copy_id = copy["id"].as_i64().unwrap();

    for (body, code) in [
        (
            json!({"platform_id": pc, "region": "ww", "format": "physical", "account_id": account_id}),
            "account_physical",
        ),
        (
            json!({"platform_id": pc, "region": "ww", "format": "digital", "account_id": 9999}),
            "unknown_account",
        ),
    ] {
        let (status, error) = app
            .json("PUT", &format!("/api/copies/{copy_id}"), Some(body))
            .await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{code}");
        assert_eq!(error, json!({"code": code, "field": "account_id"}));
    }

    let (_, games) = app
        .json("GET", "/api/games?account=steam-account", None)
        .await;
    assert_eq!(games.as_array().unwrap().len(), 1);
    let (_, accounts) = app.json("GET", "/api/accounts", None).await;
    assert_eq!(accounts[0]["copy_count"], 1);

    let (status, _) = app
        .json("DELETE", &format!("/api/accounts/{account_id}"), None)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (_, detail) = app.json("GET", &format!("/api/games/{game}"), None).await;
    assert_eq!(
        detail["releases"][0]["copies"][0]["account_id"],
        Value::Null,
        "the copy stays, with no account"
    );
}

#[tokio::test]
async fn bundle_contents_follow_the_bundle_account() {
    let app = TestApp::new().await;
    let switch = app.platform_id("switch").await;
    let first = new_account(&app, json!({"name": "Nintendo Account"})).await["id"]
        .as_i64()
        .unwrap();
    let second = new_account(&app, json!({"name": "Second Nintendo Account"})).await["id"]
        .as_i64()
        .unwrap();
    let bundle = new_game(&app, "Super Mario 3D All-Stars").await;
    let sunshine = new_game(&app, "Super Mario Sunshine").await;
    let (_, detail) = app
        .json(
            "POST",
            &format!("/api/games/{bundle}/copies"),
            Some(json!({"platform_id": switch, "region": "eu", "format": "digital", "account_id": first})),
        )
        .await;
    let bundle_copy = detail["releases"][0]["copies"][0]["id"].as_i64().unwrap();
    let (status, body) = app
        .json(
            "POST",
            &format!("/api/copies/{bundle_copy}/contents"),
            Some(json!({"game_id": sunshine})),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    let content = |detail: &Value| detail["releases"][0]["copies"][0]["account_id"].clone();
    let (_, detail) = app
        .json("GET", &format!("/api/games/{sunshine}"), None)
        .await;
    assert_eq!(content(&detail), first);

    let (status, body) = app
        .json(
            "PUT",
            &format!("/api/copies/{bundle_copy}"),
            Some(json!({"platform_id": switch, "region": "eu", "format": "digital", "account_id": second})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (_, detail) = app
        .json("GET", &format!("/api/games/{sunshine}"), None)
        .await;
    assert_eq!(content(&detail), second);
}

#[tokio::test]
async fn a_file_without_accounts_turning_a_copy_physical_raises_nothing() {
    let app = TestApp::new().await;
    let (status, result) = app
        .post_csv(
            "/api/import",
            "game_title,platform,region,format,account\nHades,Switch,eu,digital,Nintendo Account\n",
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{result}");
    let copy_id = app.scalar("SELECT id FROM copies").await;

    let csv = format!("copy_id,game_title,format\n{copy_id},Hades,physical\n");
    let (status, preview) = app.post_csv("/api/import/preview", &csv).await;
    assert_eq!(status, StatusCode::OK, "{preview}");
    assert_eq!(preview["rows"][0]["issues"], json!([]), "{preview}");
}
