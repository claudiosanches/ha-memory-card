use std::sync::atomic::{AtomicU64, Ordering};

use axum::Json;
use axum::extract::State;
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use time::OffsetDateTime;
use utoipa::ToSchema;

use super::body::{CsvBody, ImageBody};
use super::covers::Image;
use super::games::{date, price};
use super::input::{NAME_MAX, NOTES_MAX, optional, required};
use crate::AppState;
use crate::access::Actor;
use crate::covers;
use crate::db::devices::{self, Device, DeviceFields, DeviceKind};
use crate::db::photos;
use crate::domain::{self, Condition, CopyStatus, Region};
use crate::error::{ApiError, ApiJson, ApiPath, ApiResult, ErrorBody};
use crate::images;
use crate::transfer::hardware::{self, HardwarePreview, HardwareResult};

const PHOTO_MAX_BYTES: usize = 10 * 1024 * 1024;

/// Keeps file names apart when uploads land in the same millisecond.
static UPLOADS: AtomicU64 = AtomicU64::new(0);

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
    /// Platforms a console, handheld or PC plays.
    #[serde(default, deserialize_with = "super::input::id_list")]
    pub platform_ids: Vec<i64>,
    /// The platform a controller, accessory or toy is for.
    #[serde(default)]
    pub platform_id: Option<i64>,
    /// `wanted` puts it on the wishlist. Left out, it is owned.
    #[serde(default)]
    pub status: Option<CopyStatus>,
    #[serde(default)]
    pub region: Option<Region>,
    #[serde(default)]
    pub condition: Option<Condition>,
    #[serde(default)]
    pub store_id: Option<i64>,
    /// `YYYY-MM-DD`.
    #[serde(default)]
    pub purchased_on: Option<String>,
    /// Decimal text such as `299.99`; needs `currency`.
    #[serde(default)]
    pub price: Option<String>,
    #[serde(default)]
    pub currency: Option<String>,
    /// Decimal text: what it would sell for now; needs `estimated_currency`.
    #[serde(default)]
    pub estimated_value: Option<String>,
    #[serde(default)]
    pub estimated_currency: Option<String>,
    #[serde(default)]
    pub serial_number: Option<String>,
    /// UPC or EAN; spaces and hyphens are ignored.
    #[serde(default)]
    pub barcode: Option<String>,
}

fn default_active() -> bool {
    true
}

impl DeviceInput {
    /// A new console with nothing else set, for an import row to fill in.
    pub fn blank() -> Self {
        Self {
            name: String::new(),
            slug: None,
            kind: DeviceKind::Console,
            model: None,
            notes: None,
            active: true,
            platform_ids: Vec::new(),
            platform_id: None,
            status: None,
            region: None,
            condition: None,
            store_id: None,
            purchased_on: None,
            price: None,
            currency: None,
            estimated_value: None,
            estimated_currency: None,
            serial_number: None,
            barcode: None,
        }
    }

    /// The device as stored, for an import row to change.
    pub fn from_device(device: &Device) -> Self {
        Self {
            name: device.name.clone(),
            slug: None,
            kind: device.kind,
            model: device.model.clone(),
            notes: device.notes.clone(),
            active: device.active,
            platform_ids: device.platform_ids.clone(),
            platform_id: device.platform_id,
            status: Some(device.status),
            region: device.region,
            condition: device.condition,
            store_id: device.store_id,
            purchased_on: device.purchased_on.clone(),
            price: device.price.clone(),
            currency: device.currency.clone(),
            estimated_value: device.estimated_value.clone(),
            estimated_currency: device.estimated_currency.clone(),
            serial_number: device.serial_number.clone(),
            barcode: device.barcode.clone(),
        }
    }

    pub fn validate(&self) -> ApiResult<DeviceFields> {
        if self.kind.holds_games() {
            if self.platform_id.is_some() {
                return Err(ApiError::invalid("not_for_this_kind", "platform_id"));
            }
        } else if !self.platform_ids.is_empty() {
            return Err(ApiError::invalid("not_for_this_kind", "platform_ids"));
        }
        Ok(DeviceFields {
            name: required(&self.name, "name", NAME_MAX)?,
            slug: crate::slug::input(self.slug.as_deref())?,
            kind: self.kind,
            model: optional(self.model.as_deref(), "model", NAME_MAX)?,
            notes: optional(self.notes.as_deref(), "notes", NOTES_MAX)?,
            active: self.active,
            platform_ids: self.platform_ids.clone(),
            platform_id: self.platform_id,
            status: self.status.unwrap_or(CopyStatus::Owned),
            region: self.region,
            condition: self.condition,
            store_id: self.store_id,
            purchased_on: date(self.purchased_on.as_deref(), "purchased_on")?,
            price: price(
                self.price.as_deref(),
                self.currency.as_deref(),
                ("price", "currency"),
            )?,
            estimated_value: price(
                self.estimated_value.as_deref(),
                self.estimated_currency.as_deref(),
                ("estimated_value", "estimated_currency"),
            )?,
            serial_number: optional(self.serial_number.as_deref(), "serial_number", NAME_MAX)?,
            barcode: match self
                .barcode
                .as_deref()
                .map(str::trim)
                .filter(|b| !b.is_empty())
            {
                None => None,
                Some(code) => Some(
                    domain::barcode(code).ok_or(ApiError::invalid("invalid_barcode", "barcode"))?,
                ),
            },
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
        (status = UNPROCESSABLE_ENTITY, description = "Includes `unknown_platform` and `device_has_copies`", body = ErrorBody),
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
    let _ = tokio::fs::remove_dir_all(state.config.covers_dir().join(covers::device_dir(id))).await;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post, path = "/devices/{id}/photos", tag = "devices", operation_id = "addDevicePhoto",
    params(("id" = i64, Path)),
    request_body(content = Image, content_type = "application/octet-stream"),
    responses(
        (status = CREATED, body = Device),
        (status = NOT_FOUND, body = ErrorBody),
        (status = CONFLICT, description = "`too_many_photos`: 20 per device", body = ErrorBody),
        (status = UNPROCESSABLE_ENTITY, description = "`invalid_image` or `image_too_large`", body = ErrorBody),
    )
)]
pub async fn add_photo(
    State(state): State<AppState>,
    Actor(by): Actor,
    ApiPath(id): ApiPath<i64>,
    ImageBody(body): ImageBody,
) -> ApiResult<(StatusCode, Json<Device>)> {
    if body.len() > PHOTO_MAX_BYTES {
        return Err(ApiError::Invalid {
            code: "image_too_large",
            field: None,
        });
    }
    let image = images::clean(&body).map_err(|rejected| ApiError::Invalid {
        code: match rejected {
            images::Rejected::Invalid => "invalid_image",
            images::Rejected::TooLarge => "image_too_large",
        },
        field: None,
    })?;
    // Checked again when the row is added; this spares writing a file.
    photos::device_room_for(&state.db, id).await?;

    let dir = state.config.covers_dir();
    let stamp = OffsetDateTime::now_utc().unix_timestamp_nanos() / 1_000_000;
    let n = UPLOADS.fetch_add(1, Ordering::Relaxed);
    let name = format!("photo-{stamp}-{n}.{}", image.extension);
    let path = covers::write_in(&dir, &covers::device_dir(id), &name, &image.bytes)
        .await
        .map_err(|err| ApiError::Internal(format!("saving photo: {err}")))?;
    if let Err(err) = photos::add_for_device(&state.db, id, &path, &by).await {
        covers::remove_file(&dir, &path).await;
        return Err(err);
    }
    Ok((
        StatusCode::CREATED,
        Json(devices::get(&state.db, id).await?),
    ))
}

#[utoipa::path(
    delete, path = "/device-photos/{id}", tag = "devices", operation_id = "deleteDevicePhoto",
    params(("id" = i64, Path)),
    responses(
        (status = OK, description = "The photo's device", body = Device),
        (status = NOT_FOUND, body = ErrorBody),
    )
)]
pub async fn delete_photo(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<i64>,
) -> ApiResult<Json<Device>> {
    let (device_id, path) = photos::delete_for_device(&state.db, id).await?;
    covers::remove_file(&state.config.covers_dir(), &path).await;
    Ok(Json(devices::get(&state.db, device_id).await?))
}

fn csv_file(name: &str, body: Vec<u8>) -> Response {
    (
        [
            (header::CONTENT_TYPE, "text/csv; charset=utf-8".to_owned()),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"memory-card-{name}.csv\""),
            ),
            (header::CACHE_CONTROL, "no-store".to_owned()),
        ],
        body,
    )
        .into_response()
}

#[utoipa::path(
    get, path = "/hardware.csv", tag = "devices", operation_id = "exportHardware",
    responses((status = OK, description = "One row per device", content_type = "text/csv", body = String))
)]
pub async fn export_csv(State(state): State<AppState>) -> ApiResult<Response> {
    Ok(csv_file("hardware", hardware::export(&state.db).await?))
}

#[utoipa::path(
    get, path = "/hardware/template.csv", tag = "devices", operation_id = "hardwareTemplate",
    responses((status = OK, description = "Header row only", content_type = "text/csv", body = String))
)]
pub async fn template_csv() -> Response {
    csv_file("hardware-template", hardware::template())
}

#[utoipa::path(
    post, path = "/hardware/import/preview", tag = "devices", operation_id = "previewHardwareImport",
    request_body(content = String, content_type = "text/csv"),
    responses((status = OK, body = HardwarePreview))
)]
pub async fn preview_import(
    State(state): State<AppState>,
    CsvBody(body): CsvBody,
) -> ApiResult<Json<HardwarePreview>> {
    let mut conn = state.db.acquire().await?;
    Ok(Json(hardware::preview(
        hardware::plan(&mut conn, &body).await?,
    )))
}

#[utoipa::path(
    post, path = "/hardware/import", tag = "devices", operation_id = "runHardwareImport",
    request_body(content = String, content_type = "text/csv"),
    responses(
        (status = OK, body = HardwareResult),
        (status = UNPROCESSABLE_ENTITY, description = "`import_invalid`: preview the file to see why", body = ErrorBody),
    )
)]
pub async fn run_import(
    State(state): State<AppState>,
    CsvBody(body): CsvBody,
) -> ApiResult<Json<HardwareResult>> {
    Ok(Json(hardware::import(&state.db, &body).await?))
}
