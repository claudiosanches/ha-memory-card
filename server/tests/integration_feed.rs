//! The feed the Memory Card integration reads, and how the app announces it.

mod common;

use std::os::unix::fs::PermissionsExt;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::Router;
use axum::body::Body;
use axum::extract::Path;
use axum::http::{HeaderMap, Request, StatusCode};
use axum::routing::{delete, get, post};
use common::TestApp;
use http_body_util::BodyExt;
use memory_card::config::Secret;
use memory_card::sensors::Sensors;
use memory_card::sensors::integration::{self, FEEDS_MAX, Feed};
use serde_json::{Value, json};
use tower::ServiceExt;

const TOKEN: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const UUID: &str = "fedcba9876543210fedcba9876543210";
const ONE_GAME: &str = "game_title,play_status,platform,region,format,status\n\
    Celeste,playing,Switch,ww,digital,owned\n";

fn feed(app: &TestApp) -> Arc<Feed> {
    Feed::new(app.state.clone(), Secret::new(TOKEN))
}

fn get_feed(token: Option<&str>) -> Request<Body> {
    let mut request = Request::get("/integration/v1/feed");
    if let Some(token) = token {
        request = request.header("authorization", format!("Bearer {token}"));
    }
    request.body(Body::empty()).unwrap()
}

async fn status(app: &TestApp) -> Value {
    app.json("GET", "/api/status", None).await.1["sensors"].clone()
}

/// Reads one line of the feed as JSON.
async fn next_line(body: &mut Body, buffer: &mut Vec<u8>) -> Value {
    loop {
        if let Some(end) = buffer.iter().position(|&b| b == b'\n') {
            let line: Vec<u8> = buffer.drain(..=end).collect();
            return serde_json::from_slice(&line).unwrap();
        }
        let frame = tokio::time::timeout(Duration::from_secs(5), body.frame())
            .await
            .expect("a line in time")
            .expect("the feed is still open")
            .unwrap();
        if let Ok(data) = frame.into_data() {
            buffer.extend_from_slice(&data);
        }
    }
}

#[tokio::test]
async fn the_token_is_random_kept_and_readable_only_by_the_app() {
    let app = TestApp::new().await;
    let path = app.config.integration_token_path();
    let first = integration::token(&path).unwrap();
    assert_eq!(first.expose().len(), 64);
    assert!(first.expose().bytes().all(|b| b.is_ascii_hexdigit()));
    let mode = std::fs::metadata(&path).unwrap().permissions().mode();
    assert_eq!(mode & 0o777, 0o600);
    assert_eq!(integration::token(&path).unwrap().expose(), first.expose());

    std::fs::write(&path, "short").unwrap();
    let replaced = integration::token(&path).unwrap();
    assert_ne!(replaced.expose(), "short");
    assert_ne!(replaced.expose(), first.expose());
}

#[tokio::test]
async fn the_feed_needs_the_token() {
    let app = TestApp::new().await;
    let router = integration::router(feed(&app));
    for token in [None, Some("wrong"), Some(&TOKEN[..63])] {
        let response = router.clone().oneshot(get_feed(token)).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED, "{token:?}");
    }
    let roll = Request::post("/integration/v1/roll")
        .body(Body::empty())
        .unwrap();
    let response = router.oneshot(roll).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn the_feed_says_hello_then_sends_the_state_and_its_changes() {
    let app = TestApp::with_env(&[("MEMORY_CARD_SENSORS", "integration")]).await;
    assert_eq!(status(&app).await, "waiting");
    let feed = feed(&app);
    feed.refresh().await;

    let response = integration::router(feed.clone())
        .oneshot(get_feed(Some(TOKEN)))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["content-type"], "application/x-ndjson");
    assert_eq!(status(&app).await, "connected");
    let mut body = response.into_body();
    let mut buffer = Vec::new();

    let hello = next_line(&mut body, &mut buffer).await;
    assert_eq!(hello["type"], "hello");
    assert_eq!(hello["api"], 1);
    assert_eq!(hello["device"]["name"], "Memory Card");
    let entities = hello["entities"].as_array().unwrap();
    assert_eq!(entities.len(), 9);
    assert_eq!(
        entities[0],
        json!({"key": "games", "platform": "sensor", "name": "Games", "icon": "mdi:gamepad-variant",
               "unit": "games", "state_class": "measurement", "attributes": false})
    );
    assert_eq!(entities[8]["platform"], "button");

    let state = next_line(&mut body, &mut buffer).await;
    assert_eq!(state["type"], "state");
    assert_eq!(state["state"]["games"], 0);

    feed.refresh().await;
    let (imported, result) = app.post_csv("/api/import", ONE_GAME).await;
    assert_eq!(imported, StatusCode::OK, "{result}");
    feed.refresh().await;
    let state = next_line(&mut body, &mut buffer).await;
    assert_eq!(state["type"], "state");
    assert_eq!(
        state["state"]["games"], 1,
        "an unchanged state is not sent again"
    );
    assert_eq!(
        state["state"]["attributes"]["playing"]["games"],
        json!(["Celeste"])
    );

    feed.close();
    assert!(
        body.frame().await.is_none(),
        "the feed ends when the app stops"
    );
    drop(body);
    assert_eq!(status(&app).await, "waiting");
}

#[tokio::test]
async fn only_a_few_feeds_open_at_once() {
    let app = TestApp::new().await;
    let router = integration::router(feed(&app));
    let mut open = Vec::new();
    for _ in 0..FEEDS_MAX {
        let response = router.clone().oneshot(get_feed(Some(TOKEN))).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        open.push(response);
    }
    let response = router.clone().oneshot(get_feed(Some(TOKEN))).await.unwrap();
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    open.pop();
    let response = router.oneshot(get_feed(Some(TOKEN))).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

async fn roll(router: &Router, body: &str) -> (StatusCode, Value) {
    roll_as(router, None, body).await
}

async fn roll_as(router: &Router, user: Option<&str>, body: &str) -> (StatusCode, Value) {
    let mut request = Request::post("/integration/v1/roll")
        .header("authorization", format!("Bearer {TOKEN}"))
        .header("content-type", "application/json");
    if let Some(user) = user {
        request = request.header(integration::USER_HEADER, user);
    }
    let request = request.body(Body::from(body.to_owned())).unwrap();
    let response = router.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    (status, serde_json::from_slice(&body).unwrap_or(Value::Null))
}

#[tokio::test]
async fn a_roll_picks_from_the_backlog_with_the_same_filters_as_mqtt() {
    let app = TestApp::new().await;
    let router = integration::router(feed(&app));
    assert_eq!(roll(&router, "").await.0, StatusCode::NOT_FOUND);

    let (status, result) = app
        .post_csv(
            "/api/import",
            "game_title,play_status,platform,region,format,status\n\
             Tetris,backlog,Switch,eu,physical,owned\n\
             Hades,backlog,PS4,na,digital,owned\n",
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{result}");

    let (status, pick) = roll(&router, r#"{"platform": ["switch"]}"#).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        pick,
        json!({"title": "Tetris", "platforms": ["Nintendo Switch"]})
    );
    let (status, pick) = roll(&router, "roll").await;
    assert_eq!(status, StatusCode::OK);
    assert!(pick["title"].is_string());

    let (status, error) = roll(&router, r#"{"colour": 1}"#).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(error["code"], "invalid_filters");
    let (status, _) = roll(&router, &" ".repeat(5000)).await;
    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
}

#[tokio::test]
async fn only_managers_and_automations_roll_and_only_once_managers_are_set() {
    let app = TestApp::new().await;
    let (status, result) = app
        .post_csv(
            "/api/import",
            "game_title,play_status,platform,region,format,status\n\
             Tetris,backlog,Switch,eu,physical,owned\n",
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{result}");
    let router = integration::router(feed(&app));
    assert_eq!(
        roll_as(&router, Some(common::MANAGER), "").await.0,
        StatusCode::OK
    );
    assert_eq!(
        roll_as(&router, None, "").await.0,
        StatusCode::OK,
        "an automation"
    );
    let (status, error) = roll_as(&router, Some("someone_else"), "").await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(error["code"], "manager_required");
    let (status, error) = roll_as(&router, Some("bad id!"), "").await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(error["code"], "invalid_user");

    let nobody = TestApp::with_env(&[("MEMORY_CARD_MANAGERS", "")]).await;
    let router = integration::router(feed(&nobody));
    assert_eq!(roll_as(&router, None, "").await.0, StatusCode::FORBIDDEN);
}

#[derive(Default)]
struct Calls {
    announced: Vec<Value>,
    withdrawn: Vec<String>,
}

/// A Supervisor stand-in that records discovery messages.
async fn supervisor(calls: Arc<Mutex<Calls>>) -> String {
    let authorized = |headers: &HeaderMap| {
        headers.get("authorization").and_then(|v| v.to_str().ok()) == Some("Bearer token-1")
    };
    let announced = calls.clone();
    let withdrawn = calls;
    let stub = Router::new()
        .route(
            "/addons/self/info",
            get(|| async {
                axum::Json(
                    json!({"data": {"slug": "local_memory_card", "hostname": "local-memory-card"}}),
                )
            }),
        )
        .route(
            "/services/mqtt",
            get(|| async {
                (
                    StatusCode::BAD_REQUEST,
                    axum::Json(json!({"result": "error"})),
                )
            }),
        )
        .route(
            "/discovery",
            post(
                move |headers: HeaderMap, axum::Json(body): axum::Json<Value>| async move {
                    assert!(authorized(&headers));
                    announced.lock().unwrap().announced.push(body);
                    axum::Json(json!({"result": "ok", "data": {"uuid": UUID}}))
                },
            ),
        )
        .route(
            "/discovery/{uuid}",
            delete(
                move |headers: HeaderMap, Path(uuid): Path<String>| async move {
                    assert!(authorized(&headers));
                    withdrawn.lock().unwrap().withdrawn.push(uuid);
                    axum::Json(json!({"result": "ok", "data": {}}))
                },
            ),
        );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, stub).await.unwrap() });
    base
}

async fn until(what: &str, mut done: impl FnMut() -> bool) {
    for _ in 0..100 {
        if done() {
            return;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    panic!("{what} did not happen");
}

#[tokio::test]
async fn the_app_announces_the_feed_and_takes_it_back_when_the_mode_changes() {
    let calls = Arc::new(Mutex::new(Calls::default()));
    let base = supervisor(calls.clone()).await;
    let env = |mode: &'static str| {
        [
            ("MEMORY_CARD_SENSORS", mode),
            ("SUPERVISOR_TOKEN", "token-1"),
            ("MEMORY_CARD_SUPERVISOR_URL", base.as_str()),
            ("MEMORY_CARD_INTEGRATION_LISTEN", "127.0.0.1:0"),
        ]
    };
    let app = TestApp::with_env(&env("integration")).await;
    let sensors = Sensors::start(app.state.clone());
    until("the announcement", || {
        !calls.lock().unwrap().announced.is_empty()
    })
    .await;
    let announced = calls.lock().unwrap().announced[0].clone();
    assert_eq!(announced["service"], "memory_card");
    assert_eq!(announced["config"]["host"], "local-memory-card");
    let token = std::fs::read_to_string(app.config.integration_token_path()).unwrap();
    assert_eq!(announced["config"]["token"], token.as_str());
    let saved = app.config.integration_discovery_path();
    until("the saved id", || saved.exists()).await;
    assert_eq!(std::fs::read_to_string(&saved).unwrap(), UUID);

    // The announced port serves the feed, which picks up a change by itself.
    let port = announced["config"]["port"].as_u64().unwrap();
    let mut response = app
        .state
        .http
        .get(format!("http://127.0.0.1:{port}/integration/v1/feed"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let mut text = String::new();
    while text.matches('\n').count() < 2 {
        text.push_str(&String::from_utf8_lossy(
            &response.chunk().await.unwrap().unwrap(),
        ));
    }
    let (imported, result) = app.post_csv("/api/import", ONE_GAME).await;
    assert_eq!(imported, StatusCode::OK, "{result}");
    tokio::time::timeout(Duration::from_secs(5), async {
        while text.matches('\n').count() < 3 {
            text.push_str(&String::from_utf8_lossy(
                &response.chunk().await.unwrap().unwrap(),
            ));
        }
    })
    .await
    .expect("a new state after the write");
    let last: Value = serde_json::from_str(text.lines().nth(2).unwrap()).unwrap();
    assert_eq!(last["state"]["games"], 1);
    sensors.stop().await;

    // Switching to off takes the announcement back.
    let off = TestApp::with_env(&env("off")).await;
    std::fs::write(off.config.integration_discovery_path(), UUID).unwrap();
    let sensors = Sensors::start(off.state.clone());
    until("the withdrawal", || {
        !calls.lock().unwrap().withdrawn.is_empty()
    })
    .await;
    assert_eq!(calls.lock().unwrap().withdrawn, [UUID]);
    let saved = off.config.integration_discovery_path();
    until("the id removed", || !saved.exists()).await;
    sensors.stop().await;
}
