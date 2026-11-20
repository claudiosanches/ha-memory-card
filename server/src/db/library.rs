//! The library list and its filters.
//!
//! Values inside one filter are OR, filters are AND, and every copy-level
//! filter applies to the same copy.

use serde::{Deserialize, Deserializer, Serialize};
use sqlx::types::Json;
use sqlx::{QueryBuilder, Sqlite, SqlitePool};
use utoipa::{IntoParams, ToSchema};

use crate::domain::{Format, PlayStatus, Region};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Ownership {
    /// Games with at least one owned copy.
    #[default]
    Owned,
    /// Every game, with or without copies.
    Any,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum LibrarySort {
    #[default]
    Title,
    Added,
    Played,
}

/// Library filters. List filters take comma-separated values.
#[derive(Debug, Clone, Default, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct LibraryFilter {
    /// Text in the title.
    pub q: Option<String>,
    /// Platform ids.
    #[serde(default, deserialize_with = "comma_list")]
    #[param(value_type = Option<String>)]
    pub platform: Vec<i64>,
    /// Device ids.
    #[serde(default, deserialize_with = "comma_list")]
    #[param(value_type = Option<String>)]
    pub device: Vec<i64>,
    /// Store ids.
    #[serde(default, deserialize_with = "comma_list")]
    #[param(value_type = Option<String>)]
    pub store: Vec<i64>,
    /// Region codes.
    #[serde(default, deserialize_with = "comma_list")]
    #[param(value_type = Option<String>)]
    pub region: Vec<Region>,
    /// Genre ids.
    #[serde(default, deserialize_with = "comma_list")]
    #[param(value_type = Option<String>)]
    pub genre: Vec<i64>,
    /// Game mode ids.
    #[serde(default, deserialize_with = "comma_list")]
    #[param(value_type = Option<String>)]
    pub game_mode: Vec<i64>,
    /// Play statuses.
    #[serde(default, deserialize_with = "comma_list")]
    #[param(value_type = Option<String>)]
    pub play_status: Vec<PlayStatus>,
    pub format: Option<Format>,
    /// Local (same screen) players, release value first, then the game's.
    pub min_local_players: Option<i64>,
    #[param(inline)]
    pub ownership: Option<Ownership>,
    #[param(inline)]
    pub sort: Option<LibrarySort>,
}

fn comma_list<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: serde::de::DeserializeOwned,
{
    let raw: Option<String> = Option::deserialize(deserializer)?;
    raw.unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(|part| {
            serde_json::from_value(serde_json::Value::String(part.to_owned()))
                .or_else(|_| serde_json::from_str(part))
                .map_err(serde::de::Error::custom)
        })
        .collect()
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PlatformCount {
    pub platform_id: i64,
    pub copies: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct GameSummary {
    pub id: i64,
    pub title: String,
    pub igdb_id: Option<i64>,
    pub play_status: Option<PlayStatus>,
    pub last_played_at: Option<String>,
    pub owned_copies: i64,
    /// Platforms with owned copies, in platform list order.
    pub platforms: Vec<PlatformCount>,
}

#[derive(sqlx::FromRow)]
struct SummaryRow {
    id: i64,
    title: String,
    igdb_id: Option<i64>,
    play_status: Option<PlayStatus>,
    last_played_at: Option<String>,
    owned_copies: i64,
    platforms: Json<Vec<PlatformCount>>,
}

/// Builds the library query. Public so tests can inspect its query plan.
pub fn query(filter: &LibraryFilter) -> QueryBuilder<Sqlite> {
    let mut qb = QueryBuilder::new(
        "SELECT g.id, g.title, g.igdb_id, g.play_status, g.last_played_at,
                (SELECT COUNT(*) FROM copies c JOIN releases r ON r.id = c.release_id
                 WHERE r.game_id = g.id AND c.status = 'owned') AS owned_copies,
                (SELECT json_group_array(json_object('platform_id', platform_id, 'copies', n)) FROM (
                     SELECT r.platform_id, COUNT(*) AS n FROM releases r
                     JOIN copies c ON c.release_id = r.id AND c.status = 'owned'
                     JOIN platforms p ON p.id = r.platform_id
                     WHERE r.game_id = g.id
                     GROUP BY r.platform_id ORDER BY MIN(p.sort_order))) AS platforms
         FROM games g WHERE 1 = 1",
    );

    let owned = filter.ownership.unwrap_or_default() == Ownership::Owned;
    let copy_filters = !filter.platform.is_empty()
        || !filter.device.is_empty()
        || !filter.store.is_empty()
        || !filter.region.is_empty()
        || filter.format.is_some()
        || filter.min_local_players.is_some_and(|n| n > 1);

    if owned || copy_filters {
        qb.push(
            " AND EXISTS (SELECT 1 FROM copies c JOIN releases r ON r.id = c.release_id
               WHERE r.game_id = g.id",
        );
        if owned {
            qb.push(" AND c.status = 'owned'");
        }
        push_in(&mut qb, "r.platform_id", &filter.platform);
        push_in(&mut qb, "c.device_id", &filter.device);
        push_in(&mut qb, "c.store_id", &filter.store);
        push_in(&mut qb, "r.region", &filter.region);
        if let Some(format) = filter.format {
            qb.push(" AND c.format = ").push_bind(format);
        }
        if let Some(players) = filter.min_local_players.filter(|n| *n > 1) {
            qb.push(" AND COALESCE(r.local_max_players, g.local_max_players) >= ")
                .push_bind(players);
        }
        qb.push(")");
    }

    if !filter.genre.is_empty() {
        qb.push(" AND EXISTS (SELECT 1 FROM game_genres gg WHERE gg.game_id = g.id");
        push_in(&mut qb, "gg.genre_id", &filter.genre);
        qb.push(")");
    }
    if !filter.game_mode.is_empty() {
        qb.push(" AND EXISTS (SELECT 1 FROM game_game_modes gm WHERE gm.game_id = g.id");
        push_in(&mut qb, "gm.game_mode_id", &filter.game_mode);
        qb.push(")");
    }
    push_in(&mut qb, "g.play_status", &filter.play_status);

    if let Some(text) = filter.q.as_deref().map(str::trim).filter(|t| !t.is_empty()) {
        qb.push(" AND instr(lower(g.title), lower(")
            .push_bind(text.to_owned())
            .push(")) > 0");
    }

    qb.push(match filter.sort.unwrap_or_default() {
        LibrarySort::Title => " ORDER BY g.sort_title, g.id",
        LibrarySort::Added => " ORDER BY g.created_at DESC, g.id DESC",
        LibrarySort::Played => {
            " ORDER BY g.last_played_at IS NULL, g.last_played_at DESC, g.sort_title"
        }
    });
    qb
}

fn push_in<T>(qb: &mut QueryBuilder<Sqlite>, column: &str, values: &[T])
where
    T: Clone + for<'q> sqlx::Encode<'q, Sqlite> + sqlx::Type<Sqlite> + 'static,
{
    if values.is_empty() {
        return;
    }
    qb.push(format_args!(" AND {column} IN ("));
    let mut list = qb.separated(", ");
    for value in values {
        list.push_bind(value.clone());
    }
    list.push_unseparated(")");
}

pub async fn list(pool: &SqlitePool, filter: &LibraryFilter) -> sqlx::Result<Vec<GameSummary>> {
    let rows: Vec<SummaryRow> = query(filter).build_query_as().fetch_all(pool).await?;
    Ok(rows
        .into_iter()
        .map(|row| GameSummary {
            id: row.id,
            title: row.title,
            igdb_id: row.igdb_id,
            play_status: row.play_status,
            last_played_at: row.last_played_at,
            owned_copies: row.owned_copies,
            platforms: row.platforms.0,
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_comma_lists() {
        let filter: LibraryFilter = serde_urlencoded::from_str(
            "platform=1,%202,&region=eu,jp&play_status=backlog&format=digital",
        )
        .unwrap();
        assert_eq!(filter.platform, [1, 2]);
        assert_eq!(filter.region, [Region::Eu, Region::Jp]);
        assert_eq!(filter.play_status, [PlayStatus::Backlog]);
        assert_eq!(filter.format, Some(Format::Digital));
        assert!(serde_urlencoded::from_str::<LibraryFilter>("region=mars").is_err());
    }
}
