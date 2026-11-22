//! Picks an owned game to play next.

use serde::{Deserialize, Serialize};
use sqlx::{QueryBuilder, SqlitePool};
use utoipa::ToSchema;

use super::games::{self, GameDetail};
use super::library::{self, DeviceMatch, LibraryFilter, Ownership, StatusFilter};
use super::play_statuses;
use crate::error::{ApiError, ApiResult};
use crate::slug::Slug;

/// Values inside one filter are OR, filters are AND, and the copy filters
/// apply to the same owned copy.
#[derive(Debug, Default, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct SuggestionFilter {
    /// Play status keys. Empty means every status that counts as backlog.
    #[serde(default, deserialize_with = "library::filter_list")]
    pub play_status: Vec<String>,
    /// Platform slugs.
    #[serde(default, deserialize_with = "library::filter_list")]
    #[schema(value_type = Vec<String>)]
    pub platform: Vec<Slug>,
    /// Device slugs: copies on these devices, or with no device on a platform
    /// one of them plays.
    #[serde(default, deserialize_with = "library::filter_list")]
    #[schema(value_type = Vec<String>)]
    pub device: Vec<Slug>,
    /// Genre slugs.
    #[serde(default, deserialize_with = "library::filter_list")]
    #[schema(value_type = Vec<String>)]
    pub genre: Vec<Slug>,
    /// Game mode slugs.
    #[serde(default, deserialize_with = "library::filter_list")]
    #[schema(value_type = Vec<String>)]
    pub game_mode: Vec<Slug>,
    /// Local (same screen) players, release value first, then the game's.
    #[serde(default)]
    pub min_local_players: Option<i64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct Suggestion {
    pub game: GameDetail,
    /// Owned copies of the game that match the filters.
    pub copy_ids: Vec<i64>,
}

/// Picks a random match, skipping games suggested in the last day unless
/// nothing else matches, then records the pick.
pub async fn suggest(pool: &SqlitePool, filter: SuggestionFilter) -> ApiResult<Suggestion> {
    let statuses = if filter.play_status.is_empty() {
        play_statuses::backlog_keys(pool).await?
    } else {
        filter.play_status
    };
    let filter = LibraryFilter {
        play_status: statuses.into_iter().map(StatusFilter::Set).collect(),
        platform: filter.platform,
        device: filter.device,
        genre: filter.genre,
        game_mode: filter.game_mode,
        min_local_players: filter.min_local_players,
        ownership: Some(Ownership::Owned),
        ..LibraryFilter::default()
    };

    let mut tx = pool.begin().await?;
    sqlx::query!(
        "DELETE FROM suggestions
         WHERE suggested_at < strftime('%Y-%m-%dT%H:%M:%fZ', 'now', '-30 days')"
    )
    .execute(&mut *tx)
    .await?;

    let mut pick = QueryBuilder::new(
        "SELECT id FROM (
             SELECT g.id, (SELECT MAX(suggested_at) FROM suggestions s WHERE s.game_id = g.id) AS last
             FROM games g WHERE 1 = 1",
    );
    library::push_filters(&mut pick, &filter, DeviceMatch::Playable);
    // Recent picks sort last, oldest first; the rest are equal and shuffled.
    pick.push(
        ") ORDER BY CASE WHEN last > strftime('%Y-%m-%dT%H:%M:%fZ', 'now', '-1 day') THEN last END,
                   random()
         LIMIT 1",
    );
    let game_id: i64 = pick
        .build_query_scalar()
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(ApiError::NoMatch("no_candidates"))?;

    sqlx::query!("INSERT INTO suggestions (game_id) VALUES (?)", game_id)
        .execute(&mut *tx)
        .await?;

    let mut copies = QueryBuilder::new(
        "SELECT c.id FROM copies c JOIN releases r ON r.id = c.release_id
         JOIN games g ON g.id = r.game_id WHERE g.id = ",
    );
    copies.push_bind(game_id);
    library::push_copy_filters(&mut copies, &filter, DeviceMatch::Playable);
    copies.push(" ORDER BY c.id");
    let copy_ids = copies.build_query_scalar().fetch_all(&mut *tx).await?;
    tx.commit().await?;

    Ok(Suggestion {
        game: games::get(pool, game_id).await?,
        copy_ids,
    })
}
