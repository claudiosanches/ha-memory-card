use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use serde::Deserialize;
use utoipa::ToSchema;

use super::input::{NAME_MAX, NOTES_MAX, optional, required};
use crate::AppState;
use crate::db::devices::{self, Device, DeviceFields, DeviceKind};
use crate::error::{ApiJson, ApiPath, ApiResult, ErrorBody};

#[derive(Debug, Deserialize, ToSchema)]
pub struct DeviceInput {
    pub name: String,
    /// Names it in library links. Blank makes one from the name for a new
    /// one and keeps the current one on update.
    #[serde(default)]
    pub slug: Option<String>,
    pub kind: DeviceKind,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default = "default_active")]
    pub active: bool,
    #[serde(default)]
    pub platform_ids: Vec<i64>,
}

fn default_active() -> bool {
    true
}

impl DeviceInput {
    fn validate(&self) -> ApiResult<DeviceFields> {
        Ok(DeviceFields {
            name: required(&self.name, "name", NAME_MAX)?,
            slug: crate::slug::input(self.slug.as_deref())?,
            kind: self.kind,
            model: optional(self.model.as_deref(), "model", NAME_MAX)?,
            notes: optional(self.notes.as_deref(), "notes", NOTES_MAX)?,
            active: self.active,
            platform_ids: self.platform_ids.clone(),
        })
    }
}

#[utoipa::path(
    get, path = "/devices", tag = "devices", operation_id = "listDevices",
    responses((status = OK, body = Vec<Device>))
)]
pub async fn list(State(state): State<AppState>) -> ApiResult<Json<Vec<Device>>> {
    Ok(Json(devices::list(&state.db).await?))
}

#[utoipa::path(
    post, path = "/devices", tag = "devices", operation_id = "createDevice",
    request_body = DeviceInput,
    responses(
        (status = CREATED, body = Device),
        (status = CONFLICT, description = "`name_taken`, `slug_taken`", body = ErrorBody),
        (status = UNPROCESSABLE_ENTITY, description = "Includes `unknown_platform`", body = ErrorBody),
    )
)]
pub async fn create(
    State(state): State<AppState>,
    ApiJson(input): ApiJson<DeviceInput>,
) -> ApiResult<(StatusCode, Json<Device>)> {
    let id = devices::create(&state.db, &input.validate()?).await?;
    Ok((
        StatusCode::CREATED,
        Json(devices::get(&state.db, id).await?),
    ))
}

#[utoipa::path(
    put, path = "/devices/{id}", tag = "devices", operation_id = "updateDevice",
    params(("id" = i64, Path)),
    request_body = DeviceInput,
    responses(
        (status = OK, body = Device),
        (status = NOT_FOUND, body = ErrorBody),
        (status = CONFLICT, description = "`name_taken`, `slug_taken`", body = ErrorBody),
        (status = UNPROCESSABLE_ENTITY, description = "Includes `unknown_platform`", body = ErrorBody),
    )
)]
pub async fn update(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<i64>,
    ApiJson(input): ApiJson<DeviceInput>,
) -> ApiResult<Json<Device>> {
    devices::update(&state.db, id, &input.validate()?).await?;
    Ok(Json(devices::get(&state.db, id).await?))
}

#[utoipa::path(
    delete, path = "/devices/{id}", tag = "devices", operation_id = "deleteDevice",
    params(("id" = i64, Path)),
    responses(
        (status = NO_CONTENT, description = "Deleted; its copies keep existing with no device"),
        (status = NOT_FOUND, body = ErrorBody),
    )
)]
pub async fn delete(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<i64>,
) -> ApiResult<StatusCode> {
    devices::delete(&state.db, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
