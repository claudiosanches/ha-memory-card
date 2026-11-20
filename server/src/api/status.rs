use axum::Json;
use axum::extract::State;
use serde::Serialize;
use utoipa::ToSchema;

use crate::AppState;
use crate::error::{ApiError, ErrorBody};

#[derive(Debug, Serialize, ToSchema)]
pub struct Status {
    pub version: &'static str,
    pub igdb_configured: bool,
    pub games: i64,
    pub owned_copies: i64,
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
    let games: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM games")
        .fetch_one(&state.db)
        .await?;
    let owned_copies: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM copies WHERE status = 'owned'")
            .fetch_one(&state.db)
            .await?;

    Ok(Json(Status {
        version: env!("CARGO_PKG_VERSION"),
        igdb_configured: state.config.igdb.is_some(),
        games,
        owned_copies,
    }))
}
