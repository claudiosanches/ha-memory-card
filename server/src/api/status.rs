use axum::Json;
use axum::extract::State;
use serde::Serialize;
use utoipa::ToSchema;

use crate::AppState;
use crate::error::{ApiError, ErrorBody};
use crate::sensors::SensorsStatus;
use crate::tags::TagsStatus;

#[derive(Debug, Serialize, ToSchema)]
pub struct Status {
    pub version: &'static str,
    /// IGDB credentials are set and external services are on.
    pub igdb_configured: bool,
    /// False when the app options keep Memory Card offline.
    pub external_services: bool,
    /// Currency of the copy priced last; new prices start with it.
    pub last_currency: Option<String>,
    /// Whether Home Assistant sensors are being published.
    pub sensors: SensorsStatus,
    /// Whether Memory Card hears Home Assistant tag scans.
    pub tags: TagsStatus,
}

#[utoipa::path(
    get,
    path = "/status",
    tag = "status",
    operation_id = "getStatus",
    responses(
        (status = OK, body = Status),
        (status = INTERNAL_SERVER_ERROR, body = ErrorBody),
    )
)]
pub async fn get_status(State(state): State<AppState>) -> Result<Json<Status>, ApiError> {
    let last_currency: Option<String> = sqlx::query_scalar(
        "SELECT currency FROM copies WHERE currency IS NOT NULL
         ORDER BY updated_at DESC, id DESC LIMIT 1",
    )
    .fetch_optional(&state.db)
    .await?;

    Ok(Json(Status {
        version: env!("CARGO_PKG_VERSION"),
        igdb_configured: state.igdb.is_some(),
        external_services: state.config.external_services,
        last_currency,
        sensors: *state.sensors.lock().expect("sensors status lock"),
        tags: *state.tag_status.lock().expect("tag status lock"),
    }))
}
