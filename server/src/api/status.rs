use axum::Json;
use axum::extract::State;
use serde::Serialize;
use utoipa::ToSchema;

use crate::AppState;
use crate::error::{ApiError, ErrorBody};
use crate::sensors::SensorsStatus;

#[derive(Debug, Serialize, ToSchema)]
pub struct Status {
    pub version: &'static str,
    /// IGDB credentials are set and external services are on.
    pub igdb_configured: bool,
    /// False when the app options keep Memory Card offline.
    pub external_services: bool,
    /// Games with at least one owned copy.
    pub games: i64,
    pub owned_copies: i64,
    /// Whether Home Assistant sensors are being published.
    pub sensors: SensorsStatus,
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
    let games: i64 = sqlx::query_scalar(
        "SELECT COUNT(DISTINCT r.game_id) FROM releases r
           JOIN copies c ON c.release_id = r.id WHERE c.status = 'owned'",
    )
    .fetch_one(&state.db)
    .await?;
    let owned_copies: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM copies WHERE status = 'owned'")
            .fetch_one(&state.db)
            .await?;

    Ok(Json(Status {
        version: env!("CARGO_PKG_VERSION"),
        igdb_configured: state.igdb.is_some(),
        external_services: state.config.external_services,
        games,
        owned_copies,
        sensors: *state.sensors.lock().expect("sensors status lock"),
    }))
}
