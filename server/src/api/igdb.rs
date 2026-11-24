use std::sync::Arc;

use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

use crate::AppState;
use crate::access::Actor;
use crate::db::games::{self, GameDetail};
use crate::db::titles::{self, TitleDifference};
use crate::error::{ApiError, ApiJson, ApiPath, ApiResult, ErrorBody};
use crate::igdb::matches::{self, MatchError, MatchGroup};
use crate::igdb::sync::{self, SyncStatus};
use crate::igdb::{Igdb, SearchHit, store};

fn client(state: &AppState) -> ApiResult<Arc<Igdb>> {
    state
        .igdb
        .clone()
        .ok_or(ApiError::Unavailable("igdb_not_configured"))
}

/// Longer searches are not titles.
const SEARCH_MAX: usize = 200;

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct SearchQuery {
    /// At most 200 characters.
    #[serde(default)]
    pub q: String,
}

#[utoipa::path(
    get, path = "/igdb/search", tag = "igdb", operation_id = "searchIgdb",
    params(SearchQuery),
    responses(
        (status = OK, body = Vec<SearchHit>),
        (status = UNPROCESSABLE_ENTITY, description = "`too_long`", body = ErrorBody),
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
    if text.chars().count() > SEARCH_MAX {
        return Err(ApiError::invalid("too_long", "q"));
    }
    let mut hits = igdb.search(text).await?;
    mark_collected(&state, &mut hits).await?;
    Ok(Json(hits))
}

/// Fills `game_id` for hits already in the collection.
async fn mark_collected(state: &AppState, hits: &mut [SearchHit]) -> ApiResult<()> {
    let ids: Vec<i64> = hits.iter().map(|hit| hit.igdb_id).collect();
    let owned = crate::db::igdb::games_for(&state.db, &ids).await?;
    for hit in hits {
        hit.game_id = owned
            .iter()
            .find(|(igdb_id, _)| *igdb_id == hit.igdb_id)
            .map(|(_, id)| *id);
    }
    Ok(())
}

#[utoipa::path(
    get, path = "/games/{id}/igdb-contents", tag = "igdb", operation_id = "igdbBundleContents",
    params(("id" = i64, Path)),
    responses(
        (status = OK, description = "Games IGDB lists in this bundle; empty when the game has no IGDB link", body = Vec<SearchHit>),
        (status = NOT_FOUND, body = ErrorBody),
        (status = SERVICE_UNAVAILABLE, description = "`igdb_not_configured`", body = ErrorBody),
        (status = BAD_GATEWAY, description = "`igdb_failed`", body = ErrorBody),
    )
)]
pub async fn bundle_contents(
    State(state): State<AppState>,
    Actor(by): Actor,
    ApiPath(id): ApiPath<i64>,
) -> ApiResult<Json<Vec<SearchHit>>> {
    let igdb = client(&state)?;
    let Some(bundle) = games::get(&state.db, id, &by).await?.igdb_id else {
        return Ok(Json(Vec::new()));
    };
    let mut hits = igdb.bundle_contents(bundle).await?;
    mark_collected(&state, &mut hits).await?;
    Ok(Json(hits))
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
    Actor(by): Actor,
    ApiPath(id): ApiPath<i64>,
    ApiJson(input): ApiJson<LinkInput>,
) -> ApiResult<Json<GameDetail>> {
    let igdb = client(&state)?;
    games::get(&state.db, id, &by).await?;
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
    Ok(Json(games::get(&state.db, id, &by).await?))
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct MatchQuery {
    /// Continue after this game id.
    #[serde(default)]
    pub after: i64,
    /// Games per page, at most 10.
    pub limit: Option<i64>,
}

#[utoipa::path(
    get, path = "/igdb/matches", tag = "igdb", operation_id = "igdbMatches",
    params(MatchQuery),
    responses(
        (status = OK, description = "Games with no IGDB link, by id, with IGDB's closest names", body = Vec<MatchGroup>),
        (status = SERVICE_UNAVAILABLE, description = "`igdb_not_configured`", body = ErrorBody),
        (status = BAD_GATEWAY, description = "`igdb_failed`", body = ErrorBody),
    )
)]
pub async fn matches(
    State(state): State<AppState>,
    Query(query): Query<MatchQuery>,
) -> ApiResult<Json<Vec<MatchGroup>>> {
    let igdb = client(&state)?;
    let limit = query.limit.unwrap_or(5).clamp(1, 10);
    let groups = matches::next(&state.db, &igdb, query.after, limit)
        .await
        .map_err(|err| match err {
            MatchError::Database(err) => ApiError::from(err),
            MatchError::Igdb(err) => ApiError::from(err),
        })?;
    Ok(Json(groups))
}

#[utoipa::path(
    post, path = "/games/{id}/igdb-skip", tag = "igdb", operation_id = "skipIgdbMatch",
    params(("id" = i64, Path)),
    responses(
        (status = NO_CONTENT, description = "Match reviews stop offering the game"),
        (status = NOT_FOUND, body = ErrorBody),
    )
)]
pub async fn skip(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<i64>,
) -> ApiResult<axum::http::StatusCode> {
    crate::db::igdb::skip(&state.db, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
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

#[utoipa::path(
    get, path = "/igdb/titles", tag = "igdb", operation_id = "listTitleDifferences",
    responses((status = OK, description = "Linked games whose title differs from IGDB's, except titles the user kept", body = Vec<TitleDifference>))
)]
pub async fn title_differences(
    State(state): State<AppState>,
) -> ApiResult<Json<Vec<TitleDifference>>> {
    Ok(Json(titles::differences(&state.db).await?))
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct TitleChoices {
    /// Games to rename to their IGDB title.
    #[serde(default, deserialize_with = "super::input::id_list")]
    pub use_igdb: Vec<i64>,
    /// Games whose title stays; they leave the list until IGDB renames them.
    #[serde(default, deserialize_with = "super::input::id_list")]
    pub keep: Vec<i64>,
}

#[utoipa::path(
    post, path = "/igdb/titles", tag = "igdb", operation_id = "resolveTitles",
    request_body = TitleChoices,
    responses(
        (status = OK, description = "The differences left", body = Vec<TitleDifference>),
        (status = UNPROCESSABLE_ENTITY, description = "`invalid_body`", body = ErrorBody),
    )
)]
pub async fn resolve_titles(
    State(state): State<AppState>,
    Actor(by): Actor,
    ApiJson(choices): ApiJson<TitleChoices>,
) -> ApiResult<Json<Vec<TitleDifference>>> {
    titles::resolve(&state.db, &choices.use_igdb, &choices.keep, &by).await?;
    Ok(Json(titles::differences(&state.db).await?))
}
