//! The library list and its filters.
//!
//! Values inside one filter are OR, filters are AND, and every copy-level
//! filter applies to the same copy.

use serde::{Deserialize, Deserializer, Serialize};
use sqlx::types::Json;
use sqlx::{Sqlite, SqlitePool};
use utoipa::{IntoParams, ToSchema};

use super::play_statuses;
use super::slugs::Table;
use super::sql::Sql;
use crate::domain::{Condition, CopyStatus, Format, Region};
use crate::slug::Slug;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Ownership {
    /// Games with at least one owned copy.
    #[default]
    Owned,
    /// Games with at least one wanted copy: the wishlist.
    Wanted,
    /// Every game, with or without copies.
    Any,
}

impl Ownership {
    /// The copy status this ownership looks at, if any.
    fn status(self) -> Option<&'static str> {
        match self {
            Self::Owned => Some("owned"),
            Self::Wanted => Some("wanted"),
            Self::Any => None,
        }
    }
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
    /// Text in the title, a release title or another name, ignoring case and accents.
    pub q: Option<String>,
    /// Platform slugs.
    #[serde(default, deserialize_with = "comma_list")]
    #[param(value_type = Option<String>)]
    pub platform: Vec<Slug>,
    /// Device slugs.
    #[serde(default, deserialize_with = "comma_list")]
    #[param(value_type = Option<String>)]
    pub device: Vec<Slug>,
    /// Store slugs.
    #[serde(default, deserialize_with = "comma_list")]
    #[param(value_type = Option<String>)]
    pub store: Vec<Slug>,
    /// Region codes.
    #[serde(default, deserialize_with = "comma_list")]
    #[param(value_type = Option<String>)]
    pub region: Vec<Region>,
    /// Genre slugs.
    #[serde(default, deserialize_with = "comma_list")]
    #[param(value_type = Option<String>)]
    pub genre: Vec<Slug>,
    /// Game mode slugs.
    #[serde(default, deserialize_with = "comma_list")]
    #[param(value_type = Option<String>)]
    pub game_mode: Vec<Slug>,
    /// Label slugs.
    #[serde(default, deserialize_with = "comma_list")]
    #[param(value_type = Option<String>)]
    pub label: Vec<Slug>,
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
    /// Games matching `q`, filled by [`resolve_search`].
    #[serde(skip)]
    #[param(ignore)]
    pub matches: Option<Vec<i64>>,
    /// Whose play the status filter, the played sort and the play columns use.
    #[serde(skip)]
    #[param(ignore)]
    pub person: Option<String>,
}

/// Finds the games `q` matches, so the query can filter on their ids.
pub async fn resolve_search(pool: &SqlitePool, filter: &mut LibraryFilter) -> sqlx::Result<()> {
    let Some(text) = filter.q.as_deref().map(str::trim).filter(|t| !t.is_empty()) else {
        return Ok(());
    };
    let needle = crate::domain::search_key(text);
    let names: Vec<(i64, String)> = sqlx::query_as(
        "SELECT id, title FROM games
         UNION ALL SELECT game_id, title FROM releases WHERE title IS NOT NULL
         UNION ALL SELECT game_id, name FROM game_names",
    )
    .fetch_all(pool)
    .await?;
    let mut ids: Vec<i64> = names
        .into_iter()
        .filter(|(_, name)| crate::domain::search_key(name).contains(&needle))
        .map(|(id, _)| id)
        .collect();
    ids.sort_unstable();
    ids.dedup();
    filter.matches = Some(ids);
    Ok(())
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

/// Most values one filter takes.
pub const FILTER_MAX: usize = 100;

fn comma_list<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: serde::de::DeserializeOwned,
{
    let raw: Option<String> = Option::deserialize(deserializer)?;
    let list: Vec<T> = raw
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(|part| {
            serde_json::from_value(serde_json::Value::String(part.to_owned()))
                .map_err(serde::de::Error::custom)
        })
        .collect::<Result<_, _>>()?;
    capped(list)
}

/// A filter list sent as a JSON array.
pub fn filter_list<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    capped(Vec::deserialize(deserializer)?)
}

fn capped<T, E: serde::de::Error>(list: Vec<T>) -> Result<Vec<T>, E> {
    if list.len() > FILTER_MAX {
        Err(E::custom(format!("more than {FILTER_MAX} values")))
    } else {
        Ok(list)
    }
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
    pub wanted_copies: i64,
    /// Platforms with owned copies, or wanted ones on the wishlist, in platform list order.
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
    wanted_copies: i64,
    platforms: Json<Vec<PlatformCount>>,
}

/// Builds the library query. Public so tests can inspect its query plan.
pub fn query(filter: &LibraryFilter) -> Sql {
    let mut qb = Sql::new("SELECT g.id, g.title, g.igdb_id, g.cover_path, ");
    push_play_columns(&mut qb, filter);
    qb.push(
        ",
                (SELECT COUNT(*) FROM copies c JOIN releases r ON r.id = c.release_id
                 WHERE r.game_id = g.id AND c.status = 'owned') AS owned_copies,
                (SELECT COUNT(*) FROM copies c JOIN releases r ON r.id = c.release_id
                 WHERE r.game_id = g.id AND c.status = 'wanted') AS wanted_copies,
                (SELECT json_group_array(json_object('platform_id', platform_id, 'copies', n)) FROM (
                     SELECT r.platform_id, COUNT(*) AS n FROM releases r
                     JOIN copies c ON c.release_id = r.id AND c.status = ",
    );
    let shown = match filter.ownership.unwrap_or_default() {
        Ownership::Wanted => "wanted",
        Ownership::Owned | Ownership::Any => "owned",
    };
    qb.push_bind(shown).push(
        "  JOIN platforms p ON p.id = r.platform_id
                     WHERE r.game_id = g.id
                     GROUP BY r.platform_id ORDER BY MIN(p.sort_order))) AS platforms
         FROM games g WHERE 1 = 1",
    );

    push_filters(&mut qb, filter, DeviceMatch::Assigned);

    qb.push(match filter.sort.unwrap_or_default() {
        LibrarySort::Title => " ORDER BY g.sort_title, g.id",
        LibrarySort::Added => " ORDER BY g.created_at DESC, g.id DESC",
        LibrarySort::Played => " ORDER BY last_played_at DESC NULLS LAST, g.sort_title",
    });
    qb
}

/// The person's `play_status` and `last_played_at` for game `g`.
fn push_play_columns(qb: &mut Sql, filter: &LibraryFilter) {
    qb.push("(SELECT play_status FROM play_progress WHERE user_id = ")
        .push_bind(filter.person.clone())
        .push(" AND game_id = g.id) AS play_status, (SELECT MAX(played_on) FROM play_sessions WHERE user_id = ")
        .push_bind(filter.person.clone())
        .push(" AND game_id = g.id) AS last_played_at");
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
    filter.ownership.unwrap_or_default() != Ownership::Any
        || !filter.platform.is_empty()
        || !filter.device.is_empty()
        || !filter.store.is_empty()
        || !filter.region.is_empty()
        || filter.format.is_some()
        || filter.min_local_players.is_some_and(|n| n > 1)
}

/// Conditions on games `g` after a `WHERE`.
pub fn push_filters(qb: &mut Sql, filter: &LibraryFilter, device: DeviceMatch) {
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
pub fn push_game_filters(qb: &mut Sql, filter: &LibraryFilter) {
    if !filter.genre.is_empty() {
        qb.push(" AND EXISTS (SELECT 1 FROM game_genres gg WHERE gg.game_id = g.id");
        push_slugs(qb, "gg.genre_id", Table::Genres, &filter.genre);
        qb.push(")");
    }
    if !filter.game_mode.is_empty() {
        qb.push(" AND EXISTS (SELECT 1 FROM game_game_modes gm WHERE gm.game_id = g.id");
        push_slugs(qb, "gm.game_mode_id", Table::GameModes, &filter.game_mode);
        qb.push(")");
    }
    if !filter.label.is_empty() {
        qb.push(" AND EXISTS (SELECT 1 FROM game_labels gl WHERE gl.game_id = g.id");
        push_slugs(qb, "gl.label_id", Table::Labels, &filter.label);
        qb.push(")");
    }
    push_play_status(qb, filter);
    match filter.igdb {
        Some(IgdbLink::Linked) => qb.push(" AND g.igdb_id IS NOT NULL"),
        Some(IgdbLink::Unlinked) => qb.push(" AND g.igdb_id IS NULL"),
        None => &mut *qb,
    };

    if let Some(ids) = &filter.matches {
        let ids = serde_json::to_string(ids).expect("a list of numbers serializes");
        qb.push(" AND g.id IN (SELECT value FROM json_each(")
            .push_bind(ids)
            .push("))");
    } else if let Some(text) = filter.q.as_deref().map(str::trim).filter(|t| !t.is_empty()) {
        qb.push(" AND instr(lower(g.title), lower(")
            .push_bind(text.to_owned())
            .push(")) > 0");
    }
}

/// Conditions on copies `c`, their releases `r` and games `g`.
pub fn push_copy_filters(qb: &mut Sql, filter: &LibraryFilter, device: DeviceMatch) {
    if let Some(status) = filter.ownership.unwrap_or_default().status() {
        qb.push(" AND c.status = ").push_bind(status);
    }
    push_slugs(qb, "r.platform_id", Table::Platforms, &filter.platform);
    match device {
        DeviceMatch::Assigned => push_slugs(qb, "c.device_id", Table::Devices, &filter.device),
        DeviceMatch::Playable if !filter.device.is_empty() => {
            qb.push(" AND (c.device_id IN ");
            push_ids(qb, Table::Devices, &filter.device);
            qb.push(
                " OR (c.device_id IS NULL AND r.platform_id IN
                   (SELECT platform_id FROM device_platforms WHERE device_id IN ",
            );
            push_ids(qb, Table::Devices, &filter.device);
            qb.push(")))");
        }
        DeviceMatch::Playable => {}
    }
    push_slugs(qb, "c.store_id", Table::Stores, &filter.store);
    push_in(qb, "r.region", &filter.region);
    if let Some(format) = filter.format {
        qb.push(" AND c.format = ").push_bind(format);
    }
    if let Some(players) = filter.min_local_players.filter(|n| *n > 1) {
        qb.push(" AND COALESCE(r.local_max_players, g.local_max_players) >= ")
            .push_bind(players);
    }
}

/// The person's status for game `g` in `filter.play_status`.
fn push_play_status(qb: &mut Sql, filter: &LibraryFilter) {
    let statuses: Vec<String> = filter
        .play_status
        .iter()
        .filter_map(|s| match s {
            StatusFilter::Set(status) => Some(status.clone()),
            StatusFilter::Unset => None,
        })
        .collect();
    let unset = filter.play_status.contains(&StatusFilter::Unset);
    if statuses.is_empty() && !unset {
        return;
    }
    qb.push(" AND COALESCE((SELECT play_status FROM play_progress WHERE user_id = ")
        .push_bind(filter.person.clone())
        .push(" AND game_id = g.id), ")
        .push_bind(play_statuses::NONE)
        .push(") IN ");
    let mut wanted = statuses;
    if unset {
        wanted.push(play_statuses::NONE.to_owned());
    }
    qb.push_list(&wanted);
}

fn push_in<T>(qb: &mut Sql, column: &'static str, values: &[T])
where
    T: Clone + for<'q> sqlx::Encode<'q, Sqlite> + sqlx::Type<Sqlite> + 'static,
{
    if values.is_empty() {
        return;
    }
    qb.push(" AND ").push(column).push(" IN ").push_list(values);
}

/// ` AND column IN (ids of the rows of table with these slugs)`.
fn push_slugs(qb: &mut Sql, column: &'static str, table: Table, slugs: &[Slug]) {
    if slugs.is_empty() {
        return;
    }
    qb.push(" AND ").push(column).push(" IN ");
    push_ids(qb, table, slugs);
}

/// Pushes `(SELECT id FROM table WHERE slug IN (?, ...))`.
fn push_ids(qb: &mut Sql, table: Table, slugs: &[Slug]) {
    qb.push("(SELECT id FROM ")
        .push(table.name())
        .push(" WHERE slug IN ")
        .push_list(slugs)
        .push(")");
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
    pub edition: Option<String>,
    pub purchased_on: Option<String>,
    /// Decimal text; see `CopyDetail`.
    #[sqlx(skip)]
    pub price: Option<String>,
    pub currency: Option<String>,
    #[serde(skip)]
    price_minor: Option<i64>,
    /// The bundle copy this copy came in.
    pub bundle_copy_id: Option<i64>,
    pub barcode: Option<String>,
    /// The person's latest session of the game.
    pub last_played_at: Option<String>,
}

/// Copies matching the filters: copy filters apply to each copy, the others to its game.
pub fn copies_query(filter: &LibraryFilter) -> Sql {
    let mut qb = Sql::new(
        "SELECT c.id AS copy_id, g.id AS game_id, g.title,
                COALESCE(r.cover_path, g.cover_path) AS cover_path,
                r.id AS release_id, r.platform_id, r.region, r.title AS release_title,
                r.product_code, c.status, c.format, c.device_id, c.store_id, c.condition, c.notes,
                c.edition, c.purchased_on, c.price AS price_minor, c.currency,
                c.bundle_id AS bundle_copy_id, c.barcode, ",
    );
    push_play_columns(&mut qb, filter);
    qb.push(
        "
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
        LibrarySort::Added => " ORDER BY COALESCE(c.owned_at, c.created_at) DESC, c.id DESC",
        LibrarySort::Played => {
            " ORDER BY last_played_at DESC NULLS LAST, g.sort_title, g.id,
                       p.sort_order, c.id"
        }
    });
    qb
}

pub async fn list_copies(pool: &SqlitePool, filter: &LibraryFilter) -> sqlx::Result<Vec<CopyRow>> {
    let mut filter = filter.clone();
    resolve_search(pool, &mut filter).await?;
    let mut rows: Vec<CopyRow> = copies_query(&filter)
        .build_query_as()
        .fetch_all(pool)
        .await?;
    for row in &mut rows {
        row.price = row
            .price_minor
            .zip(row.currency.as_deref())
            .map(|(minor, currency)| crate::money::format(minor, currency));
    }
    Ok(rows)
}

pub async fn list(pool: &SqlitePool, filter: &LibraryFilter) -> sqlx::Result<Vec<GameSummary>> {
    let mut filter = filter.clone();
    resolve_search(pool, &mut filter).await?;
    let rows: Vec<SummaryRow> = query(&filter).build_query_as().fetch_all(pool).await?;
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
            wanted_copies: row.wanted_copies,
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
            "platform=switch,%20PS5,&store=itch-io&region=eu,jp&play_status=backlog&format=digital",
        )
        .unwrap();
        assert_eq!(filter.platform, ["switch".into(), "ps5".into()]);
        assert_eq!(filter.store, ["itch-io".into()]);
        assert_eq!(filter.region, [Region::Eu, Region::Jp]);
        assert_eq!(
            filter.play_status,
            [StatusFilter::Set("backlog".to_owned())]
        );
        assert_eq!(filter.format, Some(Format::Digital));
        assert!(serde_urlencoded::from_str::<LibraryFilter>("region=mars").is_err());
        assert!(serde_urlencoded::from_str::<LibraryFilter>("store=itch.io").is_err());
        let filter: LibraryFilter = serde_urlencoded::from_str("play_status=none,paused").unwrap();
        assert_eq!(
            filter.play_status,
            [StatusFilter::Unset, StatusFilter::Set("paused".to_owned())]
        );
        assert!(serde_urlencoded::from_str::<LibraryFilter>("play_status=Nope!").is_err());
    }
}
