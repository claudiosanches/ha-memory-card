//! Devices as the household's hardware: consoles that hold copies, and
//! controllers, accessories and toys that do not.
// Tests set up and read rows with SQL.
#![allow(clippy::disallowed_methods)]

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::{PHOTO as JPEG, TestApp};
use serde_json::{Value, json};

async fn add(app: &TestApp, body: Value) -> Value {
    let (status, device) = app.json("POST", "/api/devices", Some(body)).await;
    assert_eq!(status, StatusCode::CREATED, "{device}");
    device
}

#[tokio::test]
async fn hardware_keeps_what_it_cost_and_what_it_is_worth() {
    let app = TestApp::new().await;
    let switch = app.platform_id("switch").await;
    let console = add(
        &app,
        json!({
            "name": "Living room Switch", "kind": "console", "platform_ids": [switch],
            "region": "eu", "condition": "cib", "purchased_on": "2021-10-08",
            "price": "349,99", "currency": "EUR", "estimated_value": "250", "estimated_currency": "EUR",
            "serial_number": "XKW123", "barcode": "0045496453435"
        }),
    )
    .await;
    assert_eq!(console["price"], "349.99");
    assert_eq!(console["status"], "owned");
    assert_eq!(console["condition"], "cib");

    let controller = add(
        &app,
        json!({"name": "Pro Controller", "kind": "controller", "platform_id": switch,
               "price": "69.99", "currency": "EUR"}),
    )
    .await;
    assert_eq!(controller["platform_id"], switch);
    add(
        &app,
        json!({"name": "Link amiibo", "kind": "toy", "status": "wanted",
               "price": "15", "currency": "EUR"}),
    )
    .await;

    // Only consoles and the like play platforms; only hardware is for one.
    let (status, body) = app
        .json(
            "POST",
            "/api/devices",
            Some(json!({"name": "Joy-Con", "kind": "controller", "platform_ids": [switch]})),
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["code"], "not_for_this_kind");
    let (status, _) = app
        .json(
            "POST",
            "/api/devices",
            Some(json!({"name": "eShop", "kind": "account", "condition": "cib"})),
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    let (_, stats) = app.json("GET", "/api/stats", None).await;
    let hardware = &stats["hardware"];
    assert_eq!(hardware["owned"], 2);
    assert_eq!(hardware["wanted"], 1);
    assert_eq!(
        hardware["spent"],
        json!([{"currency": "EUR", "amount": "419.98", "copies": 2}])
    );
    assert_eq!(
        hardware["estimated"],
        json!([{"currency": "EUR", "amount": "250.00", "copies": 1}])
    );
    // The games' totals leave hardware out.
    assert_eq!(stats["spent"], json!([]));
}

#[tokio::test]
async fn copies_live_only_on_devices_that_hold_games() {
    let app = TestApp::new().await;
    let switch = app.platform_id("switch").await;
    let console = add(
        &app,
        json!({"name": "Switch", "kind": "console", "platform_ids": [switch]}),
    )
    .await;
    let controller = add(
        &app,
        json!({"name": "Pro Controller", "kind": "controller"}),
    )
    .await;
    let (_, game) = app
        .json("POST", "/api/games", Some(json!({"title": "Okami"})))
        .await;
    let game = game["id"].as_i64().unwrap();

    let copy = |device: &Value| json!({"platform_id": switch, "region": "eu", "format": "physical", "device_id": device["id"]});
    let (status, body) = app
        .json(
            "POST",
            &format!("/api/games/{game}/copies"),
            Some(copy(&controller)),
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["code"], "device_holds_no_games");
    let (status, _) = app
        .json(
            "POST",
            &format!("/api/games/{game}/copies"),
            Some(copy(&console)),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);

    // A device with copies cannot become hardware that holds none.
    let (status, body) = app
        .json(
            "PUT",
            &format!("/api/devices/{}", console["id"]),
            Some(json!({"name": "Switch", "kind": "accessory"})),
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["code"], "device_has_copies");
}

#[tokio::test]
async fn hardware_keeps_photos_until_it_goes() {
    let app = TestApp::new().await;
    let device = add(&app, json!({"name": "GameCube", "kind": "console"})).await;
    let id = device["id"].as_i64().unwrap();
    let request = Request::post(format!("/api/devices/{id}/photos"))
        .header("content-type", "application/octet-stream")
        .body(Body::from(JPEG.to_vec()))
        .unwrap();
    let (status, body) = app.request(common::SUPERVISOR, request).await;
    assert_eq!(status, StatusCode::CREATED);
    let device: Value = serde_json::from_slice(&body).unwrap();
    let path = device["photos"][0]["path"].as_str().unwrap().to_owned();
    assert!(path.starts_with(&format!("device-{id}/")), "{path}");
    let (status, _) = app.get(&format!("/covers/{path}")).await;
    assert_eq!(status, StatusCode::OK);

    let (status, _) = app
        .json("DELETE", &format!("/api/devices/{id}"), None)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _) = app.get(&format!("/covers/{path}")).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn hardware_csv_round_trips_and_updates_by_name() {
    let app = TestApp::new().await;
    let switch = app.platform_id("switch").await;
    add(
        &app,
        json!({"name": "Switch", "kind": "console", "platform_ids": [switch],
               "price": "299.99", "currency": "USD", "notes": "=SUM(A1)"}),
    )
    .await;

    let (status, bytes) = app.get("/api/hardware.csv").await;
    assert_eq!(status, StatusCode::OK);
    let csv = String::from_utf8(bytes).unwrap();
    assert!(csv.contains("Switch,console,,,Nintendo Switch"), "{csv}");
    // Formulas are escaped.
    assert!(csv.contains("'=SUM(A1)"), "{csv}");

    // Reimporting the export changes nothing it would not keep.
    let (status, preview) = app.post_csv("/api/hardware/import/preview", &csv).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(preview["rows"][0]["issues"], json!([]), "{preview}");

    let file = "name,kind,platform,store,price,currency,status\n\
                Switch,,,,320,USD,\n\
                Pro Controller,controller,switch,Game Shop,69.99,USD,\n\
                DualSense,gamepad,,,,,wishlist\n";
    let (_, preview) = app.post_csv("/api/hardware/import/preview", file).await;
    assert_eq!(preview["file_issue"], Value::Null);
    let issues: Vec<&Value> = preview["rows"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|r| r["issues"].as_array().unwrap())
        .collect();
    assert!(issues.is_empty(), "{preview}");
    assert!(preview["rows"][0]["device_id"].is_i64());
    assert!(preview["rows"][1]["device_id"].is_null());

    let (status, result) = app.post_csv("/api/hardware/import", file).await;
    assert_eq!(status, StatusCode::OK, "{result}");
    assert_eq!(result, json!({"created": 2, "updated": 1, "new_stores": 1}));
    let (_, devices) = app.json("GET", "/api/devices", None).await;
    let by_name = |name: &str| {
        devices
            .as_array()
            .unwrap()
            .iter()
            .find(|d| d["name"] == name)
            .unwrap()
            .clone()
    };
    assert_eq!(by_name("Switch")["price"], "320.00");
    assert_eq!(by_name("Switch")["platform_ids"], json!([switch]));
    assert_eq!(by_name("Switch")["notes"], "=SUM(A1)");
    assert_eq!(by_name("Pro Controller")["platform_id"], switch);
    assert_eq!(by_name("DualSense")["status"], "wanted");

    let bad = "name,kind,plays\nWii,console,Not a platform\n,controller,\n";
    let (_, preview) = app.post_csv("/api/hardware/import/preview", bad).await;
    assert_eq!(preview["rows"][0]["issues"][0]["code"], "unknown_platform");
    assert_eq!(preview["rows"][1]["issues"][0]["code"], "required");
    let (status, _) = app.post_csv("/api/hardware/import", bad).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    let (_, preview) = app
        .post_csv("/api/hardware/import/preview", "kind\nconsole\n")
        .await;
    assert_eq!(preview["file_issue"], "missing_column");
}
