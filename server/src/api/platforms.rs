use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use serde::Deserialize;
use utoipa::ToSchema;

use super::input::{NAME_MAX, SHORT_MAX, optional, required};
use crate::AppState;
use crate::db::platforms::{self, Platform, PlatformCategory, PlatformFields};
use crate::error::{ApiJson, ApiPath, ApiResult, ErrorBody};

#[derive(Debug, Deserialize, ToSchema)]
pub struct PlatformInput {
    pub name: String,
    /// Names it in library links. Blank makes one from the name for a new
    /// one and keeps the current one on update.
    #[serde(default)]
    pub slug: Option<String>,
    #[serde(default)]
    pub abbreviation: Option<String>,
    #[serde(default)]
    pub category: Option<PlatformCategory>,
    #[serde(default = "default_active")]
    pub active: bool,
}

fn default_active() -> bool {
    true
}

impl PlatformInput {
    fn validate(&self) -> ApiResult<PlatformFields> {
        Ok(PlatformFields {
            name: required(&self.name, "name", NAME_MAX)?,
            slug: crate::slug::input(self.slug.as_deref())?,
            abbreviation: optional(self.abbreviation.as_deref(), "abbreviation", SHORT_MAX)?,
            category: self.category,
            active: self.active,
        })
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct PlatformOrder {
    /// Every platform id, once, in the new order.
    pub ids: Vec<i64>,
}

#[utoipa::path(
    get, path = "/platforms", tag = "platforms", operation_id = "listPlatforms",
    responses((status = OK, description = "All platforms, active or not, in list order", body = Vec<Platform>))
)]
pub async fn list(State(state): State<AppState>) -> ApiResult<Json<Vec<Platform>>> {
    Ok(Json(platforms::list(&state.db).await?))
}

#[utoipa::path(
    post, path = "/platforms", tag = "platforms", operation_id = "createPlatform",
    request_body = PlatformInput,
    responses(
        (status = CREATED, body = Platform),
        (status = CONFLICT, description = "`name_taken`, `slug_taken`", body = ErrorBody),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody),
    )
)]
pub async fn create(
    State(state): State<AppState>,
    ApiJson(input): ApiJson<PlatformInput>,
) -> ApiResult<(StatusCode, Json<Platform>)> {
    let id = platforms::create(&state.db, &input.validate()?).await?;
    Ok((
        StatusCode::CREATED,
        Json(platforms::get(&state.db, id).await?),
    ))
}

#[utoipa::path(
    put, path = "/platforms/{id}", tag = "platforms", operation_id = "updatePlatform",
    params(("id" = i64, Path)),
    request_body = PlatformInput,
    responses(
        (status = OK, body = Platform),
        (status = NOT_FOUND, body = ErrorBody),
        (status = CONFLICT, description = "`name_taken`, `slug_taken`", body = ErrorBody),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody),
    )
)]
pub async fn update(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<i64>,
    ApiJson(input): ApiJson<PlatformInput>,
) -> ApiResult<Json<Platform>> {
    platforms::update(&state.db, id, &input.validate()?).await?;
    Ok(Json(platforms::get(&state.db, id).await?))
}

#[utoipa::path(
    delete, path = "/platforms/{id}", tag = "platforms", operation_id = "deletePlatform",
    params(("id" = i64, Path)),
    responses(
        (status = NO_CONTENT),
        (status = NOT_FOUND, body = ErrorBody),
        (status = CONFLICT, description = "`platform_in_use`: releases or devices use it", body = ErrorBody),
    )
)]
pub async fn delete(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<i64>,
) -> ApiResult<StatusCode> {
    platforms::delete(&state.db, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    put, path = "/platforms/order", tag = "platforms", operation_id = "reorderPlatforms",
    request_body = PlatformOrder,
    responses(
        (status = OK, body = Vec<Platform>),
        (status = UNPROCESSABLE_ENTITY, description = "`invalid_order`", body = ErrorBody),
    )
)]
pub async fn reorder(
    State(state): State<AppState>,
    ApiJson(order): ApiJson<PlatformOrder>,
) -> ApiResult<Json<Vec<Platform>>> {
    platforms::reorder(&state.db, &order.ids).await?;
    Ok(Json(platforms::list(&state.db).await?))
}
