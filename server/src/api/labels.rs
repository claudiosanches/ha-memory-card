use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use serde::Deserialize;
use utoipa::ToSchema;

use super::input::{NAME_MAX, required};
use crate::AppState;
use crate::db::labels::{self, Label, LabelFields, SEPARATORS};
use crate::domain::StatusColor;
use crate::error::{ApiError, ApiJson, ApiPath, ApiResult, ErrorBody};

#[derive(Debug, Deserialize, ToSchema)]
pub struct LabelInput {
    pub name: String,
    /// Names it in library links. Blank makes one from the name for a new
    /// one and keeps the current one on update.
    #[serde(default)]
    pub slug: Option<String>,
    pub color: StatusColor,
}

impl LabelInput {
    fn validate(&self) -> ApiResult<LabelFields> {
        let name = required(&self.name, "name", NAME_MAX)?;
        if name.contains(SEPARATORS) {
            return Err(ApiError::invalid("label_separator", "name"));
        }
        Ok(LabelFields {
            name,
            slug: crate::slug::input(self.slug.as_deref())?,
            color: self.color,
        })
    }
}

#[utoipa::path(
    get, path = "/labels", tag = "labels", operation_id = "listLabels",
    responses((status = OK, body = Vec<Label>))
)]
pub async fn list(State(state): State<AppState>) -> ApiResult<Json<Vec<Label>>> {
    Ok(Json(labels::list(&state.db).await?))
}

#[utoipa::path(
    post, path = "/labels", tag = "labels", operation_id = "createLabel",
    request_body = LabelInput,
    responses(
        (status = CREATED, body = Label),
        (status = CONFLICT, description = "`name_taken`, `slug_taken`", body = ErrorBody),
        (status = UNPROCESSABLE_ENTITY, description = "Also `label_separator` for a comma or semicolon in the name", body = ErrorBody),
    )
)]
pub async fn create(
    State(state): State<AppState>,
    ApiJson(input): ApiJson<LabelInput>,
) -> ApiResult<(StatusCode, Json<Label>)> {
    let id = labels::create(&state.db, &input.validate()?).await?;
    Ok((StatusCode::CREATED, Json(labels::get(&state.db, id).await?)))
}

#[utoipa::path(
    put, path = "/labels/{id}", tag = "labels", operation_id = "updateLabel",
    params(("id" = i64, Path)),
    request_body = LabelInput,
    responses(
        (status = OK, body = Label),
        (status = NOT_FOUND, body = ErrorBody),
        (status = CONFLICT, description = "`name_taken`, `slug_taken`", body = ErrorBody),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody),
    )
)]
pub async fn update(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<i64>,
    ApiJson(input): ApiJson<LabelInput>,
) -> ApiResult<Json<Label>> {
    labels::update(&state.db, id, &input.validate()?).await?;
    Ok(Json(labels::get(&state.db, id).await?))
}

#[utoipa::path(
    delete, path = "/labels/{id}", tag = "labels", operation_id = "deleteLabel",
    params(("id" = i64, Path)),
    responses(
        (status = NO_CONTENT, description = "Deleted; its games keep existing without it"),
        (status = NOT_FOUND, body = ErrorBody),
    )
)]
pub async fn delete(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<i64>,
) -> ApiResult<StatusCode> {
    labels::delete(&state.db, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
