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
    /// Where new prices start: the `currency` option, else the currency of
    /// the copy priced last, else Home Assistant's.
    pub default_currency: Option<String>,
    /// Home Assistant's language, for browsers where nobody picked one.
    pub home_assistant_language: Option<String>,
    /// How dates are written, from the app options.
    pub date_format: crate::config::DateFormat,
    /// Whether manage and hardware forms show the slug field, from the app options.
    pub show_slugs: bool,
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
    let last_currency = crate::db::games::last_currency(&state.db).await?;

    let regional = crate::regional::get(&state).await.unwrap_or_default();
    let default_currency = state
        .config
        .currency
        .clone()
        .or(last_currency)
        .or(regional.currency);

    Ok(Json(Status {
        version: env!("CARGO_PKG_VERSION"),
        igdb_configured: state.igdb.is_some(),
        external_services: state.config.external_services,
        default_currency,
        home_assistant_language: regional.language,
        date_format: state.config.date_format,
        show_slugs: state.config.show_slugs,
        sensors: *state.sensors.lock().expect("sensors status lock"),
        tags: *state.tag_status.lock().expect("tag status lock"),
    }))
}
