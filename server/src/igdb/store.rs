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

    let cover = match data.cover.as_ref().map(|c| c.image_id.as_str()) {
        Some(image_id) if is_image_id(image_id) => {
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
    let releases: Vec<(i64, String, Option<i64>, Option<String>)> = sqlx::query_as(
        "SELECT r.id, r.region, p.igdb_platform_id, r.cover_source
         FROM releases r JOIN platforms p ON p.id = r.platform_id WHERE r.game_id = ?",
    )
    .bind(game_id)
    .fetch_all(pool)
    .await?;
    // IGDB owns regional covers: each one is replaced or cleared. Uploads stay.
    let mut release_covers = Vec::new();
    for (release_id, region, _, source) in &releases {
        if source.as_deref() == Some("upload") {
            continue;
        }
        let path = match local_cover(data, region) {
            Some(image_id) => {
                match save_cover(covers_dir, igdb, Owner::Release(*release_id), image_id).await {
                    Ok(path) => Some(path),
                    Err(err) => {
                        tracing::warn!(release_id, error = %err, "cover download failed");
                        continue;
                    }
                }
            }
            None => None,
        };
        release_covers.push((*release_id, path));
    }

    // The stored value wins unless overwriting; a missing IGDB value never clears one.
    let fill = |column: &str, param: u8| {
        if overwrite {
            format!("{column} = COALESCE(?{param}, {column})")
        } else {
            format!("{column} = COALESCE({column}, ?{param})")
        }
    };

    let mut tx = write_tx(pool).await?;
    let game_counts = data
        .multiplayer_modes
        .iter()
        .find(|m| m.platform.is_none())
        .map(counts)
        .unwrap_or_default();
    let (developers, publishers) = companies(data);
    let title = Some(data.name.trim()).filter(|t| overwrite && !t.is_empty());
    sqlx::query(sqlx::AssertSqlSafe(format!(
        "UPDATE games SET igdb_id = ?1, {summary},
            first_release_date = COALESCE(?3, first_release_date),
            developers = ?4, publishers = ?5,
            {local_max}, {online_max}, {local_coop}, {online_coop},
            cover_path = COALESCE(?10, cover_path),
            cover_source = CASE WHEN ?10 IS NULL THEN cover_source ELSE 'igdb' END,
            title = COALESCE(?12, title), sort_title = COALESCE(?13, sort_title),
            igdb_synced_at = {NOW}, updated_at = {NOW}
         WHERE id = ?11",
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
    .bind(&cover)
    .bind(game_id)
    .bind(title)
    .bind(title.map(sort_title))
    .execute(&mut *tx)
    .await
    .map_err(|err| ApiError::unique(err, "igdb_linked"))?;

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
    for (release_id, region, platform, _) in &releases {
        let (release_id, platform) = (*release_id, *platform);
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
            .bind(platform.and_then(|p| release_date(data, p, region)))
            .bind(local_title(data, region))
            .bind(release_id)
            .execute(&mut *tx)
            .await?;
    }

    for (release_id, path) in &release_covers {
        sqlx::query(
            "UPDATE releases SET cover_path = ?1,
                cover_source = CASE WHEN ?1 IS NULL THEN NULL ELSE 'igdb_localization' END
             WHERE id = ?2 AND cover_source IS NOT 'upload'",
        )
        .bind(path)
        .bind(release_id)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    if let Some(kept) = &cover {
        covers::keep_only(covers_dir, Owner::Game(game_id), Some(kept)).await;
    }
    for (release_id, path) in &release_covers {
        covers::keep_only(covers_dir, Owner::Release(*release_id), path.as_deref()).await;
    }
    Ok(())
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

/// The regional title, when it differs from the game's name.
fn local_title(data: &GameData, region: &str) -> Option<String> {
    localization(data, region)
        .and_then(|l| l.name.as_deref())
        .map(str::trim)
        .filter(|name| !name.is_empty() && *name != data.name)
        .map(str::to_owned)
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
    fn dates_and_image_ids() {
        assert_eq!(date(1509062400).as_deref(), Some("2017-10-27"));
        assert!(is_image_id("co1mxf"));
        assert!(!is_image_id("../x"));
        assert!(!is_image_id(""));
    }
}
