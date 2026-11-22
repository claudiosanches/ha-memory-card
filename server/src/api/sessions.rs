use axum::extract::State;
use axum::http::StatusCode;
use axum::{Extension, Json};
use serde::Deserialize;
use utoipa::ToSchema;

use super::games::date;
use super::input::{NOTES_MAX, optional};
use crate::AppState;
use crate::access::{Access, Actor, Role};
use crate::db::games::{self, GameDetail};
use crate::db::sessions::{self, SessionFields};
use crate::error::{ApiError, ApiJson, ApiPath, ApiResult, ErrorBody};

/// A day holds at most this much play.
const MINUTES_MAX: i64 = 24 * 60;

#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct SessionInput {
    /// `YYYY-MM-DD`.
    pub played_on: String,
    /// An owned copy of the game.
    #[serde(default)]
    pub copy_id: Option<i64>,
    #[serde(default)]
    pub minutes: Option<i64>,
    #[serde(default)]
    pub notes: Option<String>,
}

impl SessionInput {
    fn validate(&self) -> ApiResult<SessionFields> {
        let played_on = date(Some(&self.played_on), "played_on")?
            .ok_or(ApiError::invalid("required", "played_on"))?;
        if self
            .minutes
            .is_some_and(|m| !(1..=MINUTES_MAX).contains(&m))
        {
            return Err(ApiError::invalid("out_of_range", "minutes"));
        }
        Ok(SessionFields {
            played_on,
            copy_id: self.copy_id,
            minutes: self.minutes,
            notes: optional(self.notes.as_deref(), "notes", NOTES_MAX)?,
        })
    }
}

#[utoipa::path(
    post, path = "/games/{id}/sessions", tag = "sessions", operation_id = "addSession",
    params(("id" = i64, Path)),
    request_body = SessionInput,
    responses(
        (status = CREATED, description = "The game with the new session", body = GameDetail),
        (status = NOT_FOUND, body = ErrorBody),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody),
    )
)]
pub async fn add(
    State(state): State<AppState>,
    Actor(by): Actor,
    ApiPath(id): ApiPath<i64>,
    ApiJson(input): ApiJson<SessionInput>,
) -> ApiResult<(StatusCode, Json<GameDetail>)> {
    sessions::add(&state.db, id, &input.validate()?, &by).await?;
    Ok((StatusCode::CREATED, Json(games::get(&state.db, id).await?)))
}

#[utoipa::path(
    put, path = "/sessions/{id}", tag = "sessions", operation_id = "updateSession",
    params(("id" = i64, Path)),
    request_body = SessionInput,
    responses(
        (status = OK, description = "The session's game", body = GameDetail),
        (status = FORBIDDEN, description = "`not_your_session`", body = ErrorBody),
        (status = NOT_FOUND, body = ErrorBody),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody),
    )
)]
pub async fn update(
    State(state): State<AppState>,
    Extension(access): Extension<Access>,
    ApiPath(id): ApiPath<i64>,
    ApiJson(input): ApiJson<SessionInput>,
) -> ApiResult<Json<GameDetail>> {
    let game_id = sessions::update(&state.db, id, &input.validate()?, owner(&access)).await?;
    Ok(Json(games::get(&state.db, game_id).await?))
}

#[utoipa::path(
    delete, path = "/sessions/{id}", tag = "sessions", operation_id = "deleteSession",
    params(("id" = i64, Path)),
    responses(
        (status = OK, description = "The session's game", body = GameDetail),
        (status = FORBIDDEN, description = "`not_your_session`", body = ErrorBody),
        (status = NOT_FOUND, body = ErrorBody),
    )
)]
pub async fn delete(
    State(state): State<AppState>,
    Extension(access): Extension<Access>,
    ApiPath(id): ApiPath<i64>,
) -> ApiResult<Json<GameDetail>> {
    let game_id = sessions::delete(&state.db, id, owner(&access)).await?;
    Ok(Json(games::get(&state.db, game_id).await?))
}

/// Viewers change only the sessions they logged; managers change any.
fn owner(access: &Access) -> Option<&str> {
    (access.role != Role::Manager).then_some(access.user.id.as_str())
}
