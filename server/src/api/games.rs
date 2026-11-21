use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use time::Date;
use time::macros::format_description;
use utoipa::ToSchema;

use super::input::{NOTES_MAX, optional, required};
use crate::AppState;
use crate::covers::{self, Owner};
use crate::db::games::{self, CopyFields, GameDetail, GameFields, PlayerCounts, ReleaseFields};
use crate::db::library::{self, GameSummary, LibraryFilter};
use crate::domain::{Condition, Format, Region};
use crate::error::{ApiError, ApiJson, ApiPath, ApiResult, ErrorBody};

const TITLE_MAX: usize = 200;
const SUMMARY_MAX: usize = 5000;
const CODE_MAX: usize = 40;
const PLAYERS_MAX: i64 = 99;

#[derive(Debug, Deserialize, ToSchema)]
pub struct NewGameInput {
    pub title: String,
    /// A play status key.
    #[serde(default)]
    pub play_status: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct GameInput {
    pub title: String,
    #[serde(default)]
    pub summary: Option<String>,
    /// A play status key.
    #[serde(default)]
    pub play_status: Option<String>,
    /// `YYYY-MM-DD`.
    #[serde(default)]
    pub last_played_at: Option<String>,
    /// Must be a copy of this game.
    #[serde(default)]
    pub last_played_copy_id: Option<i64>,
    #[serde(default)]
    pub local_max_players: Option<i64>,
    #[serde(default)]
    pub online_max_players: Option<i64>,
    #[serde(default)]
    pub local_coop: Option<bool>,
    #[serde(default)]
    pub online_coop: Option<bool>,
    #[serde(default)]
    pub game_mode_ids: Vec<i64>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CopyInput {
    pub platform_id: i64,
    pub region: Region,
    #[serde(default)]
    pub product_code: Option<String>,
    pub format: Format,
    #[serde(default)]
    pub device_id: Option<i64>,
    #[serde(default)]
    pub store_id: Option<i64>,
    /// Physical copies only.
    #[serde(default)]
    pub condition: Option<Condition>,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ReleaseInput {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub product_code: Option<String>,
    /// `YYYY-MM-DD`.
    #[serde(default)]
    pub release_date: Option<String>,
    #[serde(default)]
    pub local_max_players: Option<i64>,
    #[serde(default)]
    pub online_max_players: Option<i64>,
    #[serde(default)]
    pub local_coop: Option<bool>,
    #[serde(default)]
    pub online_coop: Option<bool>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct GameMode {
    pub id: i64,
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct Genre {
    pub id: i64,
    pub name: String,
}

fn date(value: Option<&str>, field: &'static str) -> ApiResult<Option<String>> {
    let Some(value) = value.map(str::trim).filter(|v| !v.is_empty()) else {
        return Ok(None);
    };
    Date::parse(value, format_description!("[year]-[month]-[day]"))
        .map(|_| Some(value.to_owned()))
        .map_err(|_| ApiError::invalid("invalid_date", field))
}

fn players(
    local_max: Option<i64>,
    online_max: Option<i64>,
    local_coop: Option<bool>,
    online_coop: Option<bool>,
) -> ApiResult<PlayerCounts> {
    for (value, field) in [
        (local_max, "local_max_players"),
        (online_max, "online_max_players"),
    ] {
        if value.is_some_and(|n| !(1..=PLAYERS_MAX).contains(&n)) {
            return Err(ApiError::invalid("out_of_range", field));
        }
    }
    Ok(PlayerCounts {
        local_max_players: local_max,
        online_max_players: online_max,
        local_coop,
        online_coop,
    })
}

impl GameInput {
    fn validate(&self) -> ApiResult<GameFields> {
        Ok(GameFields {
            title: required(&self.title, "title", TITLE_MAX)?,
            summary: optional(self.summary.as_deref(), "summary", SUMMARY_MAX)?,
            play_status: self.play_status.clone(),
            last_played_at: date(self.last_played_at.as_deref(), "last_played_at")?,
            last_played_copy_id: self.last_played_copy_id,
            players: players(
                self.local_max_players,
                self.online_max_players,
                self.local_coop,
                self.online_coop,
            )?,
            game_mode_ids: self.game_mode_ids.clone(),
        })
    }
}

impl CopyInput {
    fn validate(&self) -> ApiResult<CopyFields> {
        if self.condition.is_some() && self.format == Format::Digital {
            return Err(ApiError::invalid("condition_digital", "condition"));
        }
        Ok(CopyFields {
            platform_id: self.platform_id,
            region: self.region,
            product_code: optional(self.product_code.as_deref(), "product_code", CODE_MAX)?,
            format: self.format,
            device_id: self.device_id,
            store_id: self.store_id,
            condition: self.condition,
            notes: optional(self.notes.as_deref(), "notes", NOTES_MAX)?,
        })
    }
}

impl ReleaseInput {
    fn validate(&self) -> ApiResult<ReleaseFields> {
        Ok(ReleaseFields {
            title: optional(self.title.as_deref(), "title", TITLE_MAX)?,
            product_code: optional(self.product_code.as_deref(), "product_code", CODE_MAX)?,
            release_date: date(self.release_date.as_deref(), "release_date")?,
            players: players(
                self.local_max_players,
                self.online_max_players,
                self.local_coop,
                self.online_coop,
            )?,
        })
    }
}

#[utoipa::path(
    get, path = "/games", tag = "games", operation_id = "listGames",
    params(LibraryFilter),
    responses(
        (status = OK, body = Vec<GameSummary>),
        (status = UNPROCESSABLE_ENTITY, description = "`invalid_filter`", body = ErrorBody),
    )
)]
pub async fn list(
    State(state): State<AppState>,
    filter: Result<Query<LibraryFilter>, axum::extract::rejection::QueryRejection>,
) -> ApiResult<Json<Vec<GameSummary>>> {
    let Query(filter) = filter.map_err(|_| ApiError::Invalid {
        code: "invalid_filter",
        field: None,
    })?;
    Ok(Json(library::list(&state.db, &filter).await?))
}

#[utoipa::path(
    post, path = "/games", tag = "games", operation_id = "createGame",
    request_body = NewGameInput,
    responses((status = CREATED, body = GameDetail), (status = UNPROCESSABLE_ENTITY, body = ErrorBody))
)]
pub async fn create(
    State(state): State<AppState>,
    ApiJson(input): ApiJson<NewGameInput>,
) -> ApiResult<(StatusCode, Json<GameDetail>)> {
    let title = required(&input.title, "title", TITLE_MAX)?;
    let id = games::create(&state.db, &title, input.play_status.as_deref()).await?;
    Ok((StatusCode::CREATED, Json(games::get(&state.db, id).await?)))
}

#[utoipa::path(
    get, path = "/games/{id}", tag = "games", operation_id = "getGame",
    params(("id" = i64, Path)),
    responses((status = OK, body = GameDetail), (status = NOT_FOUND, body = ErrorBody))
)]
pub async fn get(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<i64>,
) -> ApiResult<Json<GameDetail>> {
    Ok(Json(games::get(&state.db, id).await?))
}

#[utoipa::path(
    put, path = "/games/{id}", tag = "games", operation_id = "updateGame",
    params(("id" = i64, Path)),
    request_body = GameInput,
    responses(
        (status = OK, body = GameDetail),
        (status = NOT_FOUND, body = ErrorBody),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody),
    )
)]
pub async fn update(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<i64>,
    ApiJson(input): ApiJson<GameInput>,
) -> ApiResult<Json<GameDetail>> {
    games::update(&state.db, id, &input.validate()?).await?;
    Ok(Json(games::get(&state.db, id).await?))
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct PlayStatusInput {
    /// A play status key; `null` clears it.
    pub play_status: Option<String>,
}

#[utoipa::path(
    put, path = "/games/{id}/play-status", tag = "games", operation_id = "setPlayStatus",
    params(("id" = i64, Path)),
    request_body = PlayStatusInput,
    responses(
        (status = OK, body = GameDetail),
        (status = NOT_FOUND, body = ErrorBody),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody),
    )
)]
pub async fn set_play_status(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<i64>,
    ApiJson(input): ApiJson<PlayStatusInput>,
) -> ApiResult<Json<GameDetail>> {
    games::set_play_status(&state.db, id, input.play_status.as_deref()).await?;
    Ok(Json(games::get(&state.db, id).await?))
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct LastPlayedInput {
    /// `YYYY-MM-DD`; `null` clears it.
    #[serde(default)]
    pub last_played_at: Option<String>,
    /// Must be a copy of this game; `null` clears it.
    #[serde(default)]
    pub last_played_copy_id: Option<i64>,
}

#[utoipa::path(
    put, path = "/games/{id}/last-played", tag = "games", operation_id = "setLastPlayed",
    params(("id" = i64, Path)),
    request_body = LastPlayedInput,
    responses(
        (status = OK, body = GameDetail),
        (status = NOT_FOUND, body = ErrorBody),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody),
    )
)]
pub async fn set_last_played(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<i64>,
    ApiJson(input): ApiJson<LastPlayedInput>,
) -> ApiResult<Json<GameDetail>> {
    let at = date(input.last_played_at.as_deref(), "last_played_at")?;
    games::set_last_played(&state.db, id, at.as_deref(), input.last_played_copy_id).await?;
    Ok(Json(games::get(&state.db, id).await?))
}

#[utoipa::path(
    delete, path = "/games/{id}", tag = "games", operation_id = "deleteGame",
    params(("id" = i64, Path)),
    responses(
        (status = NO_CONTENT, description = "Deleted with its releases and copies"),
        (status = NOT_FOUND, body = ErrorBody),
    )
)]
pub async fn delete(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<i64>,
) -> ApiResult<StatusCode> {
    let releases = games::delete(&state.db, id).await?;
    let dir = state.config.covers_dir();
    covers::keep_only(&dir, Owner::Game(id), None).await;
    for release in releases {
        covers::keep_only(&dir, Owner::Release(release), None).await;
    }
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post, path = "/games/{id}/copies", tag = "games", operation_id = "addCopy",
    params(("id" = i64, Path)),
    request_body = CopyInput,
    responses(
        (status = CREATED, description = "The game with the new copy", body = GameDetail),
        (status = NOT_FOUND, body = ErrorBody),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody),
    )
)]
pub async fn add_copy(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<i64>,
    ApiJson(input): ApiJson<CopyInput>,
) -> ApiResult<(StatusCode, Json<GameDetail>)> {
    games::add_copy(&state.db, id, &input.validate()?).await?;
    Ok((StatusCode::CREATED, Json(games::get(&state.db, id).await?)))
}

#[utoipa::path(
    put, path = "/copies/{id}", tag = "games", operation_id = "updateCopy",
    params(("id" = i64, Path)),
    request_body = CopyInput,
    responses(
        (status = OK, description = "The copy's game", body = GameDetail),
        (status = NOT_FOUND, body = ErrorBody),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody),
    )
)]
pub async fn update_copy(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<i64>,
    ApiJson(input): ApiJson<CopyInput>,
) -> ApiResult<Json<GameDetail>> {
    let game_id = games::update_copy(&state.db, id, &input.validate()?).await?;
    Ok(Json(games::get(&state.db, game_id).await?))
}

#[utoipa::path(
    delete, path = "/copies/{id}", tag = "games", operation_id = "deleteCopy",
    params(("id" = i64, Path)),
    responses(
        (status = OK, description = "The copy's game; the release stays", body = GameDetail),
        (status = NOT_FOUND, body = ErrorBody),
    )
)]
pub async fn delete_copy(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<i64>,
) -> ApiResult<Json<GameDetail>> {
    let game_id = games::delete_copy(&state.db, id).await?;
    Ok(Json(games::get(&state.db, game_id).await?))
}

#[utoipa::path(
    put, path = "/releases/{id}", tag = "games", operation_id = "updateRelease",
    params(("id" = i64, Path)),
    request_body = ReleaseInput,
    responses(
        (status = OK, description = "The release's game", body = GameDetail),
        (status = NOT_FOUND, body = ErrorBody),
        (status = CONFLICT, description = "`release_exists`: same platform, region and product code", body = ErrorBody),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody),
    )
)]
pub async fn update_release(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<i64>,
    ApiJson(input): ApiJson<ReleaseInput>,
) -> ApiResult<Json<GameDetail>> {
    let game_id = games::update_release(&state.db, id, &input.validate()?).await?;
    Ok(Json(games::get(&state.db, game_id).await?))
}

#[utoipa::path(
    delete, path = "/releases/{id}", tag = "games", operation_id = "deleteRelease",
    params(("id" = i64, Path)),
    responses(
        (status = OK, description = "The release's game; the release's copies are deleted too", body = GameDetail),
        (status = NOT_FOUND, body = ErrorBody),
    )
)]
pub async fn delete_release(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<i64>,
) -> ApiResult<Json<GameDetail>> {
    let game_id = games::delete_release(&state.db, id).await?;
    covers::keep_only(&state.config.covers_dir(), Owner::Release(id), None).await;
    Ok(Json(games::get(&state.db, game_id).await?))
}

#[utoipa::path(
    get, path = "/game-modes", tag = "games", operation_id = "listGameModes",
    responses((status = OK, body = Vec<GameMode>))
)]
pub async fn game_modes(State(state): State<AppState>) -> ApiResult<Json<Vec<GameMode>>> {
    Ok(Json(
        sqlx::query_as!(
            GameMode,
            r#"SELECT id AS "id!", name, slug FROM game_modes ORDER BY id"#
        )
        .fetch_all(&state.db)
        .await?,
    ))
}

#[utoipa::path(
    get, path = "/genres", tag = "games", operation_id = "listGenres",
    responses((status = OK, body = Vec<Genre>))
)]
pub async fn genres(State(state): State<AppState>) -> ApiResult<Json<Vec<Genre>>> {
    Ok(Json(
        sqlx::query_as!(
            Genre,
            r#"SELECT id AS "id!", name FROM genres ORDER BY name"#
        )
        .fetch_all(&state.db)
        .await?,
    ))
}
