use axum::Json;
use axum::extract::State;

use crate::AppState;
use crate::access::Actor;
use crate::db::stats::{self, Stats};
use crate::error::{ApiResult, ErrorBody};

#[utoipa::path(
    get, path = "/stats", tag = "stats", operation_id = "getStats",
    responses((status = OK, body = Stats), (status = INTERNAL_SERVER_ERROR, body = ErrorBody))
)]
pub async fn get(State(state): State<AppState>, Actor(by): Actor) -> ApiResult<Json<Stats>> {
    Ok(Json(stats::get(&state.db, &by).await?))
}
