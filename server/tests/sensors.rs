//! What the Home Assistant sensors report, when they refresh and how the broker is found.

mod common;

use std::time::Duration;

use axum::Router;
use axum::http::{HeaderMap, StatusCode};
use axum::routing::get;
use common::TestApp;
use memory_card::db::sensors;
use memory_card::sensors::broker;
use serde_json::{Value, json};

const COLLECTION: &str = "game_title,play_status,platform,region,format,status\n\
    Tetris,backlog,Switch,eu,physical,owned\n\
    Tetris,backlog,PS4,na,digital,owned\n\
    Hades,playing,PS4,na,digital,owned\n\
    Celeste,playing,Switch,ww,digital,owned\n\
    Silksong,backlog,Switch,ww,physical,wanted\n\
    Zelda,completed,Switch,eu,physical,owned\n";

async fn game_id(app: &TestApp, title: &str) -> i64 {
    sqlx::query_scalar("SELECT id FROM games WHERE title = ?")
        .bind(title)
        .fetch_one(&app.pool)
        .await
        .unwrap()
}

#[tokio::test]
async fn sensors_count_owned_copies_and_name_the_latest_games() {
    let app = TestApp::new().await;
    let empty = sensors::snapshot(&app.pool).await.unwrap();
    assert_eq!((empty.games, empty.copies, empty.playing_count), (0, 0, 0));
    assert!(empty.last_played.is_none() && empty.last_added.is_none());

    let (status, result) = app.post_csv("/api/import", COLLECTION).await;
    assert_eq!(status, StatusCode::OK, "{result}");

    let celeste = game_id(&app, "Celeste").await;
    let (_, detail) = app
        .json("GET", &format!("/api/games/{celeste}"), None)
        .await;
    let copy = detail["releases"][0]["copies"][0]["id"].as_i64().unwrap();
    let (status, _) = app
        .json(
            "POST",
            &format!("/api/games/{celeste}/sessions"),
            Some(json!({"played_on": "2026-11-20", "copy_id": copy})),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);

    let hades = game_id(&app, "Hades").await;
    let switch = app.platform_id("switch").await;
    let (status, body) = app
        .json(
            "POST",
            &format!("/api/games/{hades}/copies"),
            Some(json!({"platform_id": switch, "region": "eu", "format": "digital"})),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "{body}");

    let snapshot = sensors::snapshot(&app.pool).await.unwrap();
    assert_eq!(snapshot.games, 4, "the wishlist-only game is left out");
    assert_eq!(snapshot.copies, 6);
    assert_eq!(snapshot.wishlist, 1);
    assert_eq!(snapshot.backlog, 1, "Tetris; Silksong is only wanted");
    assert_eq!(snapshot.playing_count, 2);
    assert_eq!(snapshot.playing, vec!["Celeste", "Hades"]);

    let last_played = snapshot.last_played.unwrap();
    assert_eq!(last_played.title, "Celeste");
    assert_eq!(last_played.date, "2026-11-20");
    assert_eq!(last_played.platform.as_deref(), Some("Nintendo Switch"));

    let last_added = snapshot.last_added.unwrap();
    assert_eq!(last_added.title, "Hades");
    assert_eq!(last_added.platform, "Nintendo Switch");
    assert!(snapshot.pick.is_none());

    let (status, _) = app
        .json(
            "POST",
            "/api/suggestions",
            Some(json!({"play_status": ["backlog"]})),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let pick = sensors::snapshot(&app.pool).await.unwrap().pick.unwrap();
    assert_eq!(pick.title, "Tetris", "the only owned backlog game");
    assert_eq!(pick.platforms, ["PlayStation 4", "Nintendo Switch"]);
}

#[tokio::test]
async fn successful_writes_wake_the_sensors() {
    let app = TestApp::new().await;
    let woken = || {
        tokio::time::timeout(
            Duration::from_millis(50),
            app.state.changes.next_for_sensors(),
        )
    };

    let (status, _) = app
        .json("POST", "/api/games", Some(json!({"title": "Celeste"})))
        .await;
    assert_eq!(status, StatusCode::CREATED);
    assert!(woken().await.is_ok(), "a write wakes the sensors");

    app.json("GET", "/api/games", None).await;
    let (status, _) = app.json("DELETE", "/api/games/999", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(woken().await.is_err(), "reads and failed writes do not");
}

#[tokio::test]
async fn status_says_why_sensors_are_not_published() {
    let app = TestApp::new().await;
    let (_, status) = app.json("GET", "/api/status", None).await;
    assert_eq!(status["sensors"], "off", "off by default");

    let app = TestApp::with_env(&[("MEMORY_CARD_SENSORS", "mqtt")]).await;
    let (_, status) = app.json("GET", "/api/status", None).await;
    assert_eq!(status["sensors"], "no_broker");

    let app = TestApp::with_env(&[
        ("MEMORY_CARD_SENSORS", "mqtt"),
        ("SUPERVISOR_TOKEN", "token-1"),
    ])
    .await;
    let (_, status) = app.json("GET", "/api/status", None).await;
    assert_eq!(status["sensors"], "connecting");

    let app = TestApp::with_env(&[("MEMORY_CARD_SENSORS", "integration")]).await;
    let (_, status) = app.json("GET", "/api/status", None).await;
    assert_eq!(status["sensors"], "waiting");
}

/// A Supervisor stand-in answering `/services/mqtt` with `mqtt`.
async fn supervisor(mqtt: (StatusCode, Value)) -> String {
    let authorized = |headers: &HeaderMap| {
        headers.get("authorization").and_then(|v| v.to_str().ok()) == Some("Bearer token-1")
    };
    let stub = Router::new()
        .route(
            "/services/mqtt",
            get(move |headers: HeaderMap| async move {
                if authorized(&headers) {
                    (mqtt.0, axum::Json(mqtt.1))
                } else {
                    (StatusCode::UNAUTHORIZED, axum::Json(json!({})))
                }
            }),
        )
        .route(
            "/addons/self/info",
            get(|| async { axum::Json(json!({"data": {"slug": "abc123_memory_card", "hostname": "abc123-memory-card"}})) }),
        );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, stub).await.unwrap() });
    base
}

async fn find_with(mqtt: (StatusCode, Value)) -> Option<broker::Broker> {
    let base = supervisor(mqtt).await;
    let app = TestApp::with_env(&[
        ("SUPERVISOR_TOKEN", "token-1"),
        ("MEMORY_CARD_SUPERVISOR_URL", &base),
    ])
    .await;
    broker::find(&app.config, &app.state.http).await.unwrap()
}

#[tokio::test]
async fn the_supervisor_names_the_broker() {
    let service = json!({"data": {
        "addon": "core_mosquitto", "host": "core-mosquitto", "port": 1883, "ssl": false,
        "protocol": "3.1.1", "username": "addons", "password": "pass-1",
    }});
    let found = find_with((StatusCode::OK, service.clone())).await.unwrap();
    assert_eq!(found.mqtt.host, "core-mosquitto");
    assert_eq!(found.mqtt.port, 1883);
    assert_eq!(found.mqtt.username.as_deref(), Some("addons"));
    assert_eq!(found.mqtt.password.unwrap().expose(), "pass-1");
    assert_eq!(found.ingress_slug.as_deref(), Some("abc123_memory_card"));

    let missing = json!({"result": "error", "message": "No service available"});
    assert!(
        find_with((StatusCode::BAD_REQUEST, missing))
            .await
            .is_none()
    );

    let mut tls = service;
    tls["data"]["ssl"] = json!(true);
    assert!(find_with((StatusCode::OK, tls)).await.is_none());
}

#[tokio::test]
async fn outside_home_assistant_there_is_no_broker_to_ask() {
    let app = TestApp::new().await;
    assert!(
        broker::find(&app.config, &app.state.http)
            .await
            .unwrap()
            .is_none()
    );

    let app = TestApp::with_env(&[("MEMORY_CARD_MQTT_HOST", "localhost")]).await;
    let found = broker::find(&app.config, &app.state.http)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        (found.mqtt.host.as_str(), found.mqtt.port),
        ("localhost", 1883)
    );
    assert!(found.ingress_slug.is_none());
}

#[tokio::test]
async fn supervisor_redirects_are_not_followed() {
    let stub = Router::new().route(
        "/services/mqtt",
        get(|| async { axum::response::Redirect::temporary("http://elsewhere.invalid/steal") }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, stub).await.unwrap() });
    let app = TestApp::with_env(&[
        ("SUPERVISOR_TOKEN", "token-1"),
        ("MEMORY_CARD_SUPERVISOR_URL", &base),
    ])
    .await;
    let err = broker::find(&app.config, &app.state.http)
        .await
        .unwrap_err();
    assert!(err.to_string().contains("307"), "{err}");
}
