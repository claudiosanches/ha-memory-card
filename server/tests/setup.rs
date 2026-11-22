//! Platforms, devices and stores endpoints.

mod common;

use axum::http::StatusCode;
use common::TestApp;
use serde_json::{Value, json};

fn names(list: &Value) -> Vec<&str> {
    list.as_array()
        .unwrap()
        .iter()
        .map(|item| item["name"].as_str().unwrap())
        .collect()
}

// ---- stores ----

#[tokio::test]
async fn stores_start_with_digital_storefronts() {
    let app = TestApp::new().await;
    let (_, list) = app.json("GET", "/api/stores", None).await;
    assert_eq!(
        names(&list),
        [
            "Epic Games Store",
            "GOG",
            "itch.io",
            "Microsoft Store",
            "Nintendo eShop",
            "PlayStation Store",
            "Steam"
        ]
    );
}

#[tokio::test]
async fn store_crud() {
    let app = TestApp::new().await;

    let (status, created) = app
        .json(
            "POST",
            "/api/stores",
            Some(json!({"name": "  Best Buy ", "notes": " "})),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(created["name"], "Best Buy");
    assert_eq!(created["notes"], Value::Null);
    assert_eq!(created["copy_count"], 0);
    let id = created["id"].as_i64().unwrap();

    app.json("POST", "/api/stores", Some(json!({"name": "Amazon"})))
        .await;
    let (_, list) = app.json("GET", "/api/stores", None).await;
    assert_eq!(names(&list)[..2], ["Amazon", "Best Buy"]);

    let (status, updated) = app
        .json(
            "PUT",
            &format!("/api/stores/{id}"),
            Some(json!({"name": "Best Buy US", "notes": "Online"})),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(updated["name"], "Best Buy US");
    assert_eq!(updated["notes"], "Online");

    let (status, body) = app.json("DELETE", &format!("/api/stores/{id}"), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert_eq!(body, Value::Null);
    let (status, body) = app.json("DELETE", &format!("/api/stores/{id}"), None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["code"], "not_found");
}

#[tokio::test]
async fn store_names_conflict_ignoring_case() {
    let app = TestApp::new().await;
    app.json("POST", "/api/stores", Some(json!({"name": "Amazon"})))
        .await;
    let (status, body) = app
        .json("POST", "/api/stores", Some(json!({"name": "AMAZON"})))
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["code"], "name_taken");
}

#[tokio::test]
async fn store_slugs_come_from_the_name_or_the_input() {
    let app = TestApp::new().await;
    let (_, list) = app.json("GET", "/api/stores", None).await;
    assert_eq!(list[2]["slug"], "itch-io");

    let (status, created) = app
        .json("POST", "/api/stores", Some(json!({"name": "Café Jogos"})))
        .await;
    assert_eq!(status, StatusCode::CREATED, "{created}");
    assert_eq!(created["slug"], "cafe-jogos");
    let id = created["id"].as_i64().unwrap();

    let (_, created) = app
        .json("POST", "/api/stores", Some(json!({"name": "Café: Jogos"})))
        .await;
    assert_eq!(created["slug"], "cafe-jogos-2");

    let (status, updated) = app
        .json(
            "PUT",
            &format!("/api/stores/{id}"),
            Some(json!({"name": "Cafe", "slug": " Cafe "})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{updated}");
    assert_eq!(updated["slug"], "cafe");
    let (_, updated) = app
        .json(
            "PUT",
            &format!("/api/stores/{id}"),
            Some(json!({"name": "Cafe 2"})),
        )
        .await;
    assert_eq!(updated["slug"], "cafe", "no slug keeps the current one");

    let (status, body) = app
        .json(
            "PUT",
            &format!("/api/stores/{id}"),
            Some(json!({"name": "Cafe", "slug": "steam"})),
        )
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["code"], "slug_taken");

    // The form fills the slug from the name, so a taken name brings a taken slug.
    let (status, body) = app
        .json(
            "POST",
            "/api/stores",
            Some(json!({"name": "STEAM", "slug": "steam"})),
        )
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["code"], "name_taken");

    for bad in ["wii u", "itch.io", "-cafe", "café"] {
        let (status, body) = app
            .json(
                "POST",
                "/api/stores",
                Some(json!({"name": "New", "slug": bad})),
            )
            .await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{bad}");
        assert_eq!(body, json!({"code": "invalid_slug", "field": "slug"}));
    }
}

#[tokio::test]
async fn store_validation_names_the_field() {
    let app = TestApp::new().await;
    let (status, body) = app
        .json("POST", "/api/stores", Some(json!({"name": "   "})))
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body, json!({"code": "required", "field": "name"}));

    let (status, body) = app
        .json(
            "POST",
            "/api/stores",
            Some(json!({"name": "x".repeat(101)})),
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body, json!({"code": "too_long", "field": "name"}));

    let (status, body) = app
        .json("POST", "/api/stores", Some(json!({"nome": "typo"})))
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body, json!({"code": "invalid_body"}));

    let (status, body) = app
        .json("PUT", "/api/stores/abc", Some(json!({"name": "x"})))
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["code"], "not_found");
}

#[tokio::test]
async fn deleting_a_store_keeps_its_copies() {
    let app = TestApp::new().await;
    let (_, store) = app
        .json("POST", "/api/stores", Some(json!({"name": "Amazon"})))
        .await;
    let release = app.release("Zelda", "switch").await;
    sqlx::query("INSERT INTO copies (release_id, format, store_id) VALUES (?, 'physical', ?)")
        .bind(release)
        .bind(store["id"].as_i64())
        .execute(&app.pool)
        .await
        .unwrap();

    let (_, list) = app.json("GET", "/api/stores", None).await;
    let amazon = list
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["id"] == store["id"])
        .unwrap();
    assert_eq!(amazon["copy_count"], 1);

    let (status, _) = app
        .json("DELETE", &format!("/api/stores/{}", store["id"]), None)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let store_id: Option<i64> = sqlx::query_scalar("SELECT store_id FROM copies")
        .fetch_one(&app.pool)
        .await
        .unwrap();
    assert_eq!(store_id, None);
}

// ---- devices ----

#[tokio::test]
async fn device_crud_with_platforms() {
    let app = TestApp::new().await;
    let wii_u = app.platform_id("wii-u").await;
    let wii = app.platform_id("wii").await;

    let (status, created) = app
        .json(
            "POST",
            "/api/devices",
            Some(json!({
                "name": "Wii U",
                "kind": "console",
                "model": "Deluxe 32GB",
                "platform_ids": [wii, wii_u, wii],
            })),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "{created}");
    assert_eq!(created["kind"], "console");
    assert_eq!(created["slug"], "wii-u");
    assert_eq!(created["active"], true);
    // Deduplicated, in platform list order (Wii U sorts before Wii).
    assert_eq!(created["platform_ids"], json!([wii_u, wii]));
    let id = created["id"].as_i64().unwrap();

    let (status, updated) = app
        .json(
            "PUT",
            &format!("/api/devices/{id}"),
            Some(json!({
                "name": "Wii U (living room)",
                "kind": "console",
                "active": false,
                "platform_ids": [wii_u],
            })),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{updated}");
    assert_eq!(updated["name"], "Wii U (living room)");
    assert_eq!(updated["model"], Value::Null);
    assert_eq!(updated["active"], false);
    assert_eq!(updated["platform_ids"], json!([wii_u]));

    let (_, list) = app.json("GET", "/api/devices", None).await;
    assert_eq!(names(&list), ["Wii U (living room)"]);

    let (status, _) = app
        .json("DELETE", &format!("/api/devices/{id}"), None)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let links: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM device_platforms")
        .fetch_one(&app.pool)
        .await
        .unwrap();
    assert_eq!(links, 0);
}

#[tokio::test]
async fn device_rejects_unknown_platform_and_kind() {
    let app = TestApp::new().await;
    let (status, body) = app
        .json(
            "POST",
            "/api/devices",
            Some(json!({"name": "PS5", "kind": "console", "platform_ids": [99999]})),
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        body,
        json!({"code": "unknown_platform", "field": "platform_ids"})
    );

    let (status, body) = app
        .json(
            "POST",
            "/api/devices",
            Some(json!({"name": "PS5", "kind": "toaster"})),
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["code"], "invalid_body");

    let (_, list) = app.json("GET", "/api/devices", None).await;
    assert_eq!(list, json!([]), "nothing is written on failure");
}

#[tokio::test]
async fn device_update_failure_keeps_platforms() {
    let app = TestApp::new().await;
    let ps5 = app.platform_id("ps5").await;
    let (_, a) = app
        .json(
            "POST",
            "/api/devices",
            Some(json!({"name": "PS5", "kind": "console", "platform_ids": [ps5]})),
        )
        .await;
    app.json(
        "POST",
        "/api/devices",
        Some(json!({"name": "PSN main", "kind": "account"})),
    )
    .await;

    let (status, body) = app
        .json(
            "PUT",
            &format!("/api/devices/{}", a["id"]),
            Some(json!({"name": "psn MAIN", "kind": "console", "platform_ids": []})),
        )
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["code"], "name_taken");

    let (_, list) = app.json("GET", "/api/devices", None).await;
    assert_eq!(list[0]["platform_ids"], json!([ps5]));
}

#[tokio::test]
async fn deleting_a_device_keeps_its_copies() {
    let app = TestApp::new().await;
    let (_, device) = app
        .json(
            "POST",
            "/api/devices",
            Some(json!({"name": "Switch", "kind": "console"})),
        )
        .await;
    let release = app.release("Zelda", "switch").await;
    sqlx::query("INSERT INTO copies (release_id, format, device_id) VALUES (?, 'digital', ?)")
        .bind(release)
        .bind(device["id"].as_i64())
        .execute(&app.pool)
        .await
        .unwrap();

    let (_, list) = app.json("GET", "/api/devices", None).await;
    assert_eq!(list[0]["copy_count"], 1);

    app.json("DELETE", &format!("/api/devices/{}", device["id"]), None)
        .await;
    let copies: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM copies WHERE device_id IS NULL")
        .fetch_one(&app.pool)
        .await
        .unwrap();
    assert_eq!(copies, 1);
}

// ---- platforms ----

#[tokio::test]
async fn platform_list_is_in_sort_order_with_usage() {
    let app = TestApp::new().await;
    let release = app.release("Zelda", "switch").await;
    sqlx::query(
        "INSERT INTO releases (game_id, platform_id, region)
         SELECT game_id, platform_id, 'eu' FROM releases WHERE id = ?",
    )
    .bind(release)
    .execute(&app.pool)
    .await
    .unwrap();

    let (status, list) = app.json("GET", "/api/platforms", None).await;
    assert_eq!(status, StatusCode::OK);
    let list = list.as_array().unwrap();
    // 25 curated platforms, the rest of IGDB's catalog and two fantasy consoles.
    assert_eq!(list.len(), 216);
    assert_eq!(list.iter().filter(|p| p["active"] == true).count(), 7);
    assert_eq!(list[0]["slug"], "pc");
    assert_eq!(list[0]["category"], "computer");
    let switch_2 = list.iter().find(|p| p["slug"] == "switch-2").unwrap();
    assert_eq!(switch_2["igdb_platform_id"], 508, "linked by name");
    let dreamcast = list.iter().find(|p| p["igdb_platform_id"] == 23).unwrap();
    assert_eq!(dreamcast["category"], "console");
    assert_eq!(dreamcast["active"], false);
    let tic80 = list.iter().find(|p| p["slug"] == "tic-80").unwrap();
    assert_eq!(tic80["category"], "other");
    assert_eq!(list[0]["igdb_platform_id"], 6);
    let switch = list.iter().find(|p| p["slug"] == "switch").unwrap();
    assert_eq!(switch["game_count"], 1);
    assert_eq!(switch["active"], true);
}

#[tokio::test]
async fn platform_create_update_delete() {
    let app = TestApp::new().await;
    let (status, created) = app
        .json(
            "POST",
            "/api/platforms",
            Some(json!({"name": "Homebrew Console", "abbreviation": "DC"})),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "{created}");
    assert_eq!(created["slug"], "homebrew-console");
    assert_eq!(created["igdb_platform_id"], Value::Null);
    assert_eq!(created["custom"], true);
    assert_eq!(
        created["sort_order"], 60,
        "appended after the last active platform (50)"
    );
    let id = created["id"].as_i64().unwrap();

    let (status, body) = app
        .json("POST", "/api/platforms", Some(json!({"name": "wii u"})))
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["code"], "name_taken");

    let (status, updated) = app
        .json(
            "PUT",
            &format!("/api/platforms/{id}"),
            Some(json!({"name": "Homebrew Box", "active": false})),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(updated["name"], "Homebrew Box");
    assert_eq!(updated["abbreviation"], Value::Null);
    assert_eq!(updated["active"], false);
    assert_eq!(updated["slug"], "homebrew-console", "slug is stable");

    let (status, body) = app
        .json(
            "PUT",
            &format!("/api/platforms/{id}"),
            Some(json!({"name": "Homebrew Box", "slug": "switch"})),
        )
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["code"], "slug_taken");
    let (_, updated) = app
        .json(
            "PUT",
            &format!("/api/platforms/{id}"),
            Some(json!({"name": "Homebrew Box", "slug": "homebrew"})),
        )
        .await;
    assert_eq!(updated["slug"], "homebrew");

    let (status, _) = app
        .json("DELETE", &format!("/api/platforms/{id}"), None)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn platform_in_use_cannot_be_deleted() {
    let app = TestApp::new().await;
    let wii = app.platform_id("wii").await;
    let three_ds = app.platform_id("3ds").await;
    app.release("Mario Kart Wii", "wii").await;
    app.json(
        "POST",
        "/api/devices",
        Some(json!({"name": "New 3DS", "kind": "handheld", "platform_ids": [three_ds]})),
    )
    .await;

    for id in [wii, three_ds] {
        let (status, body) = app
            .json("DELETE", &format!("/api/platforms/{id}"), None)
            .await;
        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(body["code"], "platform_in_use");
    }
}

#[tokio::test]
async fn shipped_platforms_cannot_be_deleted() {
    let app = TestApp::new().await;
    for slug in ["ps3", "tic-80"] {
        let id = app.platform_id(slug).await;
        let (status, body) = app
            .json("DELETE", &format!("/api/platforms/{id}"), None)
            .await;
        assert_eq!(status, StatusCode::CONFLICT, "{slug}");
        assert_eq!(body["code"], "platform_builtin");
    }
    let (_, list) = app.json("GET", "/api/platforms", None).await;
    assert!(
        list.as_array()
            .unwrap()
            .iter()
            .all(|p| p["custom"] == false)
    );
}

#[tokio::test]
async fn platform_reorder() {
    let app = TestApp::new().await;
    let (_, list) = app.json("GET", "/api/platforms", None).await;
    let mut ids: Vec<i64> = list
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["id"].as_i64().unwrap())
        .collect();
    let last = list.as_array().unwrap().last().unwrap()["slug"].clone();
    ids.reverse();

    let (status, reordered) = app
        .json("PUT", "/api/platforms/order", Some(json!({"ids": ids})))
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(reordered[0]["slug"], last);
    assert_eq!(reordered[0]["sort_order"], 10);

    ids.pop();
    let (status, body) = app
        .json("PUT", "/api/platforms/order", Some(json!({"ids": ids})))
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body, json!({"code": "invalid_order", "field": "ids"}));
}
