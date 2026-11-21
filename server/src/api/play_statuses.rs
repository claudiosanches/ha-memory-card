use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

use super::input::{NAME_MAX, optional};
use crate::AppState;
use crate::db::play_statuses::{self, PlayStatus, StatusFields};
use crate::domain::{StatusColor, StatusGroup};
use crate::error::{ApiJson, ApiPath, ApiResult, ErrorBody};

#[derive(Debug, Deserialize, ToSchema)]
pub struct PlayStatusFields {
    /// Blank resets a built-in status to its default name.
    #[serde(default)]
    pub label: Option<String>,
    pub color: StatusColor,
    /// Fixed for built-in statuses.
    pub counts_as: StatusGroup,
}

impl PlayStatusFields {
    fn validate(&self) -> ApiResult<StatusFields> {
        Ok(StatusFields {
            label: optional(self.label.as_deref(), "label", NAME_MAX)?,
            color: self.color,
            counts_as: self.counts_as,
        })
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct PlayStatusOrder {
    /// Every status key, in the new order.
    pub keys: Vec<String>,
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct DeleteOptions {
    /// Status for the deleted status's games; absent leaves them with none.
    pub move_to: Option<String>,
}

#[utoipa::path(
    get, path = "/play-statuses", tag = "play statuses", operation_id = "listPlayStatuses",
    responses((status = OK, body = Vec<PlayStatus>))
)]
pub async fn list(State(state): State<AppState>) -> ApiResult<Json<Vec<PlayStatus>>> {
    Ok(Json(play_statuses::list(&state.db).await?))
}

#[utoipa::path(
    post, path = "/play-statuses", tag = "play statuses", operation_id = "createPlayStatus",
    request_body = PlayStatusFields,
    responses(
        (status = CREATED, body = PlayStatus),
        (status = CONFLICT, description = "`name_taken`", body = ErrorBody),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody),
    )
)]
pub async fn create(
    State(state): State<AppState>,
    ApiJson(input): ApiJson<PlayStatusFields>,
) -> ApiResult<(StatusCode, Json<PlayStatus>)> {
    let key = play_statuses::create(&state.db, &input.validate()?).await?;
    Ok((
        StatusCode::CREATED,
        Json(play_statuses::get(&state.db, &key).await?),
    ))
}

#[utoipa::path(
    put, path = "/play-statuses/{key}", tag = "play statuses", operation_id = "updatePlayStatus",
    params(("key" = String, Path)),
    request_body = PlayStatusFields,
    responses(
        (status = OK, body = PlayStatus),
        (status = NOT_FOUND, body = ErrorBody),
        (status = CONFLICT, description = "`name_taken`", body = ErrorBody),
        (status = UNPROCESSABLE_ENTITY, description = "`builtin_status` when changing a built-in's group", body = ErrorBody),
    )
)]
pub async fn update(
    State(state): State<AppState>,
    ApiPath(key): ApiPath<String>,
    ApiJson(input): ApiJson<PlayStatusFields>,
) -> ApiResult<Json<PlayStatus>> {
    play_statuses::update(&state.db, &key, &input.validate()?).await?;
    Ok(Json(play_statuses::get(&state.db, &key).await?))
}

#[utoipa::path(
    delete, path = "/play-statuses/{key}", tag = "play statuses", operation_id = "deletePlayStatus",
    params(("key" = String, Path), DeleteOptions),
    responses(
        (status = NO_CONTENT, description = "Deleted; its games moved to `move_to` or to no status"),
        (status = NOT_FOUND, body = ErrorBody),
        (status = CONFLICT, description = "`builtin_status`", body = ErrorBody),
        (status = UNPROCESSABLE_ENTITY, description = "`unknown_play_status`", body = ErrorBody),
    )
)]
pub async fn delete(
    State(state): State<AppState>,
    ApiPath(key): ApiPath<String>,
    Query(options): Query<DeleteOptions>,
) -> ApiResult<StatusCode> {
    play_statuses::delete(&state.db, &key, options.move_to.as_deref()).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    put, path = "/play-statuses/order", tag = "play statuses", operation_id = "reorderPlayStatuses",
    request_body = PlayStatusOrder,
    responses(
        (status = OK, body = Vec<PlayStatus>),
        (status = UNPROCESSABLE_ENTITY, description = "`invalid_order`", body = ErrorBody),
    )
)]
pub async fn reorder(
    State(state): State<AppState>,
    ApiJson(order): ApiJson<PlayStatusOrder>,
) -> ApiResult<Json<Vec<PlayStatus>>> {
    play_statuses::reorder(&state.db, &order.keys).await?;
    Ok(Json(play_statuses::list(&state.db).await?))
}
