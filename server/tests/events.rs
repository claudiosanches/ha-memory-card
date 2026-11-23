//! Change events for pages open in a browser.

mod common;

use std::time::Duration;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::{SUPERVISOR, TestApp};
use futures_util::StreamExt;
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
