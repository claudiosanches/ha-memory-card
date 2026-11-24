use axum::Json;
use axum::extract::{Query, State};

use crate::AppState;
use crate::db::household::{self, HouseholdQuery, PersonTotals};
use crate::error::{ApiError, ApiResult, ErrorBody};

#[utoipa::path(
    get, path = "/household", tag = "stats", operation_id = "getHousehold",
    params(HouseholdQuery),
    responses(
        (status = OK, description = "Everyone who keeps play, the most minutes first", body = Vec<PersonTotals>),
        (status = UNPROCESSABLE_ENTITY, description = "`invalid_filter`", body = ErrorBody),
    )
)]
pub async fn get(
    State(state): State<AppState>,
    query: Result<Query<HouseholdQuery>, axum::extract::rejection::QueryRejection>,
) -> ApiResult<Json<Vec<PersonTotals>>> {
    let Query(query) = query.map_err(|_| ApiError::Invalid {
        code: "invalid_filter",
        field: None,
    })?;
    let period = query.period.unwrap_or_default();
    Ok(Json(household::totals(&state.db, period).await?))
}
