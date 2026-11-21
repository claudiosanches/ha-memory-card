use std::sync::Arc;

use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

use crate::AppState;
use crate::db::games::{self, GameDetail};
use crate::error::{ApiError, ApiJson, ApiPath, ApiResult, ErrorBody};
use crate::igdb::sync::{self, SyncStatus};
use crate::igdb::{Igdb, SearchHit, store};

fn client(state: &AppState) -> ApiResult<Arc<Igdb>> {
    state
        .igdb
        .clone()
        .ok_or(ApiError::Unavailable("igdb_not_configured"))
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct SearchQuery {
    #[serde(default)]
    pub q: String,
}

#[utoipa::path(
    get, path = "/igdb/search", tag = "igdb", operation_id = "searchIgdb",
    params(SearchQuery),
    responses(
        (status = OK, body = Vec<SearchHit>),
        (status = SERVICE_UNAVAILABLE, description = "`igdb_not_configured`", body = ErrorBody),
        (status = BAD_GATEWAY, description = "`igdb_failed`", body = ErrorBody),
    )
)]
pub async fn search(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> ApiResult<Json<Vec<SearchHit>>> {
    let igdb = client(&state)?;
    let text = query.q.trim();
    if text.is_empty() {
        return Ok(Json(Vec::new()));
    }
    Ok(Json(igdb.search(text).await?))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct LinkInput {
    pub igdb_id: i64,
    /// Replace title, summary, game modes, player counts and release titles
    /// and dates with IGDB's where IGDB has them. Otherwise only empty ones are filled.
    #[serde(default)]
    pub overwrite: bool,
}

/// Links the game to an IGDB game and fills its metadata. Posting the current
/// IGDB id refreshes it.
#[utoipa::path(
    post, path = "/games/{id}/igdb", tag = "igdb", operation_id = "linkGame",
    params(("id" = i64, Path)),
    request_body = LinkInput,
    responses(
        (status = OK, body = GameDetail),
        (status = NOT_FOUND, body = ErrorBody),
        (status = CONFLICT, description = "`igdb_linked`: another game has this IGDB id", body = ErrorBody),
        (status = UNPROCESSABLE_ENTITY, description = "`igdb_not_found`", body = ErrorBody),
        (status = SERVICE_UNAVAILABLE, description = "`igdb_not_configured`", body = ErrorBody),
        (status = BAD_GATEWAY, description = "`igdb_failed`", body = ErrorBody),
    )
)]
pub async fn link(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<i64>,
    ApiJson(input): ApiJson<LinkInput>,
) -> ApiResult<Json<GameDetail>> {
    let igdb = client(&state)?;
    games::get(&state.db, id).await?;
    let data = igdb
        .games(&[input.igdb_id])
        .await?
        .into_iter()
        .next()
        .ok_or(ApiError::invalid("igdb_not_found", "igdb_id"))?;
    store::apply(
        &state.db,
        &state.config.covers_dir(),
        &igdb,
        id,
        &data,
        input.overwrite,
    )
    .await?;
    Ok(Json(games::get(&state.db, id).await?))
}

#[utoipa::path(
    get, path = "/igdb/sync", tag = "igdb", operation_id = "getSync",
    responses((status = OK, body = SyncStatus))
)]
pub async fn sync_status(State(state): State<AppState>) -> Json<SyncStatus> {
    Json(state.sync.lock().expect("sync status lock").clone())
}

#[utoipa::path(
    post, path = "/igdb/sync", tag = "igdb", operation_id = "startSync",
    responses(
        (status = ACCEPTED, body = SyncStatus),
        (status = CONFLICT, description = "`sync_running`", body = ErrorBody),
        (status = SERVICE_UNAVAILABLE, description = "`igdb_not_configured`", body = ErrorBody),
    )
)]
pub async fn start_sync(
    State(state): State<AppState>,
) -> ApiResult<(StatusCode, Json<SyncStatus>)> {
    Ok((StatusCode::ACCEPTED, Json(sync::start(&state)?)))
}
