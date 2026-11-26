//! IGDB links, and the metadata `igdb::store` writes onto games, releases,
//! genres and game modes.

use sqlx::{Encode, Sqlite, SqlitePool, Transaction, Type};

use super::games::{PlayerCounts, TimeToBeat};
use super::slugs::{self, Table};
use super::sql::Sql;
use super::write_tx;
use crate::covers::Owner;
use crate::error::{ApiError, ApiResult};

const NOW: &str = "strftime('%Y-%m-%dT%H:%M:%fZ', 'now')";

/// Whether a game other than `game_id` is linked to `igdb_id`.
pub async fn linked_elsewhere(pool: &SqlitePool, igdb_id: i64, game_id: i64) -> sqlx::Result<bool> {
    sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM games WHERE igdb_id = ? AND id != ?)")
        .bind(igdb_id)
        .bind(game_id)
        .fetch_one(pool)
        .await
}

/// What a write needs the game to still be. A sync lists its games before
/// fetching them, and a manager may relink or rename one meanwhile.
#[derive(Debug, Clone, Copy)]
pub enum Expected<'a> {
    /// Whatever it is now, for a manager's own link or refresh.
    Any,
    /// Linked to this IGDB id.
    Link(i64),
    /// Unlinked, with the title it was matched by.
    Unlinked(&'a str),
}

/// ` AND ...` for the games that are still as `expected`.
fn push_expected(sql: &mut Sql, expected: Expected<'_>) {
    match expected {
        Expected::Any => {}
        Expected::Link(igdb_id) => {
            sql.push(" AND igdb_id = ").push_bind(igdb_id);
        }
        Expected::Unlinked(title) => {
            sql.push(" AND igdb_id IS NULL AND title = ")
                .push_bind(title.to_owned());
        }
    }
}

/// A release, with what IGDB needs to fill it.
#[derive(sqlx::FromRow)]
pub struct Release {
    pub id: i64,
    pub region: String,
    pub igdb_platform_id: Option<i64>,
    pub cover_source: Option<String>,
}

pub async fn releases(pool: &SqlitePool, game_id: i64) -> sqlx::Result<Vec<Release>> {
    sqlx::query_as(
        "SELECT r.id, r.region, p.igdb_platform_id, r.cover_source
         FROM releases r JOIN platforms p ON p.id = r.platform_id WHERE r.game_id = ?",
    )
    .bind(game_id)
    .fetch_all(pool)
    .await
}

/// The game's `cover_source`, `None` when there is no such game.
pub async fn cover_source(pool: &SqlitePool, game_id: i64) -> sqlx::Result<Option<Option<String>>> {
    sqlx::query_scalar("SELECT cover_source FROM games WHERE id = ?")
        .bind(game_id)
        .fetch_optional(pool)
        .await
}

/// Downloaded covers, relative to the covers directory.
pub struct Covers {
    /// `None` keeps the stored cover.
    pub game: Option<String>,
    /// IGDB owns regional covers, so `None` clears one.
    pub releases: Vec<(i64, Option<String>)>,
}

/// A genre or game mode as IGDB names it.
pub struct Tag<'a> {
    pub igdb_id: i64,
    pub name: &'a str,
    pub slug_from: &'a str,
}

pub struct ReleaseUpdate {
    pub id: i64,
    pub counts: PlayerCounts,
    pub release_date: Option<String>,
    pub title: Option<String>,
}

/// What IGDB writes onto a game.
pub struct GameUpdate<'a> {
    pub igdb_id: i64,
    pub summary: Option<&'a str>,
    pub first_release_date: Option<String>,
    pub developers: Vec<String>,
    pub publishers: Vec<String>,
    pub counts: PlayerCounts,
    /// Replaces the stored one, since only IGDB sets it.
    pub time_to_beat: TimeToBeat,
    /// Replaces the title, with its sort title.
    pub title: Option<(&'a str, String)>,
    pub igdb_title: Option<&'a str>,
    /// Alternative names, with IGDB's comment.
    pub names: Vec<(&'a str, Option<&'a str>)>,
    pub links: Vec<(i64, &'static str)>,
    pub genres: Vec<Tag<'a>>,
    /// Only stored when the game has no game modes, or `replace_modes`.
    pub game_modes: Vec<Tag<'a>>,
    pub replace_modes: bool,
    pub releases: Vec<ReleaseUpdate>,
}

/// Links the game and stores `update` and the covers in one transaction.
/// `overwrite` lets IGDB values replace stored ones; a missing value never
/// clears one. Returns the covers that changed.
pub async fn write(
    pool: &SqlitePool,
    game_id: i64,
    update: &GameUpdate<'_>,
    overwrite: bool,
    covers: &Covers,
    expected: Expected<'_>,
) -> ApiResult<Vec<(Owner, Option<String>)>> {
    let mut tx = write_tx(pool).await?;
    let counts = &update.counts;
    let mut game = Sql::new("UPDATE games SET igdb_id = ");
    game.push_bind(update.igdb_id).push(", ");
    push_fill(
        &mut game,
        "summary",
        update.summary.map(str::to_owned),
        overwrite,
    );
    game.push(", first_release_date = COALESCE(")
        .push_bind(update.first_release_date.clone())
        .push(", first_release_date), developers = ")
        .push_bind(serde_json::to_string(&update.developers).expect("strings serialize"))
        .push(", publishers = ")
        .push_bind(serde_json::to_string(&update.publishers).expect("strings serialize"))
        .push(", time_to_beat_main = ")
        .push_bind(update.time_to_beat.main)
        .push(", time_to_beat_extras = ")
        .push_bind(update.time_to_beat.extras)
        .push(", time_to_beat_full = ")
        .push_bind(update.time_to_beat.full)
        .push(", ");
    push_counts(&mut game, counts, overwrite);
    let (title, sort_title) = update.title.clone().unzip();
    game.push(", title = COALESCE(")
        .push_bind(title.map(str::to_owned))
        .push(", title), sort_title = COALESCE(")
        .push_bind(sort_title)
        .push(", sort_title), igdb_title = COALESCE(")
        .push_bind(update.igdb_title.map(str::to_owned))
        .push(", igdb_title), igdb_synced_at = ")
        .push(NOW)
        .push(", updated_at = ")
        .push(NOW)
        .push(" WHERE id = ")
        .push_bind(game_id);
    push_expected(&mut game, expected);
    let updated = game
        .build()
        .execute(&mut *tx)
        .await
        .map_err(|err| ApiError::unique(err, "igdb_linked"))?;
    if updated.rows_affected() == 0 {
        return Err(match expected {
            Expected::Any => ApiError::NotFound,
            _ => ApiError::Conflict("igdb_changed"),
        });
    }

    sqlx::query("DELETE FROM game_names WHERE game_id = ? AND source = 'igdb'")
        .bind(game_id)
        .execute(&mut *tx)
        .await?;
    for (name, comment) in &update.names {
        // A name added by hand stays as it is.
        sqlx::query(
            "INSERT OR IGNORE INTO game_names (game_id, name, source, comment) VALUES (?, ?, 'igdb', ?)",
        )
        .bind(game_id)
        .bind(*name)
        .bind(*comment)
        .execute(&mut *tx)
        .await?;
    }

    sqlx::query("DELETE FROM game_links WHERE game_id = ?")
        .bind(game_id)
        .execute(&mut *tx)
        .await?;
    for (igdb_id, kind) in &update.links {
        sqlx::query("INSERT OR IGNORE INTO game_links (game_id, igdb_id, kind) VALUES (?, ?, ?)")
            .bind(game_id)
            .bind(igdb_id)
            .bind(*kind)
            .execute(&mut *tx)
            .await?;
    }

    sqlx::query("DELETE FROM game_genres WHERE game_id = ?")
        .bind(game_id)
        .execute(&mut *tx)
        .await?;
    for genre in &update.genres {
        let id = tag_id(&mut tx, TagKind::Genre, genre).await?;
        sqlx::query("INSERT OR IGNORE INTO game_genres (game_id, genre_id) VALUES (?, ?)")
            .bind(game_id)
            .bind(id)
            .execute(&mut *tx)
            .await?;
    }

    if update.replace_modes {
        sqlx::query("DELETE FROM game_game_modes WHERE game_id = ?")
            .bind(game_id)
            .execute(&mut *tx)
            .await?;
    }
    let has_modes: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM game_game_modes WHERE game_id = ?)")
            .bind(game_id)
            .fetch_one(&mut *tx)
            .await?;
    if !has_modes {
        for mode in &update.game_modes {
            let id = tag_id(&mut tx, TagKind::GameMode, mode).await?;
            sqlx::query(
                "INSERT OR IGNORE INTO game_game_modes (game_id, game_mode_id) VALUES (?, ?)",
            )
            .bind(game_id)
            .bind(id)
            .execute(&mut *tx)
            .await?;
        }
    }

    for release in &update.releases {
        let mut sql = Sql::new("UPDATE releases SET ");
        push_counts(&mut sql, &release.counts, overwrite);
        sql.push(", ");
        push_fill(
            &mut sql,
            "release_date",
            release.release_date.clone(),
            overwrite,
        );
        sql.push(", ");
        push_fill(&mut sql, "title", release.title.clone(), overwrite);
        sql.push(", updated_at = ")
            .push(NOW)
            .push(" WHERE id = ")
            .push_bind(release.id)
            // A release deleted meanwhile may have left its id to another game's.
            .push(" AND game_id = ")
            .push_bind(game_id);
        sql.build().execute(&mut *tx).await?;
    }

    let changed = store_covers(&mut tx, game_id, covers).await?;
    tx.commit().await?;
    Ok(changed)
}

/// The cover each of the game's owners has stored.
pub async fn stored_covers(
    pool: &SqlitePool,
    game_id: i64,
) -> sqlx::Result<Vec<(Owner, Option<String>)>> {
    let game: Option<String> = sqlx::query_scalar("SELECT cover_path FROM games WHERE id = ?")
        .bind(game_id)
        .fetch_optional(pool)
        .await?
        .flatten();
    let releases: Vec<(i64, Option<String>)> =
        sqlx::query_as("SELECT id, cover_path FROM releases WHERE game_id = ?")
            .bind(game_id)
            .fetch_all(pool)
            .await?;
    Ok(std::iter::once((Owner::Game(game_id), game))
        .chain(
            releases
                .into_iter()
                .map(|(id, path)| (Owner::Release(id), path)),
        )
        .collect())
}

/// Stores the covers alone. Returns the ones that changed.
pub async fn write_covers(
    pool: &SqlitePool,
    game_id: i64,
    covers: &Covers,
    expected: Expected<'_>,
) -> ApiResult<Vec<(Owner, Option<String>)>> {
    let mut tx = write_tx(pool).await?;
    let mut still = Sql::new("SELECT EXISTS (SELECT 1 FROM games WHERE id = ");
    still.push_bind(game_id);
    push_expected(&mut still, expected);
    still.push(")");
    if !still
        .build_query_scalar::<bool>()
        .fetch_one(&mut *tx)
        .await?
    {
        return Err(ApiError::Conflict("igdb_changed"));
    }
    let changed = store_covers(&mut tx, game_id, covers).await?;
    tx.commit().await?;
    Ok(changed)
}

/// `column = value`, keeping the stored value unless `overwrite`.
fn push_fill<T>(sql: &mut Sql, column: &'static str, value: T, overwrite: bool)
where
    T: for<'q> Encode<'q, Sqlite> + Type<Sqlite>,
{
    sql.push(column).push(" = COALESCE(");
    if overwrite {
        sql.push_bind(value).push(", ").push(column);
    } else {
        sql.push(column).push(", ").push_bind(value);
    }
    sql.push(")");
}

fn push_counts(sql: &mut Sql, counts: &PlayerCounts, overwrite: bool) {
    push_fill(
        sql,
        "local_max_players",
        counts.local_max_players,
        overwrite,
    );
    sql.push(", ");
    push_fill(
        sql,
        "online_max_players",
        counts.online_max_players,
        overwrite,
    );
    sql.push(", ");
    push_fill(sql, "local_coop", counts.local_coop, overwrite);
    sql.push(", ");
    push_fill(sql, "online_coop", counts.online_coop, overwrite);
}

/// Stores the covers unless an upload arrived meanwhile, and returns what changed.
async fn store_covers(
    tx: &mut Transaction<'static, Sqlite>,
    game_id: i64,
    covers: &Covers,
) -> sqlx::Result<Vec<(Owner, Option<String>)>> {
    let mut changed = Vec::new();
    if let Some(path) = &covers.game {
        let result = sqlx::query(
            "UPDATE games SET cover_path = ?, cover_source = 'igdb'
             WHERE id = ? AND cover_source IS NOT 'upload'",
        )
        .bind(path)
        .bind(game_id)
        .execute(&mut **tx)
        .await?;
        if result.rows_affected() > 0 {
            changed.push((Owner::Game(game_id), Some(path.clone())));
        }
    }
    for (release_id, path) in &covers.releases {
        let result = sqlx::query(
            "UPDATE releases SET cover_path = ?1,
                cover_source = CASE WHEN ?1 IS NULL THEN NULL ELSE 'igdb_localization' END
             WHERE id = ?2 AND game_id = ?3 AND cover_source IS NOT 'upload'",
        )
        .bind(path)
        .bind(release_id)
        .bind(game_id)
        .execute(&mut **tx)
        .await?;
        if result.rows_affected() > 0 {
            changed.push((Owner::Release(*release_id), path.clone()));
        }
    }
    Ok(changed)
}

#[derive(Clone, Copy)]
enum TagKind {
    Genre,
    GameMode,
}

impl TagKind {
    fn table(self) -> Table {
        match self {
            Self::Genre => Table::Genres,
            Self::GameMode => Table::GameModes,
        }
    }

    fn igdb_column(self) -> &'static str {
        match self {
            Self::Genre => "igdb_genre_id",
            Self::GameMode => "igdb_game_mode_id",
        }
    }
}

/// Finds a genre or game mode by IGDB id, then by name, or creates it.
async fn tag_id(
    tx: &mut Transaction<'static, Sqlite>,
    kind: TagKind,
    tag: &Tag<'_>,
) -> sqlx::Result<i64> {
    let (table, column) = (kind.table().name(), kind.igdb_column());
    let mut find = Sql::new("SELECT id FROM ");
    find.push(table)
        .push(" WHERE ")
        .push(column)
        .push(" = ")
        .push_bind(tag.igdb_id)
        .push(" UNION ALL SELECT id FROM ")
        .push(table)
        .push(" WHERE name = ")
        .push_bind(tag.name.to_owned())
        .push(" AND ")
        .push(column)
        .push(" IS NULL LIMIT 1");
    let found: Option<i64> = find.build_query_scalar().fetch_optional(&mut **tx).await?;
    if let Some(id) = found {
        let mut link = Sql::new("UPDATE ");
        link.push(table)
            .push(" SET ")
            .push(column)
            .push(" = ")
            .push_bind(tag.igdb_id)
            .push(" WHERE id = ")
            .push_bind(id);
        link.build().execute(&mut **tx).await?;
        return Ok(id);
    }
    let slug = slugs::free(tx, kind.table(), tag.slug_from).await?;
    let mut insert = Sql::new("INSERT INTO ");
    insert
        .push(table)
        .push(" (")
        .push(column)
        .push(", name, slug) VALUES (")
        .push_bind(tag.igdb_id)
        .push(", ")
        .push_bind(tag.name.to_owned())
        .push(", ")
        .push_bind(slug)
        .push(") RETURNING id");
    insert.build_query_scalar().fetch_one(&mut **tx).await
}

/// Games with no IGDB link, by id.
pub async fn unlinked(pool: &SqlitePool) -> sqlx::Result<Vec<(i64, String)>> {
    sqlx::query_as("SELECT id, title FROM games WHERE igdb_id IS NULL ORDER BY id")
        .fetch_all(pool)
        .await
}

pub async fn unlinked_count(pool: &SqlitePool) -> sqlx::Result<i64> {
    sqlx::query_scalar("SELECT COUNT(*) FROM games WHERE igdb_id IS NULL")
        .fetch_one(pool)
        .await
}

/// `(game id, IGDB id)` of the linked games, by game id.
pub async fn linked(pool: &SqlitePool) -> sqlx::Result<Vec<(i64, i64)>> {
    sqlx::query_as("SELECT id, igdb_id FROM games WHERE igdb_id IS NOT NULL ORDER BY id")
        .fetch_all(pool)
        .await
}

/// `(IGDB id, game id)` of the games linked to these IGDB ids.
pub async fn games_for(pool: &SqlitePool, igdb_ids: &[i64]) -> sqlx::Result<Vec<(i64, i64)>> {
    let ids = serde_json::to_string(igdb_ids).expect("a list of numbers serializes");
    sqlx::query_as(
        "SELECT igdb_id, id FROM games WHERE igdb_id IN (SELECT value FROM json_each(?))",
    )
    .bind(ids)
    .fetch_all(pool)
    .await
}

/// The game's IGDB id. `RowNotFound` when there is no such game.
pub async fn igdb_id(pool: &SqlitePool, game_id: i64) -> sqlx::Result<Option<i64>> {
    sqlx::query_scalar!("SELECT igdb_id FROM games WHERE id = ?", game_id)
        .fetch_one(pool)
        .await
}

/// IGDB platform ids of the game's releases.
pub async fn igdb_platforms(pool: &SqlitePool, game_id: i64) -> sqlx::Result<Vec<i64>> {
    sqlx::query_scalar(
        "SELECT DISTINCT p.igdb_platform_id FROM releases r JOIN platforms p ON p.id = r.platform_id
         WHERE r.game_id = ? AND p.igdb_platform_id IS NOT NULL",
    )
    .bind(game_id)
    .fetch_all(pool)
    .await
}

/// Unlinked games not skipped from review, after `after`, by id.
pub async fn to_review(
    pool: &SqlitePool,
    after: i64,
    limit: i64,
) -> sqlx::Result<Vec<(i64, String)>> {
    let games = sqlx::query!(
        r#"SELECT id AS "id!", title FROM games
           WHERE igdb_id IS NULL AND igdb_skipped = 0 AND id > ?
           ORDER BY id LIMIT ?"#,
        after,
        limit
    )
    .fetch_all(pool)
    .await?;
    Ok(games.into_iter().map(|g| (g.id, g.title)).collect())
}

/// Names of the platforms the game has releases on, in platform order.
pub async fn platform_names(pool: &SqlitePool, game_id: i64) -> sqlx::Result<Vec<String>> {
    sqlx::query_scalar!(
        r#"SELECT p.name FROM platforms p
           WHERE EXISTS (SELECT 1 FROM releases r WHERE r.platform_id = p.id AND r.game_id = ?)
           ORDER BY p.sort_order, p.name"#,
        game_id
    )
    .fetch_all(pool)
    .await
}

/// Leaves the game out of match review. `NotFound` when there is no such game.
pub async fn skip(pool: &SqlitePool, game_id: i64) -> ApiResult<()> {
    let result = sqlx::query!("UPDATE games SET igdb_skipped = 1 WHERE id = ?", game_id)
        .execute(pool)
        .await?;
    super::found(result.rows_affected())
}
