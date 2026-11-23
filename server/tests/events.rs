//! Change events for pages open in a browser.

mod common;

use std::time::Duration;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::{SUPERVISOR, TestApp};
use futures_util::StreamExt;
use memory_card::changes::{LISTENERS_MAX, LISTENERS_PER_PERSON};
use serde_json::json;
use tower::ServiceExt;

#[tokio::test]
async fn pages_hear_about_changes_until_the_server_stops() {
    let app = TestApp::new().await;
    let request = Request::get("/api/events")
        .header("x-remote-user-id", "viewer")
        .body(Body::empty())
        .unwrap();
    let response = app.router_from(SUPERVISOR).oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["content-type"], "text/event-stream");
    let mut events = response.into_body().into_data_stream();

    app.json("POST", "/api/games", Some(json!({"title": "Okami"})))
        .await;
    let event = tokio::time::timeout(Duration::from_secs(2), events.next())
        .await
        .expect("an event after the change")
        .unwrap()
        .unwrap();
    assert_eq!(
        String::from_utf8_lossy(&event),
        "event: changed\ndata: 1\n\n"
    );

    app.state.changes.close();
    let end = tokio::time::timeout(Duration::from_secs(2), events.next())
        .await
        .expect("the stream ends when the server stops");
    assert!(end.is_none());
}

async fn listen(app: &TestApp, user: &str) -> axum::response::Response {
    let request = Request::get("/api/events")
        .header("x-remote-user-id", user)
        .body(Body::empty())
        .unwrap();
    app.router_from(SUPERVISOR).oneshot(request).await.unwrap()
}

#[tokio::test]
async fn only_so_many_pages_listen_at_once() {
    let app = TestApp::new().await;
    let people = LISTENERS_MAX / LISTENERS_PER_PERSON;
    let mut open = Vec::new();
    for person in 0..people {
        for _ in 0..LISTENERS_PER_PERSON {
            let response = listen(&app, &format!("viewer-{person}")).await;
            assert_eq!(response.status(), StatusCode::OK);
            open.push(response);
        }
    }
    assert_eq!(
        listen(&app, "someone-else").await.status(),
        StatusCode::TOO_MANY_REQUESTS
    );
    // A closed page gives its place back.
    open.pop();
    assert_eq!(listen(&app, "someone-else").await.status(), StatusCode::OK);
}

#[tokio::test]
async fn one_person_listens_with_at_most_16_pages() {
    let app = TestApp::new().await;
    let mut open = Vec::new();
    for _ in 0..LISTENERS_PER_PERSON {
        let response = listen(&app, "viewer").await;
        assert_eq!(response.status(), StatusCode::OK);
        open.push(response);
    }
    assert_eq!(
        listen(&app, "viewer").await.status(),
        StatusCode::TOO_MANY_REQUESTS
    );
    assert_eq!(listen(&app, "manager").await.status(), StatusCode::OK);
    open.pop();
    assert_eq!(listen(&app, "viewer").await.status(), StatusCode::OK);
}
