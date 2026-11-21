use axum::Json;
use axum::extract::State;
use serde::Serialize;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use utoipa::ToSchema;

use crate::AppState;
use crate::access::Actor;
use crate::db::games::{self, GameDetail};
use crate::db::tags;
use crate::error::{ApiError, ApiPath, ApiResult, ErrorBody};
use crate::tags::{LINK_WAIT, PendingLink, TagsStatus};

#[derive(Debug, Serialize, ToSchema)]
pub struct TagLink {
    /// When the copy stops waiting for a scan.
    pub expires_at: String,
}

#[utoipa::path(
    post, path = "/copies/{id}/tag-link", tag = "tags", operation_id = "startTagLink",
    params(("id" = i64, Path)),
    responses(
        (status = OK, description = "The next tag scanned in Home Assistant links to this copy", body = TagLink),
        (status = NOT_FOUND, body = ErrorBody),
        (status = SERVICE_UNAVAILABLE, description = "`home_assistant_unavailable`", body = ErrorBody),
    )
)]
pub async fn start_link(
    State(state): State<AppState>,
    Actor(by): Actor,
    ApiPath(id): ApiPath<i64>,
) -> ApiResult<Json<TagLink>> {
    if *state.tag_status.lock().expect("tag status lock") != TagsStatus::Connected {
        return Err(ApiError::Unavailable("home_assistant_unavailable"));
    }
    games::copy_exists(&state.db, id).await?;
    let until = OffsetDateTime::now_utc() + LINK_WAIT;
    *state.tag_link.lock().expect("tag link lock") = Some(PendingLink {
        copy_id: id,
        by,
        until,
    });
    Ok(Json(TagLink {
        expires_at: until.format(&Rfc3339).expect("a UTC time formats"),
    }))
}

#[utoipa::path(
    delete, path = "/copies/{id}/tag-link", tag = "tags", operation_id = "cancelTagLink",
    params(("id" = i64, Path)),
    responses((status = NO_CONTENT, description = "The copy no longer waits for a scan"))
)]
pub async fn cancel_link(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<i64>,
) -> axum::http::StatusCode {
    let mut pending = state.tag_link.lock().expect("tag link lock");
    if pending.as_ref().is_some_and(|link| link.copy_id == id) {
        *pending = None;
    }
    axum::http::StatusCode::NO_CONTENT
}

#[utoipa::path(
    delete, path = "/tags/{id}", tag = "tags", operation_id = "unlinkTag",
    params(("id" = String, Path, description = "Home Assistant tag ID")),
    responses(
        (status = OK, description = "The tag's copy's game", body = GameDetail),
        (status = NOT_FOUND, body = ErrorBody),
    )
)]
pub async fn unlink(
    State(state): State<AppState>,
    ApiPath(tag_id): ApiPath<String>,
) -> ApiResult<Json<GameDetail>> {
    let game_id = tags::unlink(&state.db, &tag_id).await?;
    Ok(Json(games::get(&state.db, game_id).await?))
}
