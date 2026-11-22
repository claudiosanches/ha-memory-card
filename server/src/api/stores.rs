use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use serde::Deserialize;
use utoipa::ToSchema;

use super::input::{NAME_MAX, NOTES_MAX, optional, required};
use crate::AppState;
use crate::db::stores::{self, Store, StoreFields};
use crate::error::{ApiJson, ApiPath, ApiResult, ErrorBody};

#[derive(Debug, Deserialize, ToSchema)]
pub struct StoreInput {
    pub name: String,
    /// Names it in library links. Blank makes one from the name for a new
    /// one and keeps the current one on update.
    #[serde(default)]
    pub slug: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

impl StoreInput {
    fn validate(&self) -> ApiResult<StoreFields> {
        Ok(StoreFields {
            name: required(&self.name, "name", NAME_MAX)?,
            slug: crate::slug::input(self.slug.as_deref())?,
            notes: optional(self.notes.as_deref(), "notes", NOTES_MAX)?,
        })
    }
}

#[utoipa::path(
    get, path = "/stores", tag = "stores", operation_id = "listStores",
    responses((status = OK, body = Vec<Store>))
)]
pub async fn list(State(state): State<AppState>) -> ApiResult<Json<Vec<Store>>> {
    Ok(Json(stores::list(&state.db).await?))
}

#[utoipa::path(
    post, path = "/stores", tag = "stores", operation_id = "createStore",
    request_body = StoreInput,
    responses(
        (status = CREATED, body = Store),
        (status = CONFLICT, description = "`name_taken`, `slug_taken`", body = ErrorBody),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody),
    )
)]
pub async fn create(
    State(state): State<AppState>,
    ApiJson(input): ApiJson<StoreInput>,
) -> ApiResult<(StatusCode, Json<Store>)> {
    let id = stores::create(&state.db, &input.validate()?).await?;
    Ok((StatusCode::CREATED, Json(stores::get(&state.db, id).await?)))
}

#[utoipa::path(
    put, path = "/stores/{id}", tag = "stores", operation_id = "updateStore",
    params(("id" = i64, Path)),
    request_body = StoreInput,
    responses(
        (status = OK, body = Store),
        (status = NOT_FOUND, body = ErrorBody),
        (status = CONFLICT, description = "`name_taken`, `slug_taken`", body = ErrorBody),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody),
    )
)]
pub async fn update(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<i64>,
    ApiJson(input): ApiJson<StoreInput>,
) -> ApiResult<Json<Store>> {
    stores::update(&state.db, id, &input.validate()?).await?;
    Ok(Json(stores::get(&state.db, id).await?))
}

#[utoipa::path(
    delete, path = "/stores/{id}", tag = "stores", operation_id = "deleteStore",
    params(("id" = i64, Path)),
    responses(
        (status = NO_CONTENT, description = "Deleted; its copies keep existing with no store"),
        (status = NOT_FOUND, body = ErrorBody),
    )
)]
pub async fn delete(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<i64>,
) -> ApiResult<StatusCode> {
    stores::delete(&state.db, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
