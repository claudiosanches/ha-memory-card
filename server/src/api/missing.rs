use axum::Json;
use axum::extract::State;

use crate::AppState;
use crate::db::missing::{self, MissingDetails};
use crate::error::ApiResult;

#[utoipa::path(
    get, path = "/missing-details", tag = "games", operation_id = "listMissingDetails",
    responses((status = OK, description = "Games lacking at least one detail, in title order", body = Vec<MissingDetails>))
)]
pub async fn list(State(state): State<AppState>) -> ApiResult<Json<Vec<MissingDetails>>> {
    Ok(Json(missing::list(&state.db, state.igdb.is_some()).await?))
}
