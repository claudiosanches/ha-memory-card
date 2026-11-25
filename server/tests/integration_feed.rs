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

    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
    assert_eq!(integration::token(&path).unwrap().expose(), first.expose());
    let mode = std::fs::metadata(&path).unwrap().permissions().mode();
    assert_eq!(
        mode & 0o777,
        0o600,
        "kept, and readable only by the app again"
    );

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
        assert_eq!(response.headers()["connection"], "close");
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
    assert_eq!(
        entities.len(),
        7,
        "the household's, while nobody keeps play"
    );
    assert_eq!(
        entities[0],
        json!({"key": "games", "platform": "sensor", "name": "Games", "icon": "mdi:gamepad-variant",
               "unit": "games", "state_class": "measurement", "attributes": false,
               "object_id": "games"})
    );
    assert_eq!(entities[6]["platform"], "button");

    let state = next_line(&mut body, &mut buffer).await;
    assert_eq!(state["type"], "state");
    assert_eq!(state["state"]["games"], 0);

    feed.refresh().await;
    let (imported, result) = app.post_csv("/api/import", ONE_GAME).await;
    assert_eq!(imported, StatusCode::OK, "{result}");
    feed.refresh().await;
    // The importer keeps play now, so their sensors come first.
    let hello = next_line(&mut body, &mut buffer).await;
    assert_eq!(
        hello["type"], "hello",
        "an unchanged state is not sent again"
    );
    let entities = hello["entities"].as_array().unwrap();
    assert_eq!(entities.len(), 10);
    assert_eq!(entities[7]["person"]["kind"], "backlog");
    let state = next_line(&mut body, &mut buffer).await;
    assert_eq!(state["type"], "state");
    assert_eq!(state["state"]["games"], 1);
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
async fn a_feed_opened_before_the_first_read_waits_for_it() {
    let app = TestApp::with_env(&[("MEMORY_CARD_SENSORS", "integration")]).await;
    let feed = feed(&app);
    let response = integration::router(feed.clone())
        .oneshot(get_feed(Some(TOKEN)))
        .await
        .unwrap();
    let mut body = response.into_body();
    assert!(
        tokio::time::timeout(Duration::from_millis(200), body.frame())
            .await
            .is_err(),
        "no hello before the people are read"
    );

    feed.refresh().await;
    let mut buffer = Vec::new();
    assert_eq!(next_line(&mut body, &mut buffer).await["type"], "hello");
    assert_eq!(next_line(&mut body, &mut buffer).await["type"], "state");
    feed.close();
}

#[tokio::test]
async fn a_feed_read_late_still_gets_new_entities() {
    let app = TestApp::with_env(&[("MEMORY_CARD_SENSORS", "integration")]).await;
    let feed = feed(&app);
    feed.refresh().await;
    let response = integration::router(feed.clone())
        .oneshot(get_feed(Some(TOKEN)))
        .await
        .unwrap();
    let mut body = response.into_body();
    let mut buffer = Vec::new();
    assert_eq!(next_line(&mut body, &mut buffer).await["type"], "hello");
    assert_eq!(next_line(&mut body, &mut buffer).await["type"], "state");

    // New entities, then only new values, before Home Assistant reads either.
    app.post_csv("/api/import", ONE_GAME).await;
    feed.refresh().await;
    app.post_csv(
        "/api/import",
        "game_title,platform,region,format\nTetris,Switch,ww,physical\n",
    )
    .await;
    feed.refresh().await;

    let hello = next_line(&mut body, &mut buffer).await;
    assert_eq!(hello["type"], "hello", "{hello}");
    assert_eq!(hello["entities"].as_array().unwrap().len(), 10);
    let state = next_line(&mut body, &mut buffer).await;
    assert_eq!(state["state"]["games"], 2);
    feed.close();
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
    roll_as(router, Some(common::MANAGER), body).await
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
async fn a_roll_picks_from_the_callers_backlog_with_the_same_filters_as_mqtt() {
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
    let (status, pick) = roll(&router, "").await;
    assert_eq!(status, StatusCode::OK);
    assert!(pick["title"].is_string());
    // Each person picks from their own backlog.
    let (status, _) = roll_as(&router, None, r#"{"person": "someone_else"}"#).await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    let (status, error) = roll(&router, r#"{"colour": 1}"#).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(error["code"], "invalid_filters");
    let (status, _) = roll(&router, &" ".repeat(5000)).await;
    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
}

#[tokio::test]
async fn a_roll_is_for_one_person_who_keeps_play() {
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
    let manager = format!(r#"{{"person": "{}"}}"#, common::MANAGER);
    assert_eq!(roll(&router, "").await.0, StatusCode::OK);
    assert_eq!(roll(&router, &manager).await.0, StatusCode::OK);
    assert_eq!(
        roll_as(&router, None, &manager).await.0,
        StatusCode::OK,
        "an automation names the person"
    );

    for (user, body, status, code) in [
        (
            None,
            "",
            StatusCode::UNPROCESSABLE_ENTITY,
            "person_required",
        ),
        (
            None,
            r#"{"person": "bad id!"}"#,
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_user",
        ),
        (
            Some(common::MANAGER),
            r#"{"person": "someone_else"}"#,
            StatusCode::FORBIDDEN,
            "not_your_roll",
        ),
        (
            Some("someone_else"),
            "",
            StatusCode::FORBIDDEN,
            "play_not_allowed",
        ),
        (
            Some("bad id!"),
            "",
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_user",
        ),
    ] {
        let (got, error) = roll_as(&router, user, body).await;
        assert_eq!(
            (got, error["code"].as_str()),
            (status, Some(code)),
            "{user:?} {body}"
        );
    }

    // With viewers keeping play, an automation names only people the app knows.
    let viewers = TestApp::with_env(&[("MEMORY_CARD_VIEWERS_TRACK_PROGRESS", "true")]).await;
    let (status, result) = viewers
        .post_csv(
            "/api/import",
            "game_title,play_status,platform,region,format,status\n\
             Tetris,backlog,Switch,eu,physical,owned\n",
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{result}");
    let router = integration::router(feed(&viewers));
    let made_up = r#"{"person": "made-up", "play_status": ["none"]}"#;
    let (status, error) = roll_as(&router, None, made_up).await;
    assert_eq!(
        (status, error["code"].as_str()),
        (StatusCode::FORBIDDEN, Some("play_not_allowed"))
    );
    assert_eq!(
        viewers
            .scalar("SELECT COUNT(*) FROM users WHERE id = 'made-up'")
            .await,
        0
    );
    // Once they keep play in the app, it knows them.
    let tetris = viewers.scalar("SELECT id FROM games").await;
    let play = Request::put(format!("/api/games/{tetris}/play-status"))
        .header("x-remote-user-id", "made-up")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"play_status": "backlog"}"#))
        .unwrap();
    assert_eq!(
        viewers.request(common::SUPERVISOR, play).await.0,
        StatusCode::OK
    );
    assert_eq!(
        roll_as(&router, None, r#"{"person": "made-up"}"#).await.1["title"],
        "Tetris"
    );

    let nobody = TestApp::with_env(&[("MEMORY_CARD_MANAGERS", "")]).await;
    let router = integration::router(feed(&nobody));
    assert_eq!(
        roll_as(&router, None, &manager).await.0,
        StatusCode::FORBIDDEN
    );
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
            ("MEMORY_CARD_INTEGRATION_PEER", "127.0.0.1"),
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
        while text.matches('\n').count() < 4 {
            text.push_str(&String::from_utf8_lossy(
                &response.chunk().await.unwrap().unwrap(),
            ));
        }
    })
    .await
    .expect("a new state after the write");
    let hello: Value = serde_json::from_str(text.lines().nth(2).unwrap()).unwrap();
    assert_eq!(hello["type"], "hello");
    let last: Value = serde_json::from_str(text.lines().nth(3).unwrap()).unwrap();
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

#[tokio::test]
async fn the_feed_closes_connections_from_anyone_but_home_assistant() {
    let calls = Arc::new(Mutex::new(Calls::default()));
    let base = supervisor(calls.clone()).await;
    let app = TestApp::with_env(&[
        ("MEMORY_CARD_SENSORS", "integration"),
        ("SUPERVISOR_TOKEN", "token-1"),
        ("MEMORY_CARD_SUPERVISOR_URL", base.as_str()),
        ("MEMORY_CARD_INTEGRATION_LISTEN", "127.0.0.1:0"),
    ])
    .await;
    let sensors = Sensors::start(app.state.clone());
    until("the announcement", || {
        !calls.lock().unwrap().announced.is_empty()
    })
    .await;
    let port = calls.lock().unwrap().announced[0]["config"]["port"]
        .as_u64()
        .unwrap();
    let token = std::fs::read_to_string(app.config.integration_token_path()).unwrap();
    let answer = app
        .state
        .http
        .get(format!("http://127.0.0.1:{port}/integration/v1/feed"))
        .bearer_auth(&token)
        .send()
        .await;
    assert!(answer.is_err(), "closed unanswered: {answer:?}");
    sensors.stop().await;
}
