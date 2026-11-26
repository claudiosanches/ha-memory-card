use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use serde::Deserialize;
use time::Date;
use time::macros::format_description;
use utoipa::ToSchema;

use super::input::{NAME_MAX, NOTES_MAX, SUMMARY_MAX, TITLE_MAX, optional, required};
use crate::AppState;
use crate::access::Actor;
use crate::covers::{self, Owner};
use crate::db::games::{
    self, CopyFields, DroppedRelease, GameDetail, GameFields, GameMode, Genre, PlayerCounts, Price,
    ReleaseFields,
};
use crate::db::library::{self, CopyRow, GameSummary, LibraryFilter};
use crate::db::{labels, progress};
use crate::domain::{self, Condition, CopyStatus, Format, Region};
use crate::error::{ApiError, ApiJson, ApiPath, ApiResult, ErrorBody};
use crate::money;

const CODE_MAX: usize = 40;
const PLAYERS_MAX: i64 = 99;
const NAMES_MAX: usize = 50;

#[derive(Debug, Deserialize, ToSchema)]
pub struct NewGameInput {
    pub title: String,
    /// A play status key: the creator's own status for the game.
    #[serde(default)]
    pub play_status: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct GameInput {
    pub title: String,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub local_max_players: Option<i64>,
    #[serde(default)]
    pub online_max_players: Option<i64>,
    #[serde(default)]
    pub local_coop: Option<bool>,
    #[serde(default)]
    pub online_coop: Option<bool>,
    #[serde(default, deserialize_with = "super::input::id_list")]
    pub game_mode_ids: Vec<i64>,
    /// Other names to search by. Left out, the current ones stay.
    #[serde(default)]
    pub names: Option<Vec<String>>,
    /// Left out, the current labels stay. At most 50.
    #[serde(default)]
    pub label_ids: Option<Vec<i64>>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CopyInput {
    /// `wanted` puts the copy on the wishlist. Left out, a new copy is owned
    /// and an existing one keeps its status.
    #[serde(default)]
    pub status: Option<CopyStatus>,
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
    #[serde(default)]
    pub edition: Option<String>,
    /// `YYYY-MM-DD`.
    #[serde(default)]
    pub purchased_on: Option<String>,
    /// Decimal text such as `59.99`; needs `currency`.
    #[serde(default)]
    pub price: Option<String>,
    /// ISO 4217 code.
    #[serde(default)]
    pub currency: Option<String>,
    /// Decimal text: what an owned copy would sell for now; needs `estimated_currency`.
    #[serde(default)]
    pub estimated_value: Option<String>,
    /// ISO 4217 code.
    #[serde(default)]
    pub estimated_currency: Option<String>,
    /// UPC or EAN; spaces and hyphens are ignored.
    #[serde(default)]
    pub barcode: Option<String>,
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

pub(super) fn date(value: Option<&str>, field: &'static str) -> ApiResult<Option<String>> {
    let Some(value) = value.map(str::trim).filter(|v| !v.is_empty()) else {
        return Ok(None);
    };
    Date::parse(value, format_description!("[year]-[month]-[day]"))
        .map(|_| Some(value.to_owned()))
        .map_err(|_| ApiError::invalid("invalid_date", field))
}

/// Trims and deduplicates names, ignoring case; blanks are dropped.
fn names(values: &[String]) -> ApiResult<Vec<String>> {
    if values.len() > NAMES_MAX {
        return Err(ApiError::invalid("too_many", "names"));
    }
    let mut names: Vec<String> = Vec::new();
    for value in values {
        let Some(name) = optional(Some(value), "names", TITLE_MAX)? else {
            continue;
        };
        if !names
            .iter()
            .any(|n| n.to_lowercase() == name.to_lowercase())
        {
            names.push(name);
        }
    }
    Ok(names)
}

/// An amount needs a currency; a currency alone is dropped. `fields` names
/// the amount and currency fields in errors.
pub(super) fn price(
    amount: Option<&str>,
    currency: Option<&str>,
    fields: (&'static str, &'static str),
) -> ApiResult<Option<Price>> {
    let Some(amount) = amount.map(str::trim).filter(|a| !a.is_empty()) else {
        return Ok(None);
    };
    let currency = match currency.map(str::trim).filter(|c| !c.is_empty()) {
        None => return Err(ApiError::invalid("required", fields.1)),
        Some(code) => {
            money::currency(code).ok_or(ApiError::invalid("invalid_currency", fields.1))?
        }
    };
    let minor =
        money::parse(amount, &currency).ok_or(ApiError::invalid("invalid_price", fields.0))?;
    Ok(Some(Price { minor, currency }))
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
            players: players(
                self.local_max_players,
                self.online_max_players,
                self.local_coop,
                self.online_coop,
            )?,
            game_mode_ids: self.game_mode_ids.clone(),
            names: self.names.as_deref().map(names).transpose()?,
            label_ids: match &self.label_ids {
                Some(ids) if ids.len() > labels::PER_GAME_MAX => {
                    return Err(ApiError::invalid("too_many", "label_ids"));
                }
                ids => ids.clone(),
            },
        })
    }
}

impl CopyInput {
    fn validate(&self) -> ApiResult<CopyFields> {
        if self.condition.is_some() && self.format == Format::Digital {
            return Err(ApiError::invalid("condition_digital", "condition"));
        }
        Ok(CopyFields {
            status: self.status,
            platform_id: self.platform_id,
            region: self.region,
            product_code: optional(self.product_code.as_deref(), "product_code", CODE_MAX)?,
            format: self.format,
            device_id: self.device_id,
            store_id: self.store_id,
            condition: self.condition,
            notes: optional(self.notes.as_deref(), "notes", NOTES_MAX)?,
            edition: optional(self.edition.as_deref(), "edition", NAME_MAX)?,
            purchased_on: date(self.purchased_on.as_deref(), "purchased_on")?,
            price: price(
                self.price.as_deref(),
                self.currency.as_deref(),
                ("price", "currency"),
            )?,
            estimated_value: price(
                self.estimated_value.as_deref(),
                self.estimated_currency.as_deref(),
                ("estimated_value", "estimated_currency"),
            )?,
            barcode: match self
                .barcode
                .as_deref()
                .map(str::trim)
                .filter(|b| !b.is_empty())
            {
                None => None,
                Some(code) => Some(
                    domain::barcode(code).ok_or(ApiError::invalid("invalid_barcode", "barcode"))?,
                ),
            },
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
    Actor(by): Actor,
    filter: Result<Query<LibraryFilter>, axum::extract::rejection::QueryRejection>,
) -> ApiResult<Json<Vec<GameSummary>>> {
    let Query(mut filter) = filter.map_err(|_| ApiError::Invalid {
        code: "invalid_filter",
        field: None,
    })?;
    filter.person = Some(by);
    Ok(Json(library::list(&state.db, &filter).await?))
}

#[utoipa::path(
    get, path = "/copies", tag = "games", operation_id = "listCopies",
    params(LibraryFilter),
    responses(
        (status = OK, description = "Copies matching the library filters", body = Vec<CopyRow>),
        (status = UNPROCESSABLE_ENTITY, description = "`invalid_filter`", body = ErrorBody),
    )
)]
pub async fn list_copies(
    State(state): State<AppState>,
    Actor(by): Actor,
    filter: Result<Query<LibraryFilter>, axum::extract::rejection::QueryRejection>,
) -> ApiResult<Json<Vec<CopyRow>>> {
    let Query(mut filter) = filter.map_err(|_| ApiError::Invalid {
        code: "invalid_filter",
        field: None,
    })?;
    filter.person = Some(by);
    Ok(Json(library::list_copies(&state.db, &filter).await?))
}

#[utoipa::path(
    post, path = "/games", tag = "games", operation_id = "createGame",
    request_body = NewGameInput,
    responses((status = CREATED, body = GameDetail), (status = UNPROCESSABLE_ENTITY, body = ErrorBody))
)]
pub async fn create(
    State(state): State<AppState>,
    Actor(by): Actor,
    ApiJson(input): ApiJson<NewGameInput>,
) -> ApiResult<(StatusCode, Json<GameDetail>)> {
    let title = required(&input.title, "title", TITLE_MAX)?;
    let id = games::create(&state.db, &title, input.play_status.as_deref(), &by).await?;
    Ok((
        StatusCode::CREATED,
        Json(games::get(&state.db, id, &by).await?),
    ))
}

#[utoipa::path(
    get, path = "/games/{id}", tag = "games", operation_id = "getGame",
    params(("id" = i64, Path)),
    responses((status = OK, body = GameDetail), (status = NOT_FOUND, body = ErrorBody))
)]
pub async fn get(
    State(state): State<AppState>,
    Actor(by): Actor,
    ApiPath(id): ApiPath<i64>,
) -> ApiResult<Json<GameDetail>> {
    Ok(Json(games::get(&state.db, id, &by).await?))
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
    Actor(by): Actor,
    ApiPath(id): ApiPath<i64>,
    ApiJson(input): ApiJson<GameInput>,
) -> ApiResult<Json<GameDetail>> {
    games::update(&state.db, id, &input.validate()?, &by).await?;
    Ok(Json(games::get(&state.db, id, &by).await?))
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct PlayStatusInput {
    /// A play status key for the caller's own play; `null` clears it.
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
    Actor(by): Actor,
    ApiPath(id): ApiPath<i64>,
    ApiJson(input): ApiJson<PlayStatusInput>,
) -> ApiResult<Json<GameDetail>> {
    progress::set(&state.db, &by, id, input.play_status.as_deref()).await?;
    Ok(Json(games::get(&state.db, id, &by).await?))
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
    let cover = covers::lock(id).await;
    covers::keep_only(&state.config.covers_dir(), Owner::Game(id), None).await;
    drop(cover);
    drop_covers(&state, &releases).await;
    super::photos::prune(&state).await;
    Ok(StatusCode::NO_CONTENT)
}

/// Removes the cover files of releases the database no longer has.
pub(crate) async fn drop_covers(state: &AppState, releases: &[DroppedRelease]) {
    let dir = state.config.covers_dir();
    for release in releases {
        let _cover = covers::lock(release.game_id).await;
        covers::keep_only(&dir, Owner::Release(release.release_id), None).await;
    }
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
    Actor(by): Actor,
    ApiPath(id): ApiPath<i64>,
    ApiJson(input): ApiJson<CopyInput>,
) -> ApiResult<(StatusCode, Json<GameDetail>)> {
    games::add_copy(&state.db, id, &input.validate()?, &by).await?;
    Ok((
        StatusCode::CREATED,
        Json(games::get(&state.db, id, &by).await?),
    ))
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ContentInput {
    pub game_id: i64,
}

#[utoipa::path(
    post, path = "/copies/{id}/contents", tag = "games", operation_id = "addContent",
    params(("id" = i64, Path, description = "The bundle copy")),
    request_body = ContentInput,
    responses(
        (status = CREATED, description = "The bundle's game", body = GameDetail),
        (status = NOT_FOUND, body = ErrorBody),
        (status = CONFLICT, description = "`nested_bundle`: the copy is itself in a bundle; `already_in_bundle`", body = ErrorBody),
        (status = UNPROCESSABLE_ENTITY, description = "`same_game`, `unknown_game`", body = ErrorBody),
    )
)]
pub async fn add_content(
    State(state): State<AppState>,
    Actor(by): Actor,
    ApiPath(id): ApiPath<i64>,
    ApiJson(input): ApiJson<ContentInput>,
) -> ApiResult<(StatusCode, Json<GameDetail>)> {
    let game_id = games::add_content(&state.db, id, input.game_id, &by).await?;
    Ok((
        StatusCode::CREATED,
        Json(games::get(&state.db, game_id, &by).await?),
    ))
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
    Actor(by): Actor,
    ApiPath(id): ApiPath<i64>,
    ApiJson(input): ApiJson<CopyInput>,
) -> ApiResult<Json<GameDetail>> {
    let (game_id, dropped) = games::update_copy(&state.db, id, &input.validate()?, &by).await?;
    drop_covers(&state, &dropped).await;
    Ok(Json(games::get(&state.db, game_id, &by).await?))
}

#[utoipa::path(
    delete, path = "/copies/{id}", tag = "games", operation_id = "deleteCopy",
    params(("id" = i64, Path)),
    responses(
        (status = OK, description = "The copy's game; a release left with no copies is deleted", body = GameDetail),
        (status = NOT_FOUND, body = ErrorBody),
    )
)]
pub async fn delete_copy(
    State(state): State<AppState>,
    Actor(by): Actor,
    ApiPath(id): ApiPath<i64>,
) -> ApiResult<Json<GameDetail>> {
    let (game_id, dropped) = games::delete_copy(&state.db, id).await?;
    drop_covers(&state, &dropped).await;
    super::photos::prune(&state).await;
    Ok(Json(games::get(&state.db, game_id, &by).await?))
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
    Actor(by): Actor,
    ApiPath(id): ApiPath<i64>,
    ApiJson(input): ApiJson<ReleaseInput>,
) -> ApiResult<Json<GameDetail>> {
    let game_id = games::update_release(&state.db, id, &input.validate()?).await?;
    Ok(Json(games::get(&state.db, game_id, &by).await?))
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
    Actor(by): Actor,
    ApiPath(id): ApiPath<i64>,
) -> ApiResult<Json<GameDetail>> {
    let (game_id, mut dropped) = games::delete_release(&state.db, id).await?;
    dropped.push(DroppedRelease {
        game_id,
        release_id: id,
    });
    drop_covers(&state, &dropped).await;
    super::photos::prune(&state).await;
    Ok(Json(games::get(&state.db, game_id, &by).await?))
}

#[utoipa::path(
    get, path = "/game-modes", tag = "games", operation_id = "listGameModes",
    responses((status = OK, body = Vec<GameMode>))
)]
pub async fn game_modes(State(state): State<AppState>) -> ApiResult<Json<Vec<GameMode>>> {
    Ok(Json(games::game_modes(&state.db).await?))
}

#[utoipa::path(
    get, path = "/genres", tag = "games", operation_id = "listGenres",
    responses((status = OK, body = Vec<Genre>))
)]
pub async fn genres(State(state): State<AppState>) -> ApiResult<Json<Vec<Genre>>> {
    Ok(Json(games::genres(&state.db).await?))
}
