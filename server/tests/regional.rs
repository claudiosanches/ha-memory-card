//! Home Assistant's language and currency as defaults.

mod common;

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use axum::Router;
use axum::http::{HeaderMap, StatusCode};
use axum::routing::get;
use common::TestApp;
use serde_json::{Value, json};

/// A Home Assistant stand-in that counts how often it is asked.
async fn home_assistant(calls: Arc<AtomicUsize>) -> String {
    let stub = Router::new().route(
        "/api/config",
        get(move |headers: HeaderMap| async move {
            calls.fetch_add(1, Ordering::SeqCst);
            if headers.get("authorization").and_then(|v| v.to_str().ok()) != Some("Bearer ha-token") {
                return (StatusCode::UNAUTHORIZED, axum::Json(json!({})));
            }
            (StatusCode::OK, axum::Json(json!({"language": "pt-BR", "currency": "brl", "time_zone": "America/Sao_Paulo"})))
        }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, stub).await.unwrap() });
    base
}

async fn status(app: &TestApp) -> Value {
    let (status, body) = app.json("GET", "/api/status", None).await;
    assert_eq!(status, StatusCode::OK);
    body
}

#[tokio::test]
async fn home_assistant_sets_the_defaults_once() {
    let calls = Arc::new(AtomicUsize::new(0));
    let url = home_assistant(calls.clone()).await;
    let app = TestApp::with_env(&[
        ("MEMORY_CARD_HA_URL", &url),
        ("MEMORY_CARD_HA_TOKEN", "ha-token"),
    ])
    .await;

    let body = status(&app).await;
    assert_eq!(body["home_assistant_language"], "pt-BR");
    assert_eq!(body["default_currency"], "BRL");
    status(&app).await;
    assert_eq!(calls.load(Ordering::SeqCst), 1, "asked once");

    // A priced copy's currency comes before Home Assistant's.
    let release = app.release("Okami", "switch").await;
    sqlx::query("INSERT INTO copies (release_id, format, price, currency) VALUES (?, 'physical', 1000, 'JPY')")
        .bind(release)
        .execute(&app.pool)
        .await
        .unwrap();
    assert_eq!(status(&app).await["default_currency"], "JPY");
}

#[tokio::test]
async fn the_currency_option_comes_first() {
    let url = home_assistant(Arc::default()).await;
    let app = TestApp::with_env(&[
        ("MEMORY_CARD_HA_URL", &url),
        ("MEMORY_CARD_HA_TOKEN", "ha-token"),
        ("MEMORY_CARD_CURRENCY", "eur"),
    ])
    .await;
    let release = app.release("Okami", "switch").await;
    sqlx::query("INSERT INTO copies (release_id, format, price, currency) VALUES (?, 'physical', 1000, 'JPY')")
        .bind(release)
        .execute(&app.pool)
        .await
        .unwrap();
    assert_eq!(status(&app).await["default_currency"], "EUR");
}

#[tokio::test]
async fn outside_home_assistant_there_are_no_defaults() {
    let app = TestApp::new().await;
    let body = status(&app).await;
    assert_eq!(body["home_assistant_language"], Value::Null);
    assert_eq!(body["default_currency"], Value::Null);

    // A Home Assistant that refuses is asked again later, not on every request.
    let calls = Arc::new(AtomicUsize::new(0));
    let url = home_assistant(calls.clone()).await;
    let app = TestApp::with_env(&[
        ("MEMORY_CARD_HA_URL", &url),
        ("MEMORY_CARD_HA_TOKEN", "wrong"),
    ])
    .await;
    assert_eq!(status(&app).await["home_assistant_language"], Value::Null);
    status(&app).await;
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}
