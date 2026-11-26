use std::collections::HashMap;

use serde::Serialize;
use sqlx::types::Json;
use sqlx::{SqliteConnection, SqlitePool};
use utoipa::ToSchema;

use super::found;
use super::photos::CopyPhoto;
use super::sessions::{self, PersonPlay, PlaySession};
use super::tags::CopyTag;
use super::users::Person;
use crate::covers::Owner;
use crate::domain::{Condition, CopySource, CopyStatus, Format, Region, sort_title};
use crate::error::{ApiError, ApiResult};
use crate::money;

#[derive(Debug, Serialize, ToSchema)]
pub struct CopyDetail {
    pub id: i64,
    pub status: CopyStatus,
    pub format: Format,
    pub device_id: Option<i64>,
    pub store_id: Option<i64>,
    pub condition: Option<Condition>,
    pub notes: Option<String>,
    pub edition: Option<String>,
    /// `YYYY-MM-DD`.
    pub purchased_on: Option<String>,
    /// Decimal text such as `59.99`: paid for an owned copy, expected for a wanted one.
    pub price: Option<String>,
    /// ISO 4217 code of `price`.
    pub currency: Option<String>,
    /// Decimal text: what an owned copy would sell for now.
    pub estimated_value: Option<String>,
    /// ISO 4217 code of `estimated_value`.
    pub estimated_currency: Option<String>,
    /// UPC or EAN from the box.
    pub barcode: Option<String>,
    pub source: CopySource,
    /// The bundle copy this copy came in. Its status, format, device, store,
    /// condition, platform and region follow that copy.
    pub bundle: Option<CopyLink>,
    /// Copies of other games that came in this one, by title.
    pub contents: Vec<CopyLink>,
    /// Oldest first.
    pub photos: Vec<CopyPhoto>,
    /// Home Assistant tags that start playing this copy, oldest first.
    pub tags: Vec<CopyTag>,
    pub created_at: String,
    pub created_by: Option<Person>,
    pub updated_by: Option<Person>,
}

/// A copy of another game.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct CopyLink {
    pub copy_id: i64,
    pub game_id: i64,
    pub title: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ReleaseDetail {
    pub id: i64,
    pub platform_id: i64,
    pub region: Region,
    pub title: Option<String>,
    pub product_code: Option<String>,
    pub release_date: Option<String>,
    /// This release's own cover, relative to the covers URL. The game's cover applies when null.
    pub cover_path: Option<String>,
    pub cover_uploaded: bool,
    pub local_max_players: Option<i64>,
    pub online_max_players: Option<i64>,
    pub local_coop: Option<bool>,
    pub online_coop: Option<bool>,
    pub copies: Vec<CopyDetail>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct GameDetail {
    pub id: i64,
    pub igdb_id: Option<i64>,
    pub title: String,
    pub summary: Option<String>,
    /// Relative to the covers URL.
    pub cover_path: Option<String>,
    pub cover_uploaded: bool,
    pub first_release_date: Option<String>,
    pub developers: Vec<String>,
    pub publishers: Vec<String>,
    /// The viewer's play status key.
    pub play_status: Option<String>,
    pub local_max_players: Option<i64>,
    pub online_max_players: Option<i64>,
    pub local_coop: Option<bool>,
    pub online_coop: Option<bool>,
    pub time_to_beat: TimeToBeat,
    pub game_mode_ids: Vec<i64>,
    pub genre_ids: Vec<i64>,
    /// By name.
    pub labels: Vec<super::labels::GameLabel>,
    pub created_at: String,
    pub created_by: Option<Person>,
    /// The last person to edit the game itself; copies record their own.
    pub updated_by: Option<Person>,
    /// In platform list order, then region.
    pub releases: Vec<ReleaseDetail>,
    /// The viewer's sessions, latest first; the first is when they last played.
    pub sessions: Vec<PlaySession>,
    /// Everyone else who keeps play for the game.
    pub household: Vec<PersonPlay>,
    /// Games in the collection that IGDB names as another version of this one, by title.
    pub versions: Vec<GameVersion>,
    /// Other names the game goes by, which the library search also matches.
    pub names: Vec<GameName>,
}

/// IGDB's average time to finish the game, in minutes.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
pub struct TimeToBeat {
    /// The main story.
    pub main: Option<i64>,
    /// The story with some extras.
    pub extras: Option<i64>,
    /// Everything.
    pub full: Option<i64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct GameName {
    pub name: String,
    /// From IGDB, which replaces them on a refresh, or added by hand.
    pub from_igdb: bool,
    /// IGDB's note, such as "Japanese title".
    pub comment: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct GameVersion {
    pub game_id: i64,
    pub title: String,
    pub cover_path: Option<String>,
    /// What that game is to this one.
    pub relation: VersionRelation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum VersionRelation {
    Remaster,
    Remake,
    Port,
    /// This game is a remaster, remake or port of it.
    Original,
    /// This game is an edition or other version of it.
    MainVersion,
    /// It is an edition or other version of this game.
    Edition,
}

impl VersionRelation {
    /// `outgoing` when this game's own IGDB data names the other one.
    fn from_link(kind: &str, outgoing: bool) -> Self {
        match (kind, outgoing) {
            ("remaster", true) => Self::Remaster,
            ("remake", true) => Self::Remake,
            ("port", true) => Self::Port,
            ("version_of", true) => Self::MainVersion,
            ("version_of", false) => Self::Edition,
            _ => Self::Original,
        }
    }
}

async fn versions(pool: &SqlitePool, id: i64) -> sqlx::Result<Vec<GameVersion>> {
    let rows = sqlx::query!(
        r#"SELECT h.id AS "id!", h.title, h.cover_path, l.kind, 1 AS "outgoing!: bool", h.sort_title
           FROM game_links l JOIN games h ON h.igdb_id = l.igdb_id
           WHERE l.game_id = ?1 AND h.id != ?1
           UNION ALL
           SELECT h.id, h.title, h.cover_path, l.kind, 0, h.sort_title
           FROM games g JOIN game_links l ON l.igdb_id = g.igdb_id JOIN games h ON h.id = l.game_id
           WHERE g.id = ?1 AND h.id != ?1
           ORDER BY 6, 1"#,
        id
    )
    .fetch_all(pool)
    .await?;
    let mut versions: Vec<GameVersion> = Vec::with_capacity(rows.len());
    for row in rows {
        if versions.iter().any(|v| v.game_id == row.id) {
            continue;
        }
        versions.push(GameVersion {
            game_id: row.id,
            title: row.title,
            cover_path: row.cover_path,
            relation: VersionRelation::from_link(&row.kind, row.outgoing),
        });
    }
    Ok(versions)
}

/// Player counts as stored on games and releases. `None` means unknown.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PlayerCounts {
    pub local_max_players: Option<i64>,
    pub online_max_players: Option<i64>,
    pub local_coop: Option<bool>,
    pub online_coop: Option<bool>,
}

pub struct GameFields {
    pub title: String,
    pub summary: Option<String>,
    pub players: PlayerCounts,
    pub game_mode_ids: Vec<i64>,
    /// Names added by hand; `None` keeps them.
    pub names: Option<Vec<String>>,
    /// `None` keeps them.
    pub label_ids: Option<Vec<i64>>,
}

pub struct CopyFields {
    /// `None` keeps the current status, or makes a new copy owned.
    pub status: Option<CopyStatus>,
    pub platform_id: i64,
    pub region: Region,
    pub product_code: Option<String>,
    pub format: Format,
    pub device_id: Option<i64>,
    pub store_id: Option<i64>,
    pub condition: Option<Condition>,
    pub notes: Option<String>,
    pub edition: Option<String>,
    pub purchased_on: Option<String>,
    pub price: Option<Price>,
    pub estimated_value: Option<Price>,
    pub barcode: Option<String>,
}

/// An amount in minor units and its currency.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Price {
    pub minor: i64,
    pub currency: String,
}

pub struct ReleaseFields {
    pub title: Option<String>,
    pub product_code: Option<String>,
    pub release_date: Option<String>,
    pub players: PlayerCounts,
}

/// The game as `viewer` sees it: the household's game with their own play.
pub async fn get(pool: &SqlitePool, id: i64, viewer: &str) -> ApiResult<GameDetail> {
    let game = sqlx::query!(
        r#"SELECT g.id AS "id!", g.igdb_id, g.title, g.summary, g.cover_path,
                  g.cover_source IS 'upload' AS "cover_uploaded!: bool", g.first_release_date,
                  g.developers AS "developers: Json<Vec<String>>",
                  g.publishers AS "publishers: Json<Vec<String>>",
                  g.local_max_players, g.online_max_players,
                  g.local_coop AS "local_coop: bool", g.online_coop AS "online_coop: bool",
                  g.time_to_beat_main, g.time_to_beat_extras, g.time_to_beat_full,
                  (SELECT json_group_array(game_mode_id) FROM game_game_modes WHERE game_id = g.id)
                      AS "game_mode_ids!: Json<Vec<i64>>",
                  (SELECT json_group_array(genre_id) FROM game_genres WHERE game_id = g.id)
                      AS "genre_ids!: Json<Vec<i64>>",
                  g.created_at, g.created_by,
                  COALESCE(cu.display_name, cu.name) AS created_by_name,
                  g.updated_by, COALESCE(uu.display_name, uu.name) AS updated_by_name
           FROM games g
           LEFT JOIN users cu ON cu.id = g.created_by
           LEFT JOIN users uu ON uu.id = g.updated_by
           WHERE g.id = ?"#,
        id
    )
    .fetch_optional(pool)
    .await?
    .ok_or(ApiError::NotFound)?;

    let releases = sqlx::query!(
        r#"SELECT r.id AS "id!", r.platform_id, r.region AS "region: Region", r.title, r.product_code,
                  r.release_date, r.cover_path, r.cover_source IS 'upload' AS "cover_uploaded!: bool",
                  r.local_max_players, r.online_max_players,
                  r.local_coop AS "local_coop: bool", r.online_coop AS "online_coop: bool"
           FROM releases r JOIN platforms p ON p.id = r.platform_id
           WHERE r.game_id = ?
           ORDER BY p.sort_order, r.region, r.id"#,
        id
    )
    .fetch_all(pool)
    .await?;

    let copies = sqlx::query!(
        r#"SELECT c.id AS "id!", c.release_id, c.status AS "status: CopyStatus",
                  c.format AS "format: Format", c.device_id, c.store_id,
                  c.condition AS "condition: Condition", c.notes, c.edition, c.purchased_on,
                  c.price, c.currency, c.estimated_value, c.estimated_currency,
                  c.barcode, c.source AS "source: CopySource", c.created_at,
                  c.created_by, COALESCE(cu.display_name, cu.name) AS created_by_name,
                  c.updated_by, COALESCE(uu.display_name, uu.name) AS updated_by_name
           FROM copies c JOIN releases r ON r.id = c.release_id
           LEFT JOIN users cu ON cu.id = c.created_by
           LEFT JOIN users uu ON uu.id = c.updated_by
           WHERE r.game_id = ?
           ORDER BY c.id"#,
        id
    )
    .fetch_all(pool)
    .await?;

    let links = sqlx::query!(
        r#"SELECT c.id AS "copy_id!", r.game_id, g.title,
                  b.id AS "bundle_copy_id!", br.game_id AS bundle_game_id, bg.title AS bundle_title
           FROM copies c
           JOIN releases r ON r.id = c.release_id JOIN games g ON g.id = r.game_id
           JOIN copies b ON b.id = c.bundle_id
           JOIN releases br ON br.id = b.release_id JOIN games bg ON bg.id = br.game_id
           WHERE r.game_id = ?1 OR br.game_id = ?1
           ORDER BY g.sort_title, c.id"#,
        id
    )
    .fetch_all(pool)
    .await?;
    let photos = sqlx::query!(
        r#"SELECT p.id AS "id!", p.copy_id, p.path, p.created_at
           FROM copy_photos p JOIN copies c ON c.id = p.copy_id
           JOIN releases r ON r.id = c.release_id
           WHERE r.game_id = ? ORDER BY p.id"#,
        id
    )
    .fetch_all(pool)
    .await?;
    let mut photos_of: HashMap<i64, Vec<CopyPhoto>> = HashMap::new();
    for p in photos {
        photos_of.entry(p.copy_id).or_default().push(CopyPhoto {
            id: p.id,
            path: p.path,
            created_at: p.created_at,
        });
    }
    let tags = sqlx::query!(
        r#"SELECT t.tag_id AS "tag_id!", t.copy_id, t.created_at
           FROM nfc_tags t JOIN copies c ON c.id = t.copy_id
           JOIN releases r ON r.id = c.release_id
           WHERE r.game_id = ? ORDER BY t.created_at, t.tag_id"#,
        id
    )
    .fetch_all(pool)
    .await?;
    let mut tags_of: HashMap<i64, Vec<CopyTag>> = HashMap::new();
    for t in tags {
        tags_of.entry(t.copy_id).or_default().push(CopyTag {
            tag_id: t.tag_id,
            created_at: t.created_at,
        });
    }
    let mut bundle_of: HashMap<i64, CopyLink> = HashMap::new();
    let mut contents_of: HashMap<i64, Vec<CopyLink>> = HashMap::new();
    for l in links {
        if l.game_id == id {
            bundle_of.entry(l.copy_id).or_insert_with(|| CopyLink {
                copy_id: l.bundle_copy_id,
                game_id: l.bundle_game_id,
                title: l.bundle_title.clone(),
            });
        }
        if l.bundle_game_id == id {
            contents_of
                .entry(l.bundle_copy_id)
                .or_default()
                .push(CopyLink {
                    copy_id: l.copy_id,
                    game_id: l.game_id,
                    title: l.title,
                });
        }
    }
    let mut copies_of: HashMap<i64, Vec<_>> = HashMap::new();
    for c in copies {
        copies_of.entry(c.release_id).or_default().push(c);
    }

    let releases = releases
        .into_iter()
        .map(|r| {
            let mine = copies_of.remove(&r.id).unwrap_or_default();
            ReleaseDetail {
                id: r.id,
                platform_id: r.platform_id,
                region: r.region,
                title: r.title,
                product_code: r.product_code,
                release_date: r.release_date,
                cover_path: r.cover_path,
                cover_uploaded: r.cover_uploaded,
                local_max_players: r.local_max_players,
                online_max_players: r.online_max_players,
                local_coop: r.local_coop,
                online_coop: r.online_coop,
                copies: mine
                    .into_iter()
                    .map(|c| CopyDetail {
                        id: c.id,
                        status: c.status,
                        format: c.format,
                        device_id: c.device_id,
                        store_id: c.store_id,
                        condition: c.condition,
                        notes: c.notes,
                        edition: c.edition,
                        purchased_on: c.purchased_on,
                        price: c
                            .price
                            .zip(c.currency.as_deref())
                            .map(|(minor, currency)| money::format(minor, currency)),
                        currency: c.currency,
                        estimated_value: c
                            .estimated_value
                            .zip(c.estimated_currency.as_deref())
                            .map(|(minor, currency)| money::format(minor, currency)),
                        estimated_currency: c.estimated_currency,
                        barcode: c.barcode,
                        source: c.source,
                        bundle: bundle_of.remove(&c.id),
                        contents: contents_of.remove(&c.id).unwrap_or_default(),
                        photos: photos_of.remove(&c.id).unwrap_or_default(),
                        tags: tags_of.remove(&c.id).unwrap_or_default(),
                        created_at: c.created_at,
                        created_by: Person::from_columns(c.created_by, c.created_by_name),
                        updated_by: Person::from_columns(c.updated_by, c.updated_by_name),
                    })
                    .collect(),
            }
        })
        .collect();

    let mut game_mode_ids = game.game_mode_ids.0;
    game_mode_ids.sort_unstable();
    let mut genre_ids = game.genre_ids.0;
    genre_ids.sort_unstable();

    Ok(GameDetail {
        id: game.id,
        igdb_id: game.igdb_id,
        title: game.title,
        summary: game.summary,
        cover_path: game.cover_path,
        cover_uploaded: game.cover_uploaded,
        first_release_date: game.first_release_date,
        developers: game.developers.0,
        publishers: game.publishers.0,
        play_status: super::progress::get(pool, viewer, id).await?,
        local_max_players: game.local_max_players,
        online_max_players: game.online_max_players,
        local_coop: game.local_coop,
        online_coop: game.online_coop,
        time_to_beat: TimeToBeat {
            main: game.time_to_beat_main,
            extras: game.time_to_beat_extras,
            full: game.time_to_beat_full,
        },
        game_mode_ids,
        genre_ids,
        labels: super::labels::for_game(pool, id).await?,
        created_at: game.created_at,
        created_by: Person::from_columns(game.created_by, game.created_by_name),
        updated_by: Person::from_columns(game.updated_by, game.updated_by_name),
        releases,
        sessions: sessions::for_person(pool, id, viewer).await?,
        household: sessions::household(pool, id, viewer).await?,
        versions: versions(pool, id).await?,
        names: sqlx::query_as!(
            GameName,
            r#"SELECT name, source = 'igdb' AS "from_igdb!: bool", comment
               FROM game_names WHERE game_id = ? ORDER BY source = 'igdb', name"#,
            id
        )
        .fetch_all(pool)
        .await?,
    })
}

/// Adds a game, with `play_status` as its creator's own.
pub async fn create(
    pool: &SqlitePool,
    title: &str,
    play_status: Option<&str>,
    by: &str,
) -> ApiResult<i64> {
    let mut tx = crate::db::write_tx(pool).await?;
    super::play_statuses::ensure_exists(&mut *tx, play_status).await?;
    let sort = sort_title(title);
    let id = sqlx::query_scalar!(
        r#"INSERT INTO games (title, sort_title, created_by, updated_by)
           VALUES (?, ?, ?, ?) RETURNING id AS "id!""#,
        title,
        sort,
        by,
        by
    )
    .fetch_one(&mut *tx)
    .await?;
    if let Some(status) = play_status {
        sqlx::query!(
            "INSERT INTO play_progress (user_id, game_id, play_status) VALUES (?, ?, ?)",
            by,
            id,
            status
        )
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(id)
}

pub async fn update(pool: &SqlitePool, id: i64, fields: &GameFields, by: &str) -> ApiResult<()> {
    let mut tx = crate::db::write_tx(pool).await?;
    let modes = game_mode_ids(&mut tx, &fields.game_mode_ids).await?;

    let sort = sort_title(&fields.title);
    let p = &fields.players;
    let result = sqlx::query!(
        "UPDATE games
         SET title = ?, sort_title = ?, summary = ?,
             local_max_players = ?, online_max_players = ?,
             local_coop = ?, online_coop = ?, updated_by = ?,
             updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE id = ?",
        fields.title,
        sort,
        fields.summary,
        p.local_max_players,
        p.online_max_players,
        p.local_coop,
        p.online_coop,
        by,
        id
    )
    .execute(&mut *tx)
    .await?;
    found(result.rows_affected())?;

    sqlx::query!("DELETE FROM game_game_modes WHERE game_id = ?", id)
        .execute(&mut *tx)
        .await?;
    sqlx::query!(
        "INSERT INTO game_game_modes (game_id, game_mode_id) SELECT ?, value FROM json_each(?)",
        id,
        modes
    )
    .execute(&mut *tx)
    .await?;

    if let Some(ids) = &fields.label_ids {
        super::labels::set_for_game(&mut tx, id, ids).await?;
    }

    if let Some(names) = &fields.names {
        sqlx::query!(
            "DELETE FROM game_names WHERE game_id = ? AND source = 'user'",
            id
        )
        .execute(&mut *tx)
        .await?;
        for name in names {
            // A name IGDB already gives becomes the user's, so it survives a refresh.
            sqlx::query!(
                "INSERT INTO game_names (game_id, name, source) VALUES (?, ?, 'user')
                 ON CONFLICT (game_id, name) DO UPDATE SET source = 'user', comment = NULL",
                id,
                name
            )
            .execute(&mut *tx)
            .await?;
        }
    }

    tx.commit().await?;
    Ok(())
}

/// Deletes a game with its releases and copies, and returns every release
/// deleted, the emptied ones of other games included.
pub async fn delete(pool: &SqlitePool, id: i64) -> ApiResult<Vec<DroppedRelease>> {
    let mut tx = crate::db::write_tx(pool).await?;
    let mut releases: Vec<DroppedRelease> = sqlx::query_scalar!(
        r#"DELETE FROM releases WHERE game_id = ? RETURNING id AS "id!""#,
        id
    )
    .fetch_all(&mut *tx)
    .await?
    .into_iter()
    .map(|release| DroppedRelease {
        game_id: id,
        release_id: release,
    })
    .collect();
    let result = sqlx::query!("DELETE FROM games WHERE id = ?", id)
        .execute(&mut *tx)
        .await?;
    found(result.rows_affected())?;
    releases.extend(drop_empty_releases(&mut tx).await?);
    tx.commit().await?;
    Ok(releases)
}

/// A release deleted with its game or because no copy was left in it, for
/// removing its cover files.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DroppedRelease {
    pub game_id: i64,
    pub release_id: i64,
}

/// Deletes every release left with no copies. Runs after each write that
/// moves or deletes copies; a deleted bundle copy takes its contents, in other
/// games' releases, with it.
pub(crate) async fn drop_empty_releases(
    conn: &mut SqliteConnection,
) -> sqlx::Result<Vec<DroppedRelease>> {
    Ok(sqlx::query!(
        r#"DELETE FROM releases
           WHERE NOT EXISTS (SELECT 1 FROM copies WHERE copies.release_id = releases.id)
           RETURNING game_id, id AS "id!""#
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(|row| DroppedRelease {
        game_id: row.game_id,
        release_id: row.id,
    })
    .collect())
}

/// Adds a copy, creating its release when the game has none for that
/// platform, region and product code.
pub async fn add_copy(
    pool: &SqlitePool,
    game_id: i64,
    fields: &CopyFields,
    by: &str,
) -> ApiResult<()> {
    let mut tx = crate::db::write_tx(pool).await?;
    let game_exists: bool = sqlx::query_scalar!(
        r#"SELECT EXISTS (SELECT 1 FROM games WHERE id = ?) AS "exists!: bool""#,
        game_id
    )
    .fetch_one(&mut *tx)
    .await?;
    if !game_exists {
        return Err(ApiError::NotFound);
    }
    check_references(&mut tx, fields).await?;
    let release_id = release_for(&mut tx, game_id, fields).await?;
    let status = fields.status.unwrap_or(CopyStatus::Owned);
    let (price, currency) = money_columns(fields.price.as_ref());
    let (value, value_currency) = money_columns(fields.estimated_value.as_ref());
    sqlx::query!(
        "INSERT INTO copies (release_id, status, format, device_id, store_id, condition, notes,
                             edition, purchased_on, price, currency, estimated_value,
                             estimated_currency, barcode, created_by, updated_by)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        release_id,
        status,
        fields.format,
        fields.device_id,
        fields.store_id,
        fields.condition,
        fields.notes,
        fields.edition,
        fields.purchased_on,
        price,
        currency,
        value,
        value_currency,
        fields.barcode,
        by,
        by
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(())
}

/// Updates a copy and returns its game id. Changing platform, region or
/// product code moves it to the matching release, creating it if needed.
pub async fn update_copy(
    pool: &SqlitePool,
    copy_id: i64,
    fields: &CopyFields,
    by: &str,
) -> ApiResult<(i64, Vec<DroppedRelease>)> {
    let mut tx = crate::db::write_tx(pool).await?;
    let game_id = copy_game(&mut tx, copy_id).await?;
    let in_bundle = sqlx::query_scalar!("SELECT bundle_id FROM copies WHERE id = ?", copy_id)
        .fetch_one(&mut *tx)
        .await?
        .is_some();
    if in_bundle {
        // Everything else follows the bundle copy.
        sqlx::query!(
            "UPDATE copies SET notes = ?, updated_by = ?,
                 updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
             WHERE id = ?",
            fields.notes,
            by,
            copy_id
        )
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        return Ok((game_id, Vec::new()));
    }
    check_references(&mut tx, fields).await?;
    let release_id = release_for(&mut tx, game_id, fields).await?;
    let (price, currency) = money_columns(fields.price.as_ref());
    let (value, value_currency) = money_columns(fields.estimated_value.as_ref());
    sqlx::query!(
        "UPDATE copies
         SET release_id = ?, status = COALESCE(?, status), format = ?, device_id = ?,
             store_id = ?, condition = ?, notes = ?, edition = ?, purchased_on = ?,
             price = ?, currency = ?, estimated_value = ?, estimated_currency = ?,
             barcode = ?, updated_by = ?,
             updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE id = ?",
        release_id,
        fields.status,
        fields.format,
        fields.device_id,
        fields.store_id,
        fields.condition,
        fields.notes,
        fields.edition,
        fields.purchased_on,
        price,
        currency,
        value,
        value_currency,
        fields.barcode,
        by,
        copy_id
    )
    .execute(&mut *tx)
    .await?;
    sync_contents(&mut tx, copy_id).await?;
    let dropped = drop_empty_releases(&mut tx).await?;
    tx.commit().await?;
    Ok((game_id, dropped))
}

/// Copies the bundle copy's shared fields onto its contents and moves them
/// to their games' releases on the bundle's platform and region.
pub(crate) async fn sync_contents(conn: &mut SqliteConnection, bundle_id: i64) -> sqlx::Result<()> {
    sqlx::query!(
        "UPDATE copies
         SET status = b.status, format = b.format, device_id = b.device_id,
             store_id = b.store_id, condition = b.condition,
             updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         FROM (SELECT status, format, device_id, store_id, condition FROM copies WHERE id = ?1) AS b
         WHERE copies.bundle_id = ?1",
        bundle_id
    )
    .execute(&mut *conn)
    .await?;
    let moved = sqlx::query!(
        r#"SELECT c.id AS "id!", r.game_id, br.platform_id, br.region AS "region: Region"
           FROM copies c JOIN releases r ON r.id = c.release_id
           JOIN copies b ON b.id = c.bundle_id JOIN releases br ON br.id = b.release_id
           WHERE c.bundle_id = ? AND (r.platform_id != br.platform_id OR r.region != br.region)"#,
        bundle_id
    )
    .fetch_all(&mut *conn)
    .await?;
    for copy in moved {
        let release = release_id(conn, copy.game_id, copy.platform_id, copy.region, None).await?;
        sqlx::query!(
            "UPDATE copies SET release_id = ? WHERE id = ?",
            release,
            copy.id
        )
        .execute(&mut *conn)
        .await?;
    }
    Ok(())
}

/// Adds a copy of `game_id` inside a bundle copy and returns the bundle's game id.
pub async fn add_content(
    pool: &SqlitePool,
    bundle_id: i64,
    game_id: i64,
    by: &str,
) -> ApiResult<i64> {
    let mut tx = crate::db::write_tx(pool).await?;
    let bundle = sqlx::query!(
        r#"SELECT c.bundle_id, r.game_id, r.platform_id, r.region AS "region: Region"
           FROM copies c JOIN releases r ON r.id = c.release_id WHERE c.id = ?"#,
        bundle_id
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(ApiError::NotFound)?;
    if bundle.bundle_id.is_some() {
        return Err(ApiError::Conflict("nested_bundle"));
    }
    if bundle.game_id == game_id {
        return Err(ApiError::invalid("same_game", "game_id"));
    }
    let exists: bool = sqlx::query_scalar!(
        r#"SELECT EXISTS (SELECT 1 FROM games WHERE id = ?) AS "exists!: bool""#,
        game_id
    )
    .fetch_one(&mut *tx)
    .await?;
    if !exists {
        return Err(ApiError::invalid("unknown_game", "game_id"));
    }
    let already: bool = sqlx::query_scalar!(
        r#"SELECT EXISTS (SELECT 1 FROM copies c JOIN releases r ON r.id = c.release_id
                          WHERE c.bundle_id = ? AND r.game_id = ?) AS "already!: bool""#,
        bundle_id,
        game_id
    )
    .fetch_one(&mut *tx)
    .await?;
    if already {
        return Err(ApiError::Conflict("already_in_bundle"));
    }
    let release = release_id(&mut tx, game_id, bundle.platform_id, bundle.region, None).await?;
    sqlx::query!(
        "INSERT INTO copies (release_id, status, format, device_id, store_id, condition,
                             bundle_id, created_by, updated_by)
         SELECT ?, status, format, device_id, store_id, condition, id, ?, ?
         FROM copies WHERE id = ?",
        release,
        by,
        by,
        bundle_id
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(bundle.game_id)
}

/// Deletes a copy and returns its game id, with any release left empty.
pub async fn delete_copy(pool: &SqlitePool, copy_id: i64) -> ApiResult<(i64, Vec<DroppedRelease>)> {
    let mut tx = crate::db::write_tx(pool).await?;
    let game_id = copy_game(&mut tx, copy_id).await?;
    sqlx::query!("DELETE FROM copies WHERE id = ?", copy_id)
        .execute(&mut *tx)
        .await?;
    let dropped = drop_empty_releases(&mut tx).await?;
    tx.commit().await?;
    Ok((game_id, dropped))
}

/// Updates a release and returns its game id.
pub async fn update_release(
    pool: &SqlitePool,
    release_id: i64,
    fields: &ReleaseFields,
) -> ApiResult<i64> {
    let p = &fields.players;
    sqlx::query_scalar!(
        "UPDATE releases
         SET title = ?, product_code = ?, release_date = ?, local_max_players = ?,
             online_max_players = ?, local_coop = ?, online_coop = ?,
             updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE id = ?
         RETURNING game_id",
        fields.title,
        fields.product_code,
        fields.release_date,
        p.local_max_players,
        p.online_max_players,
        p.local_coop,
        p.online_coop,
        release_id
    )
    .fetch_optional(pool)
    .await
    .map_err(|err| ApiError::unique(err, "release_exists"))?
    .ok_or(ApiError::NotFound)
}

/// Deletes a release with its copies and returns its game id.
pub async fn delete_release(
    pool: &SqlitePool,
    release_id: i64,
) -> ApiResult<(i64, Vec<DroppedRelease>)> {
    let mut tx = crate::db::write_tx(pool).await?;
    let game_id = sqlx::query_scalar!(
        "DELETE FROM releases WHERE id = ? RETURNING game_id",
        release_id
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(ApiError::NotFound)?;
    let dropped = drop_empty_releases(&mut tx).await?;
    tx.commit().await?;
    Ok((game_id, dropped))
}

fn money_columns(price: Option<&Price>) -> (Option<i64>, Option<&str>) {
    price.map(|p| (p.minor, p.currency.as_str())).unzip()
}

pub async fn copy_exists(pool: &SqlitePool, copy_id: i64) -> ApiResult<()> {
    let exists: bool = sqlx::query_scalar!(
        r#"SELECT EXISTS (SELECT 1 FROM copies WHERE id = ?) AS "exists!: bool""#,
        copy_id
    )
    .fetch_one(pool)
    .await?;
    if exists {
        Ok(())
    } else {
        Err(ApiError::NotFound)
    }
}

async fn release_game(pool: &SqlitePool, release_id: i64) -> ApiResult<i64> {
    sqlx::query_scalar!("SELECT game_id FROM releases WHERE id = ?", release_id)
        .fetch_optional(pool)
        .await?
        .ok_or(ApiError::NotFound)
}

async fn copy_game(conn: &mut SqliteConnection, copy_id: i64) -> ApiResult<i64> {
    sqlx::query_scalar!(
        "SELECT r.game_id FROM copies c JOIN releases r ON r.id = c.release_id WHERE c.id = ?",
        copy_id
    )
    .fetch_optional(&mut *conn)
    .await?
    .ok_or(ApiError::NotFound)
}

async fn check_references(conn: &mut SqliteConnection, fields: &CopyFields) -> ApiResult<()> {
    let platform: bool = sqlx::query_scalar!(
        r#"SELECT EXISTS (SELECT 1 FROM platforms WHERE id = ?) AS "exists!: bool""#,
        fields.platform_id
    )
    .fetch_one(&mut *conn)
    .await?;
    if !platform {
        return Err(ApiError::invalid("unknown_platform", "platform_id"));
    }
    if let Some(device_id) = fields.device_id {
        let exists: bool = sqlx::query_scalar!(
            r#"SELECT EXISTS (SELECT 1 FROM devices WHERE id = ?) AS "exists!: bool""#,
            device_id
        )
        .fetch_one(&mut *conn)
        .await?;
        if !exists {
            return Err(ApiError::invalid("unknown_device", "device_id"));
        }
    }
    if let Some(store_id) = fields.store_id {
        let exists: bool = sqlx::query_scalar!(
            r#"SELECT EXISTS (SELECT 1 FROM stores WHERE id = ?) AS "exists!: bool""#,
            store_id
        )
        .fetch_one(&mut *conn)
        .await?;
        if !exists {
            return Err(ApiError::invalid("unknown_store", "store_id"));
        }
    }
    Ok(())
}

async fn release_for(
    conn: &mut SqliteConnection,
    game_id: i64,
    fields: &CopyFields,
) -> ApiResult<i64> {
    release_id(
        conn,
        game_id,
        fields.platform_id,
        fields.region,
        fields.product_code.as_deref(),
    )
    .await
    .map_err(Into::into)
}

/// The game's release for this platform, region and product code, created if needed.
async fn release_id(
    conn: &mut SqliteConnection,
    game_id: i64,
    platform_id: i64,
    region: Region,
    product_code: Option<&str>,
) -> sqlx::Result<i64> {
    let existing = sqlx::query_scalar!(
        r#"SELECT id AS "id!" FROM releases
           WHERE game_id = ? AND platform_id = ? AND region = ?
             AND lower(COALESCE(product_code, '')) = lower(COALESCE(?, ''))"#,
        game_id,
        platform_id,
        region,
        product_code
    )
    .fetch_optional(&mut *conn)
    .await?;
    if let Some(id) = existing {
        return Ok(id);
    }
    sqlx::query_scalar!(
        r#"INSERT INTO releases (game_id, platform_id, region, product_code)
           VALUES (?, ?, ?, ?) RETURNING id AS "id!""#,
        game_id,
        platform_id,
        region,
        product_code
    )
    .fetch_one(&mut *conn)
    .await
}

/// Deduplicates ids, checks they exist in `game_modes`, returns them as a JSON array.
async fn game_mode_ids(conn: &mut SqliteConnection, ids: &[i64]) -> ApiResult<String> {
    let mut ids = ids.to_vec();
    ids.sort_unstable();
    ids.dedup();
    let json = serde_json::to_string(&ids).expect("integers serialize");
    let known: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM game_modes WHERE id IN (SELECT value FROM json_each(?))",
    )
    .bind(&json)
    .fetch_one(&mut *conn)
    .await?;
    if known != ids.len() as i64 {
        return Err(ApiError::invalid("unknown_id", "game_mode_ids"));
    }
    Ok(json)
}

/// The game a cover belongs to.
pub async fn cover_game(pool: &SqlitePool, owner: Owner) -> ApiResult<i64> {
    match owner {
        Owner::Game(id) => {
            sqlx::query_scalar!(r#"SELECT id AS "id!" FROM games WHERE id = ?"#, id)
                .fetch_optional(pool)
                .await?
        }
        Owner::Release(id) => release_game(pool, id).await.map(Some)?,
    }
    .ok_or(ApiError::NotFound)
}

/// Sets an uploaded cover, or with `None` removes one. Returns whether the row changed.
pub async fn set_uploaded_cover(
    pool: &SqlitePool,
    owner: Owner,
    path: Option<&str>,
) -> ApiResult<bool> {
    let result = match (owner, path) {
        (Owner::Game(id), Some(path)) => sqlx::query!(
            "UPDATE games SET cover_path = ?, cover_source = 'upload',
                 updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
             WHERE id = ?",
            path,
            id
        )
        .execute(pool),
        (Owner::Game(id), None) => sqlx::query!(
            "UPDATE games SET cover_path = NULL, cover_source = NULL,
                 updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
             WHERE id = ? AND cover_source = 'upload'",
            id
        )
        .execute(pool),
        (Owner::Release(id), Some(path)) => sqlx::query!(
            "UPDATE releases SET cover_path = ?, cover_source = 'upload',
                 updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
             WHERE id = ?",
            path,
            id
        )
        .execute(pool),
        (Owner::Release(id), None) => sqlx::query!(
            "UPDATE releases SET cover_path = NULL, cover_source = NULL,
                 updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
             WHERE id = ? AND cover_source = 'upload'",
            id
        )
        .execute(pool),
    }
    .await?;
    Ok(result.rows_affected() > 0)
}

#[derive(Debug, Serialize, ToSchema)]
pub struct GameMode {
    pub id: i64,
    pub name: String,
    pub slug: String,
}

pub async fn game_modes(pool: &SqlitePool) -> sqlx::Result<Vec<GameMode>> {
    sqlx::query_as!(
        GameMode,
        r#"SELECT id AS "id!", name, slug FROM game_modes ORDER BY id"#
    )
    .fetch_all(pool)
    .await
}

#[derive(Debug, Serialize, ToSchema)]
pub struct Genre {
    pub id: i64,
    pub name: String,
    pub slug: String,
}

pub async fn genres(pool: &SqlitePool) -> sqlx::Result<Vec<Genre>> {
    sqlx::query_as!(
        Genre,
        r#"SELECT id AS "id!", name, slug FROM genres ORDER BY name"#
    )
    .fetch_all(pool)
    .await
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BarcodeCopy {
    pub copy_id: i64,
    pub game_id: i64,
    pub title: String,
    pub platform_id: i64,
    pub region: Region,
    pub status: CopyStatus,
}

/// Copies with this barcode, by game.
pub async fn with_barcode(pool: &SqlitePool, barcode: &str) -> sqlx::Result<Vec<BarcodeCopy>> {
    sqlx::query_as!(
        BarcodeCopy,
        r#"SELECT c.id AS "copy_id!", g.id AS "game_id!", g.title, r.platform_id,
                  r.region AS "region: Region", c.status AS "status: CopyStatus"
           FROM copies c JOIN releases r ON r.id = c.release_id JOIN games g ON g.id = r.game_id
           WHERE c.barcode = ?
           ORDER BY g.sort_title, c.id"#,
        barcode
    )
    .fetch_all(pool)
    .await
}

/// The currency of the copy changed last that has one.
pub async fn last_currency(pool: &SqlitePool) -> sqlx::Result<Option<String>> {
    sqlx::query_scalar(
        "SELECT currency FROM copies WHERE currency IS NOT NULL
         ORDER BY updated_at DESC, id DESC LIMIT 1",
    )
    .fetch_optional(pool)
    .await
}
