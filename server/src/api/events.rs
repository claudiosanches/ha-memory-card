use std::convert::Infallible;

use axum::extract::State;
use axum::response::sse::{Event, KeepAlive, Sse};
use futures_util::Stream;

use crate::AppState;

/// Pages open in a browser refetch on these, so a tag scan or another
/// user's change shows without a reload.
#[utoipa::path(
    get,
    path = "/events",
    tag = "status",
    responses(
        (status = OK, description = "Server-sent events: `changed`, with how many changes so far, after every change to the collection", content_type = "text/event-stream", body = String),
    )
)]
pub async fn stream(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let changes = state.changes.subscribe();
    let events = futures_util::stream::unfold(changes, |mut changes| async move {
        // Without marking anything seen: a change since subscribing still counts.
        if changes.borrow().1 {
            return None;
        }
        changes.changed().await.ok()?;
        let (count, closed) = *changes.borrow_and_update();
        if closed {
            return None;
        }
        // Browsers drop an event without data, so it carries the change count.
        let event = Event::default().event("changed").data(count.to_string());
        Some((Ok(event), changes))
    });
    Sse::new(events).keep_alive(KeepAlive::default())
}
