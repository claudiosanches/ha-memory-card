//! The library list and its filters.
//!
//! Values inside one filter are OR, filters are AND, and every copy-level
//! filter applies to the same copy.

use serde::{Deserialize, Deserializer, Serialize};
use sqlx::types::Json;
use sqlx::{QueryBuilder, Sqlite, SqlitePool};
use utoipa::{IntoParams, ToSchema};

use super::play_statuses;
use crate::domain::{Condition, CopyStatus, Format, Region};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Ownership {
    /// Games with at least one owned copy.
    #[default]
    Owned,
    /// Every game, with or without copies.
    Any,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum IgdbLink {
    Linked,
    Unlinked,
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
    /// Play statuses, plus `none` for games without one.
    #[serde(default, deserialize_with = "comma_list")]
    #[param(value_type = Option<String>)]
    pub play_status: Vec<StatusFilter>,
    pub format: Option<Format>,
    /// Local (same screen) players, release value first, then the game's.
    pub min_local_players: Option<i64>,
    #[param(inline)]
    pub ownership: Option<Ownership>,
    /// Whether the game is linked to IGDB.
    #[param(inline)]
    pub igdb: Option<IgdbLink>,
    #[param(inline)]
    pub sort: Option<LibrarySort>,
}

/// A play status key, or `none` for games without one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatusFilter {
    Set(String),
    Unset,
}

impl<'de> Deserialize<'de> for StatusFilter {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        if raw == play_statuses::NONE {
            return Ok(Self::Unset);
        }
        let valid = !raw.is_empty()
            && raw
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
        if valid {
            Ok(Self::Set(raw))
        } else {
            Err(serde::de::Error::custom("invalid play status key"))
        }
    }
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
    /// Relative to the covers URL.
    pub cover_path: Option<String>,
    pub play_status: Option<String>,
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
    cover_path: Option<String>,
    play_status: Option<String>,
    last_played_at: Option<String>,
    owned_copies: i64,
    platforms: Json<Vec<PlatformCount>>,
}

/// Builds the library query. Public so tests can inspect its query plan.
pub fn query(filter: &LibraryFilter) -> QueryBuilder<Sqlite> {
    let mut qb = QueryBuilder::new(
        "SELECT g.id, g.title, g.igdb_id, g.cover_path, g.play_status, g.last_played_at,
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

    push_filters(&mut qb, filter, DeviceMatch::Assigned);

    qb.push(match filter.sort.unwrap_or_default() {
        LibrarySort::Title => " ORDER BY g.sort_title, g.id",
        LibrarySort::Added => " ORDER BY g.created_at DESC, g.id DESC",
        LibrarySort::Played => {
            " ORDER BY g.last_played_at IS NULL, g.last_played_at DESC, g.sort_title"
        }
    });
    qb
}

/// How the device filter matches a copy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceMatch {
    /// The copy is on the device.
    Assigned,
    /// The copy is on the device, or has no device and the device plays its platform.
    Playable,
}

/// Whether any filter applies to copies rather than games.
pub fn has_copy_filters(filter: &LibraryFilter) -> bool {
    filter.ownership.unwrap_or_default() == Ownership::Owned
        || !filter.platform.is_empty()
        || !filter.device.is_empty()
        || !filter.store.is_empty()
        || !filter.region.is_empty()
        || filter.format.is_some()
        || filter.min_local_players.is_some_and(|n| n > 1)
}

/// Conditions on games `g` after a `WHERE`.
pub fn push_filters(qb: &mut QueryBuilder<Sqlite>, filter: &LibraryFilter, device: DeviceMatch) {
    if has_copy_filters(filter) {
        qb.push(
            " AND EXISTS (SELECT 1 FROM copies c JOIN releases r ON r.id = c.release_id
               WHERE r.game_id = g.id",
        );
        push_copy_filters(qb, filter, device);
        qb.push(")");
    }
    push_game_filters(qb, filter);
}

/// Conditions on games `g` that do not look at copies.
pub fn push_game_filters(qb: &mut QueryBuilder<Sqlite>, filter: &LibraryFilter) {
    if !filter.genre.is_empty() {
        qb.push(" AND EXISTS (SELECT 1 FROM game_genres gg WHERE gg.game_id = g.id");
        push_in(qb, "gg.genre_id", &filter.genre);
        qb.push(")");
    }
    if !filter.game_mode.is_empty() {
        qb.push(" AND EXISTS (SELECT 1 FROM game_game_modes gm WHERE gm.game_id = g.id");
        push_in(qb, "gm.game_mode_id", &filter.game_mode);
        qb.push(")");
    }
    push_play_status(qb, &filter.play_status);
    match filter.igdb {
        Some(IgdbLink::Linked) => qb.push(" AND g.igdb_id IS NOT NULL"),
        Some(IgdbLink::Unlinked) => qb.push(" AND g.igdb_id IS NULL"),
        None => &mut *qb,
    };

    if let Some(text) = filter.q.as_deref().map(str::trim).filter(|t| !t.is_empty()) {
        qb.push(" AND instr(lower(g.title), lower(")
            .push_bind(text.to_owned())
            .push(")) > 0");
    }
}

/// Conditions on copies `c`, their releases `r` and games `g`.
pub fn push_copy_filters(
    qb: &mut QueryBuilder<Sqlite>,
    filter: &LibraryFilter,
    device: DeviceMatch,
) {
    if filter.ownership.unwrap_or_default() == Ownership::Owned {
        qb.push(" AND c.status = 'owned'");
    }
    push_in(qb, "r.platform_id", &filter.platform);
    match device {
        DeviceMatch::Assigned => push_in(qb, "c.device_id", &filter.device),
        DeviceMatch::Playable if !filter.device.is_empty() => {
            qb.push(" AND (c.device_id IN ");
            push_list(qb, &filter.device);
            qb.push(
                " OR (c.device_id IS NULL AND r.platform_id IN
                   (SELECT platform_id FROM device_platforms WHERE device_id IN ",
            );
            push_list(qb, &filter.device);
            qb.push(")))");
        }
        DeviceMatch::Playable => {}
    }
    push_in(qb, "c.store_id", &filter.store);
    push_in(qb, "r.region", &filter.region);
    if let Some(format) = filter.format {
        qb.push(" AND c.format = ").push_bind(format);
    }
    if let Some(players) = filter.min_local_players.filter(|n| *n > 1) {
        qb.push(" AND COALESCE(r.local_max_players, g.local_max_players) >= ")
            .push_bind(players);
    }
}

fn push_play_status(qb: &mut QueryBuilder<Sqlite>, filter: &[StatusFilter]) {
    let statuses: Vec<String> = filter
        .iter()
        .filter_map(|s| match s {
            StatusFilter::Set(status) => Some(status.clone()),
            StatusFilter::Unset => None,
        })
        .collect();
    let unset = filter.contains(&StatusFilter::Unset);
    match (statuses.is_empty(), unset) {
        (true, false) => {}
        (true, true) => {
            qb.push(" AND g.play_status IS NULL");
        }
        (false, false) => push_in(qb, "g.play_status", &statuses),
        (false, true) => {
            qb.push(" AND (g.play_status IS NULL OR g.play_status IN ");
            push_list(qb, &statuses);
            qb.push(")");
        }
    }
}

fn push_in<T>(qb: &mut QueryBuilder<Sqlite>, column: &'static str, values: &[T])
where
    T: Clone + for<'q> sqlx::Encode<'q, Sqlite> + sqlx::Type<Sqlite> + 'static,
{
    if values.is_empty() {
        return;
    }
    qb.push(" AND ").push(column).push(" IN ");
    push_list(qb, values);
}

/// Pushes `(?, ?, ...)`.
fn push_list<T>(qb: &mut QueryBuilder<Sqlite>, values: &[T])
where
    T: Clone + for<'q> sqlx::Encode<'q, Sqlite> + sqlx::Type<Sqlite> + 'static,
{
    qb.push("(");
    let mut list = qb.separated(", ");
    for value in values {
        list.push_bind(value.clone());
    }
    list.push_unseparated(")");
}

/// One copy in the copies view, with its release and game.
#[derive(Debug, Serialize, ToSchema, sqlx::FromRow)]
pub struct CopyRow {
    pub copy_id: i64,
    pub game_id: i64,
    pub title: String,
    /// The release cover, else the game's. Relative to the covers URL.
    pub cover_path: Option<String>,
    pub play_status: Option<String>,
    pub release_id: i64,
    pub platform_id: i64,
    pub region: Region,
    pub release_title: Option<String>,
    pub product_code: Option<String>,
    pub status: CopyStatus,
    pub format: Format,
    pub device_id: Option<i64>,
    pub store_id: Option<i64>,
    pub condition: Option<Condition>,
    pub notes: Option<String>,
}

/// Copies matching the filters: copy filters apply to each copy, the others to its game.
pub fn copies_query(filter: &LibraryFilter) -> QueryBuilder<Sqlite> {
    let mut qb = QueryBuilder::new(
        "SELECT c.id AS copy_id, g.id AS game_id, g.title,
                COALESCE(r.cover_path, g.cover_path) AS cover_path, g.play_status,
                r.id AS release_id, r.platform_id, r.region, r.title AS release_title,
                r.product_code, c.status, c.format, c.device_id, c.store_id, c.condition, c.notes
         FROM copies c
         JOIN releases r ON r.id = c.release_id
         JOIN games g ON g.id = r.game_id
         JOIN platforms p ON p.id = r.platform_id
         WHERE 1 = 1",
    );
    push_copy_filters(&mut qb, filter, DeviceMatch::Assigned);
    push_game_filters(&mut qb, filter);
    qb.push(match filter.sort.unwrap_or_default() {
        LibrarySort::Title => " ORDER BY g.sort_title, g.id, p.sort_order, r.region, c.id",
        LibrarySort::Added => " ORDER BY c.created_at DESC, c.id DESC",
        LibrarySort::Played => {
            " ORDER BY g.last_played_at IS NULL, g.last_played_at DESC, g.sort_title, g.id,
                       p.sort_order, c.id"
        }
    });
    qb
}

pub async fn list_copies(pool: &SqlitePool, filter: &LibraryFilter) -> sqlx::Result<Vec<CopyRow>> {
    copies_query(filter).build_query_as().fetch_all(pool).await
}

pub async fn list(pool: &SqlitePool, filter: &LibraryFilter) -> sqlx::Result<Vec<GameSummary>> {
    let rows: Vec<SummaryRow> = query(filter).build_query_as().fetch_all(pool).await?;
    Ok(rows
        .into_iter()
        .map(|row| GameSummary {
            id: row.id,
            title: row.title,
            igdb_id: row.igdb_id,
            cover_path: row.cover_path,
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
        assert_eq!(
            filter.play_status,
            [StatusFilter::Set("backlog".to_owned())]
        );
        assert_eq!(filter.format, Some(Format::Digital));
        assert!(serde_urlencoded::from_str::<LibraryFilter>("region=mars").is_err());
        let filter: LibraryFilter = serde_urlencoded::from_str("play_status=none,paused").unwrap();
        assert_eq!(
            filter.play_status,
            [StatusFilter::Unset, StatusFilter::Set("paused".to_owned())]
        );
        assert!(serde_urlencoded::from_str::<LibraryFilter>("play_status=Nope!").is_err());
    }
}
