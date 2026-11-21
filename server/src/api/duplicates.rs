use axum::Json;
use axum::extract::State;

use crate::AppState;
use crate::db::duplicates::{self, DuplicateGroup};
use crate::error::ApiResult;

#[utoipa::path(
    get, path = "/duplicates", tag = "duplicates", operation_id = "listDuplicates",
    responses((status = OK, description = "Games with two or more owned copies", body = Vec<DuplicateGroup>))
)]
pub async fn list(State(state): State<AppState>) -> ApiResult<Json<Vec<DuplicateGroup>>> {
    Ok(Json(duplicates::list(&state.db).await?))
}
