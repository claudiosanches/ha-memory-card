use axum::Json;
use axum::extract::State;

use crate::AppState;
use crate::db::suggestions::{self, Suggestion, SuggestionFilter};
use crate::error::{ApiJson, ApiResult, ErrorBody};

#[utoipa::path(
    post, path = "/suggestions", tag = "suggestions", operation_id = "suggestGame",
    request_body = SuggestionFilter,
    responses(
        (status = OK, description = "A random owned game, not one picked in the last day when others match", body = Suggestion),
        (status = NOT_FOUND, description = "`no_candidates`", body = ErrorBody),
        (status = UNPROCESSABLE_ENTITY, description = "`invalid_body`", body = ErrorBody),
    )
)]
pub async fn suggest(
    State(state): State<AppState>,
    ApiJson(filter): ApiJson<SuggestionFilter>,
) -> ApiResult<Json<Suggestion>> {
    Ok(Json(suggestions::suggest(&state.db, filter).await?))
}
