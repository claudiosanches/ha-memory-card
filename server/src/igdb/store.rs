//! Writes IGDB data onto a game. IGDB owns the first release date, companies,
//! genres and cover. Title, summary, game modes, player counts and release
//! titles and dates are only filled when empty, so hand edits survive a
//! refresh, unless `overwrite` replaces them where IGDB has a value. Covers
//! for Japan, Korea and Europe releases come from IGDB localizations.

use std::path::Path;

use sqlx::{Sqlite, SqlitePool, Transaction};
use time::OffsetDateTime;
use time::macros::format_description;

use super::{GameData, Igdb, Localization, MultiplayerMode};
use crate::covers::{self, Owner};
use crate::db::games::PlayerCounts;
use crate::db::platforms::slugify;
use crate::db::write_tx;
use crate::domain::sort_title;
use crate::error::{ApiError, ApiResult};

const NOW: &str = "strftime('%Y-%m-%dT%H:%M:%fZ', 'now')";

/// `cover_source` of an uploaded cover. IGDB never replaces one.
const UPLOAD: &str = "upload";

/// IGDB `release_region` for worldwide releases, the fallback for release dates.
const WORLDWIDE: i64 = 8;

/// Links `game_id` to `data.id` and stores its metadata. `Conflict("igdb_linked")`
/// when another game already has that IGDB id.
pub async fn apply(
    pool: &SqlitePool,
    covers_dir: &Path,
    igdb: &Igdb,
    game_id: i64,
    data: &GameData,
    overwrite: bool,
) -> ApiResult<()> {
    // Checked before downloading covers; the unique index still decides.
    let taken: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM games WHERE igdb_id = ? AND id != ?)")
            .bind(data.id)
            .bind(game_id)
            .fetch_one(pool)
            .await?;
    if taken {
        return Err(ApiError::Conflict("igdb_linked"));
    }

    let releases = releases(pool, game_id).await?;
    let covers = download_covers(pool, covers_dir, igdb, game_id, data, &releases).await?;

    // The stored value wins unless overwriting; a missing IGDB value never clears one.
    let fill = |column: &'static str, param: u8| {
        if overwrite {
            format!("{column} = COALESCE(?{param}, {column})")
        } else {
            format!("{column} = COALESCE({column}, ?{param})")
        }
    };

    let mut tx = write_tx(pool).await?;
    let game_counts = game_counts(&data.multiplayer_modes);
    let (developers, publishers) = companies(data);
    let igdb_title = Some(data.name.trim()).filter(|t| !t.is_empty());
    let title = igdb_title.filter(|_| overwrite);
    sqlx::query(sqlx::AssertSqlSafe(format!(
        "UPDATE games SET igdb_id = ?1, {summary},
            first_release_date = COALESCE(?3, first_release_date),
            developers = ?4, publishers = ?5,
            {local_max}, {online_max}, {local_coop}, {online_coop},
            title = COALESCE(?11, title), sort_title = COALESCE(?12, sort_title),
            igdb_title = COALESCE(?13, igdb_title),
            igdb_synced_at = {NOW}, updated_at = {NOW}
         WHERE id = ?10",
        summary = fill("summary", 2),
        local_max = fill("local_max_players", 6),
        online_max = fill("online_max_players", 7),
        local_coop = fill("local_coop", 8),
        online_coop = fill("online_coop", 9),
    )))
    .bind(data.id)
    .bind(data.summary.as_deref().filter(|s| !s.trim().is_empty()))
    .bind(data.first_release_date.and_then(date))
    .bind(serde_json::to_string(&developers).expect("strings serialize"))
    .bind(serde_json::to_string(&publishers).expect("strings serialize"))
    .bind(game_counts.local_max_players)
    .bind(game_counts.online_max_players)
    .bind(game_counts.local_coop)
    .bind(game_counts.online_coop)
    .bind(game_id)
    .bind(title)
    .bind(title.map(sort_title))
    .bind(igdb_title)
    .execute(&mut *tx)
    .await
    .map_err(|err| ApiError::unique(err, "igdb_linked"))?;

    sqlx::query("DELETE FROM game_names WHERE game_id = ? AND source = 'igdb'")
        .bind(game_id)
        .execute(&mut *tx)
        .await?;
    for alternative in &data.alternative_names {
        let name = alternative.name.trim();
        if name.is_empty() {
            continue;
        }
        // A name added by hand stays as it is.
        sqlx::query(
            "INSERT OR IGNORE INTO game_names (game_id, name, source, comment) VALUES (?, ?, 'igdb', ?)",
        )
        .bind(game_id)
        .bind(name)
        .bind(alternative.comment.as_deref().map(str::trim).filter(|c| !c.is_empty()))
        .execute(&mut *tx)
        .await?;
    }

    sqlx::query("DELETE FROM game_links WHERE game_id = ?")
        .bind(game_id)
        .execute(&mut *tx)
        .await?;
    for (igdb_id, kind) in data.links() {
        sqlx::query("INSERT OR IGNORE INTO game_links (game_id, igdb_id, kind) VALUES (?, ?, ?)")
            .bind(game_id)
            .bind(igdb_id)
            .bind(kind)
            .execute(&mut *tx)
            .await?;
    }

    sqlx::query("DELETE FROM game_genres WHERE game_id = ?")
        .bind(game_id)
        .execute(&mut *tx)
        .await?;
    for genre in &data.genres {
        let id = tag_id(
            &mut tx,
            "genres",
            "igdb_genre_id",
            genre.id,
            &genre.name,
            None,
        )
        .await?;
        sqlx::query("INSERT OR IGNORE INTO game_genres (game_id, genre_id) VALUES (?, ?)")
            .bind(game_id)
            .bind(id)
            .execute(&mut *tx)
            .await?;
    }

    let replace_modes = overwrite && !data.game_modes.is_empty();
    if replace_modes {
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
        for mode in &data.game_modes {
            let slug = mode.slug.clone().unwrap_or_else(|| slugify(&mode.name));
            let id = tag_id(
                &mut tx,
                "game_modes",
                "igdb_game_mode_id",
                mode.id,
                &mode.name,
                Some(&slug),
            )
            .await?;
            sqlx::query(
                "INSERT OR IGNORE INTO game_game_modes (game_id, game_mode_id) VALUES (?, ?)",
            )
            .bind(game_id)
            .bind(id)
            .execute(&mut *tx)
            .await?;
        }
    }

    let release_sql = format!(
        "UPDATE releases SET {}, {}, {}, {}, {}, {}, updated_at = {NOW} WHERE id = ?7",
        fill("local_max_players", 1),
        fill("online_max_players", 2),
        fill("local_coop", 3),
        fill("online_coop", 4),
        fill("release_date", 5),
        fill("title", 6),
    );
    for release in &releases {
        let platform = release.igdb_platform_id;
        let c = platform
            .and_then(|p| {
                data.multiplayer_modes
                    .iter()
                    .find(|m| m.platform == Some(p))
            })
            .map(counts)
            .unwrap_or_default();
        sqlx::query(sqlx::AssertSqlSafe(release_sql.clone()))
            .bind(c.local_max_players)
            .bind(c.online_max_players)
            .bind(c.local_coop)
            .bind(c.online_coop)
            .bind(platform.and_then(|p| release_date(data, p, &release.region)))
            .bind(release_title(data, &release.region))
            .bind(release.id)
            .execute(&mut *tx)
            .await?;
    }

    let changed = write_covers(&mut tx, game_id, &covers).await?;
    tx.commit().await?;
    clean_up(covers_dir, changed).await;
    Ok(())
}

/// Downloads and stores the game's IGDB covers without touching its other data.
pub async fn refresh_covers(
    pool: &SqlitePool,
    covers_dir: &Path,
    igdb: &Igdb,
    game_id: i64,
    data: &GameData,
) -> ApiResult<()> {
    let releases = releases(pool, game_id).await?;
    let covers = download_covers(pool, covers_dir, igdb, game_id, data, &releases).await?;
    let mut tx = write_tx(pool).await?;
    let changed = write_covers(&mut tx, game_id, &covers).await?;
    tx.commit().await?;
    clean_up(covers_dir, changed).await;
    Ok(())
}

#[derive(sqlx::FromRow)]
struct ReleaseRow {
    id: i64,
    region: String,
    igdb_platform_id: Option<i64>,
    cover_source: Option<String>,
}

async fn releases(pool: &SqlitePool, game_id: i64) -> sqlx::Result<Vec<ReleaseRow>> {
    sqlx::query_as(
        "SELECT r.id, r.region, p.igdb_platform_id, r.cover_source
         FROM releases r JOIN platforms p ON p.id = r.platform_id WHERE r.game_id = ?",
    )
    .bind(game_id)
    .fetch_all(pool)
    .await
}

/// Downloaded covers, relative to the covers directory. Uploads are skipped.
struct Covers {
    /// `None` keeps the stored cover.
    game: Option<String>,
    /// IGDB owns regional covers, so `None` clears one.
    releases: Vec<(i64, Option<String>)>,
}

async fn download_covers(
    pool: &SqlitePool,
    covers_dir: &Path,
    igdb: &Igdb,
    game_id: i64,
    data: &GameData,
    releases: &[ReleaseRow],
) -> ApiResult<Covers> {
    let source: Option<String> = sqlx::query_scalar("SELECT cover_source FROM games WHERE id = ?")
        .bind(game_id)
        .fetch_optional(pool)
        .await?
        .ok_or(ApiError::NotFound)?;
    let game = match data.cover.as_ref().map(|c| c.image_id.as_str()) {
        Some(image_id) if is_image_id(image_id) && source.as_deref() != Some(UPLOAD) => {
            match save_cover(covers_dir, igdb, Owner::Game(game_id), image_id).await {
                Ok(path) => Some(path),
                Err(err) => {
                    tracing::warn!(game_id, error = %err, "cover download failed");
                    None
                }
            }
        }
        _ => None,
    };

    let mut release_covers = Vec::new();
    for release in releases {
        if release.cover_source.as_deref() == Some(UPLOAD) {
            continue;
        }
        let path = match local_cover(data, &release.region) {
            Some(image_id) => {
                match save_cover(covers_dir, igdb, Owner::Release(release.id), image_id).await {
                    Ok(path) => Some(path),
                    Err(err) => {
                        tracing::warn!(release_id = release.id, error = %err, "cover download failed");
                        continue;
                    }
                }
            }
            None => None,
        };
        release_covers.push((release.id, path));
    }
    Ok(Covers {
        game,
        releases: release_covers,
    })
}

/// Stores the covers unless an upload arrived meanwhile, and returns what changed.
async fn write_covers(
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
             WHERE id = ?2 AND cover_source IS NOT 'upload'",
        )
        .bind(path)
        .bind(release_id)
        .execute(&mut **tx)
        .await?;
        if result.rows_affected() > 0 {
            changed.push((Owner::Release(*release_id), path.clone()));
        }
    }
    Ok(changed)
}

async fn clean_up(covers_dir: &Path, changed: Vec<(Owner, Option<String>)>) {
    for (owner, kept) in changed {
        covers::keep_only(covers_dir, owner, kept.as_deref()).await;
    }
}

/// Finds a genre or game mode by IGDB id, then by name, or creates it.
async fn tag_id(
    tx: &mut Transaction<'static, Sqlite>,
    table: &'static str,
    igdb_column: &'static str,
    igdb_id: i64,
    name: &str,
    slug: Option<&str>,
) -> sqlx::Result<i64> {
    let found: Option<i64> = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
        "SELECT id FROM {table} WHERE {igdb_column} = ?1
         UNION ALL SELECT id FROM {table} WHERE name = ?2 AND {igdb_column} IS NULL
         LIMIT 1"
    )))
    .bind(igdb_id)
    .bind(name)
    .fetch_optional(&mut **tx)
    .await?;
    if let Some(id) = found {
        sqlx::query(sqlx::AssertSqlSafe(format!(
            "UPDATE {table} SET {igdb_column} = ? WHERE id = ?"
        )))
        .bind(igdb_id)
        .bind(id)
        .execute(&mut **tx)
        .await?;
        return Ok(id);
    }
    match slug {
        Some(slug) => sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
            "INSERT INTO {table} ({igdb_column}, name, slug) VALUES (?, ?, ?) RETURNING id"
        )))
        .bind(igdb_id)
        .bind(name)
        .bind(slug),
        None => sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
            "INSERT INTO {table} ({igdb_column}, name) VALUES (?, ?) RETURNING id"
        )))
        .bind(igdb_id)
        .bind(name),
    }
    .fetch_one(&mut **tx)
    .await
}

/// Earliest date for the platform in the release's region, else worldwide.
fn release_date(data: &GameData, platform: i64, region: &str) -> Option<String> {
    let earliest = |wanted: i64| {
        data.release_dates
            .iter()
            .filter(|d| d.platform == Some(platform) && d.release_region == Some(wanted))
            .filter_map(|d| d.date)
            .min()
    };
    release_region(region)
        .and_then(earliest)
        .or_else(|| earliest(WORLDWIDE))
        .and_then(date)
}

fn release_region(region: &str) -> Option<i64> {
    Some(match region {
        "eu" => 1,
        "na" => 2,
        "au" => 3,
        "nz" => 4,
        "jp" => 5,
        "cn" => 6,
        "asia" => 7,
        "ww" => 8,
        "kr" => 9,
        "br" => 10,
        _ => return None,
    })
}

/// IGDB has localizations for Korea (2), Japan (3) and Europe (4).
fn localization<'a>(data: &'a GameData, region: &str) -> Option<&'a Localization> {
    let wanted = match region {
        "kr" => 2,
        "jp" => 3,
        "eu" => 4,
        _ => return None,
    };
    data.game_localizations
        .iter()
        .find(|l| l.region == Some(wanted))
}

/// The regional name when IGDB has one. Otherwise IGDB's name, except in
/// Asian regions, where that English name is not what the box says.
fn release_title(data: &GameData, region: &str) -> Option<String> {
    let regional = localization(data, region)
        .and_then(|l| l.name.as_deref())
        .map(str::trim)
        .filter(|name| !name.is_empty());
    let fallback = match region {
        "jp" | "kr" | "cn" | "asia" => None,
        _ => Some(data.name.trim()).filter(|name| !name.is_empty()),
    };
    regional.or(fallback).map(str::to_owned)
}

/// The regional cover's image id, when it differs from the game's cover.
fn local_cover<'a>(data: &'a GameData, region: &str) -> Option<&'a str> {
    let generic = data.cover.as_ref().map(|c| c.image_id.as_str());
    localization(data, region)?
        .cover
        .as_ref()
        .map(|c| c.image_id.as_str())
        .filter(|id| is_image_id(id) && Some(*id) != generic)
}

fn companies(data: &GameData) -> (Vec<String>, Vec<String>) {
    let mut developers = Vec::new();
    let mut publishers = Vec::new();
    for involved in &data.involved_companies {
        let Some(company) = &involved.company else {
            continue;
        };
        if involved.developer && !developers.contains(&company.name) {
            developers.push(company.name.clone());
        }
        if involved.publisher && !publishers.contains(&company.name) {
            publishers.push(company.name.clone());
        }
    }
    (developers, publishers)
}

/// The entry without a platform, or else all entries merged, so a release on
/// another platform (a Virtual Console copy, say) still gets the game's values.
fn game_counts(modes: &[MultiplayerMode]) -> PlayerCounts {
    if let Some(mode) = modes.iter().find(|m| m.platform.is_none()) {
        return counts(mode);
    }
    modes
        .iter()
        .map(counts)
        .fold(PlayerCounts::default(), |a, b| PlayerCounts {
            local_max_players: a.local_max_players.max(b.local_max_players),
            online_max_players: a.online_max_players.max(b.online_max_players),
            local_coop: a.local_coop.max(b.local_coop),
            online_coop: a.online_coop.max(b.online_coop),
        })
}

/// Local and online stay apart; co-op maxima count as players too. Values are
/// kept within the 1 to 99 the edit form accepts.
fn counts(mode: &MultiplayerMode) -> PlayerCounts {
    let players = |a: Option<i64>, b: Option<i64>| {
        a.into_iter()
            .chain(b)
            .max()
            .filter(|n| *n >= 1)
            .map(|n| n.min(99))
    };
    PlayerCounts {
        local_max_players: players(mode.offlinemax, mode.offlinecoopmax),
        online_max_players: players(mode.onlinemax, mode.onlinecoopmax),
        local_coop: mode.offlinecoop,
        online_coop: mode.onlinecoop,
    }
}

fn date(timestamp: i64) -> Option<String> {
    let date = OffsetDateTime::from_unix_timestamp(timestamp).ok()?.date();
    date.format(format_description!("[year]-[month]-[day]"))
        .ok()
}

/// Image ids become file names, so only plain ones are accepted.
fn is_image_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= 64 && id.bytes().all(|b| b.is_ascii_alphanumeric())
}

/// Returns the path relative to the covers directory.
async fn save_cover(
    covers_dir: &Path,
    igdb: &Igdb,
    owner: Owner,
    image_id: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let file_name = format!("{image_id}.jpg");
    let relative = owner.path(&file_name);
    if covers::exists(covers_dir, &relative).await? {
        return Ok(relative);
    }
    let bytes = igdb.cover(image_id).await?;
    Ok(covers::write(covers_dir, owner, &file_name, &bytes).await?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_keep_local_and_online_apart() {
        let mode = MultiplayerMode {
            offlinemax: Some(2),
            offlinecoopmax: Some(4),
            onlinemax: Some(500),
            offlinecoop: Some(true),
            ..Default::default()
        };
        let c = counts(&mode);
        assert_eq!(c.local_max_players, Some(4));
        assert_eq!(c.online_max_players, Some(99));
        assert_eq!(c.local_coop, Some(true));
        assert_eq!(c.online_coop, None);
        assert_eq!(
            counts(&MultiplayerMode {
                offlinemax: Some(0),
                ..Default::default()
            })
            .local_max_players,
            None
        );
    }

    #[test]
    fn game_counts_merge_platform_entries_when_none_is_generic() {
        let n64 = MultiplayerMode {
            platform: Some(4),
            offlinemax: Some(4),
            offlinecoop: Some(false),
            ..Default::default()
        };
        let wii = MultiplayerMode {
            platform: Some(5),
            offlinecoopmax: Some(2),
            offlinecoop: Some(true),
            ..Default::default()
        };
        let merged = game_counts(&[n64, wii]);
        assert_eq!(merged.local_max_players, Some(4));
        assert_eq!(merged.local_coop, Some(true));
        assert_eq!(merged.online_coop, None);

        let generic = MultiplayerMode {
            offlinemax: Some(2),
            ..Default::default()
        };
        let n64 = MultiplayerMode {
            platform: Some(4),
            offlinemax: Some(4),
            ..Default::default()
        };
        assert_eq!(game_counts(&[n64, generic]).local_max_players, Some(2));
    }

    #[test]
    fn dates_and_image_ids() {
        assert_eq!(date(1509062400).as_deref(), Some("2017-10-27"));
        assert!(is_image_id("co1mxf"));
        assert!(!is_image_id("../x"));
        assert!(!is_image_id(""));
    }
}
