//! Barcodes on copies and the lookup behind a scan.

mod common;

use axum::Router;
use axum::body::Body;
use axum::extract::Query;
use axum::http::{Request, StatusCode};
use axum::routing::get;
use common::{SUPERVISOR, TestApp};
use serde_json::{Value, json};
use std::collections::HashMap;

/// A stand-in for UPCitemdb that knows one code and is over its limit for another.
async fn database() -> String {
    let stub = Router::new().route(
        "/lookup",
        get(|Query(query): Query<HashMap<String, String>>| async move {
            match query.get("upc").map(String::as_str) {
                Some("4902370536485") => (
                    StatusCode::OK,
                    axum::Json(json!({"code": "OK", "total": 1, "items": [
                        {"title": " Super Mario Odyssey - Nintendo Switch ", "brand": "Nintendo"}
                    ]})),
                ),
                Some("96385074") => (StatusCode::TOO_MANY_REQUESTS, axum::Json(json!({}))),
                _ => (
                    StatusCode::NOT_FOUND,
                    axum::Json(json!({"code": "NOT_FOUND"})),
                ),
            }
        }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}/lookup", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, stub).await.unwrap() });
    base
}

async fn look_up(app: &TestApp, code: &str) -> (StatusCode, Value) {
    app.json("GET", &format!("/api/barcodes/{code}"), None)
        .await
}

#[tokio::test]
async fn scans_find_copies_then_ask_the_database() {
    let url = database().await;
    let app = TestApp::with_env(&[("MEMORY_CARD_BARCODE_URL", &url)]).await;
    let switch = app.platform_id("switch").await;
    let (_, game) = app
        .json("POST", "/api/games", Some(json!({"title": "Hades"})))
        .await;
    let copy = json!({"platform_id": switch, "region": "eu", "format": "physical", "barcode": "0 45496 59002-4"});
    let (status, detail) = app
        .json(
            "POST",
            &format!("/api/games/{}/copies", game["id"]),
            Some(copy),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "{detail}");
    assert_eq!(
        detail["releases"][0]["copies"][0]["barcode"],
        "045496590024"
    );

    let (status, owned) = look_up(&app, "045496590024").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(owned["lookup"], "in_collection");
    assert_eq!(owned["copies"][0]["title"], "Hades");
    assert_eq!(owned["copies"][0]["region"], "eu");

    let (_, found) = look_up(&app, "4902370536485").await;
    assert_eq!(found["lookup"], "found");
    assert_eq!(
        found["product"],
        json!({"title": "Super Mario Odyssey - Nintendo Switch", "brand": "Nintendo"})
    );
    assert_eq!(
        look_up(&app, "4006381333931").await.1["lookup"],
        "not_found"
    );
    assert_eq!(look_up(&app, "96385074").await.1["lookup"], "failed");

    let (status, body) = look_up(&app, "4902370536484").await;
    assert_eq!(
        (status, body["code"].clone()),
        (StatusCode::UNPROCESSABLE_ENTITY, json!("invalid_barcode"))
    );
    let bad =
        json!({"platform_id": switch, "region": "eu", "format": "physical", "barcode": "12345"});
    let (status, body) = app
        .json(
            "POST",
            &format!("/api/games/{}/copies", game["id"]),
            Some(bad),
        )
        .await;
    assert_eq!(
        (status, body["field"].clone()),
        (StatusCode::UNPROCESSABLE_ENTITY, json!("barcode"))
    );
}

#[tokio::test]
async fn offline_scans_only_check_the_collection() {
    let url = database().await;
    let app = TestApp::with_env(&[
        ("MEMORY_CARD_BARCODE_URL", &url),
        ("MEMORY_CARD_EXTERNAL_SERVICES", "false"),
    ])
    .await;
    let (_, body) = look_up(&app, "4902370536485").await;
    assert_eq!(body["lookup"], "offline");
    assert_eq!(body["product"], Value::Null);
}

#[tokio::test]
async fn viewers_only_check_the_collection() {
    let url = database().await;
    let app = TestApp::with_env(&[("MEMORY_CARD_BARCODE_URL", &url)]).await;
    let request = Request::get("/api/barcodes/4902370536485")
        .header("x-remote-user-id", "viewer")
        .body(Body::empty())
        .unwrap();
    let (status, body) = app.raw(SUPERVISOR, request).await;
    assert_eq!(status, StatusCode::OK);
    let body: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(body["lookup"], "not_in_collection");
    assert_eq!(body["product"], Value::Null);
}

#[tokio::test]
async fn an_oversized_answer_is_refused() {
    // Streamed, so no length says how much is coming.
    let stub = Router::new().route(
        "/lookup",
        get(|| async {
            let chunks = futures_util::stream::iter(
                (0..64).map(|_| Ok::<_, std::convert::Infallible>(vec![b' '; 32 * 1024])),
            );
            Body::from_stream(chunks)
        }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/lookup", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, stub).await.unwrap() });

    let result = memory_card::barcodes::lookup(&memory_card::http::shared(), &url, "1").await;
    assert!(
        matches!(result, Err(memory_card::barcodes::LookupError::TooLarge)),
        "{result:?}"
    );
}

#[tokio::test]
async fn product_text_is_cut_to_what_the_api_takes() {
    let stub = Router::new().route(
        "/lookup",
        get(|| async {
            axum::Json(json!({"items": [{"title": "(".repeat(5000), "brand": "b".repeat(300)}]}))
        }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/lookup", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, stub).await.unwrap() });

    let product = memory_card::barcodes::lookup(&memory_card::http::shared(), &url, "1")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(product.title, "(".repeat(200));
    assert_eq!(product.brand.as_deref(), Some("b".repeat(100).as_str()));
}
