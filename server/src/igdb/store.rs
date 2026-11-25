//! Writes IGDB data onto a game. IGDB owns the first release date, companies,
//! genres and cover. Title, summary, game modes, player counts and release
//! titles and dates are only filled when empty, so hand edits survive a
//! refresh, unless `overwrite` replaces them where IGDB has a value. Covers
//! for Japan, Korea and Europe releases come from IGDB localizations.

use std::path::Path;

use time::OffsetDateTime;
use time::macros::format_description;

use super::{GameData, Igdb, Localization, MultiplayerMode};
use crate::covers::{self, Owner};
use crate::db::Pool;
use crate::db::games::PlayerCounts;
use crate::db::igdb::{self as db, Covers, Expected, GameUpdate, Release, ReleaseUpdate, Tag};
use crate::domain::sort_title;
use crate::error::{ApiError, ApiResult};

/// `cover_source` of an uploaded cover. IGDB never replaces one.
const UPLOAD: &str = "upload";

/// IGDB `release_region` for worldwide releases, the fallback for release dates.
const WORLDWIDE: i64 = 8;

/// Links `game_id` to `data.id` and stores its metadata. `Conflict("igdb_linked")`
/// when another game already has that IGDB id.
pub async fn apply(
    pool: &Pool,
    covers_dir: &Path,
    igdb: &Igdb,
    game_id: i64,
    data: &GameData,
    overwrite: bool,
    expected: Expected,
) -> ApiResult<()> {
    let _cover = covers::lock(game_id).await;
    // Checked before downloading covers; the unique index still decides.
    if db::linked_elsewhere(pool, data.id, game_id).await? {
        return Err(ApiError::Conflict("igdb_linked"));
    }

    let releases = db::releases(pool, game_id).await?;
    let covers = download_covers(pool, covers_dir, igdb, game_id, data, &releases).await?;
    let (developers, publishers) = companies(data);
    let igdb_title = Some(data.name.trim()).filter(|t| !t.is_empty());
    let update = GameUpdate {
        igdb_id: data.id,
        summary: data.summary.as_deref().filter(|s| !s.trim().is_empty()),
        first_release_date: data.first_release_date.and_then(date),
        developers,
        publishers,
        counts: game_counts(&data.multiplayer_modes),
        title: igdb_title
            .filter(|_| overwrite)
            .map(|title| (title, sort_title(title))),
        igdb_title,
        names: data
            .alternative_names
            .iter()
            .map(|alternative| {
                let comment = alternative.comment.as_deref().map(str::trim);
                (alternative.name.trim(), comment.filter(|c| !c.is_empty()))
            })
            .filter(|(name, _)| !name.is_empty())
            .collect(),
        links: data.links(),
        genres: data
            .genres
            .iter()
            .map(|genre| Tag {
                igdb_id: genre.id,
                name: &genre.name,
                slug_from: &genre.name,
            })
            .collect(),
        game_modes: data
            .game_modes
            .iter()
            .map(|mode| Tag {
                igdb_id: mode.id,
                name: &mode.name,
                slug_from: mode.slug.as_deref().unwrap_or(&mode.name),
            })
            .collect(),
        replace_modes: overwrite && !data.game_modes.is_empty(),
        releases: releases
            .iter()
            .map(|release| {
                let platform = release.igdb_platform_id;
                ReleaseUpdate {
                    id: release.id,
                    counts: platform
                        .and_then(|p| {
                            data.multiplayer_modes
                                .iter()
                                .find(|m| m.platform == Some(p))
                        })
                        .map(counts)
                        .unwrap_or_default(),
                    release_date: platform.and_then(|p| release_date(data, p, &release.region)),
                    title: release_title(data, &release.region),
                }
            })
            .collect(),
    };
    let changed = match db::write(pool, game_id, &update, overwrite, &covers, expected).await {
        Ok(changed) => changed,
        Err(err @ ApiError::Conflict("igdb_changed")) => {
            // The covers just downloaded were never stored.
            clean_up(covers_dir, db::stored_covers(pool, game_id).await?).await;
            return Err(err);
        }
        Err(err) => return Err(err),
    };
    clean_up(covers_dir, changed).await;
    Ok(())
}

/// Downloads and stores the game's IGDB covers without touching its other data.
pub async fn refresh_covers(
    pool: &Pool,
    covers_dir: &Path,
    igdb: &Igdb,
    game_id: i64,
    data: &GameData,
) -> ApiResult<()> {
    let _cover = covers::lock(game_id).await;
    let releases = db::releases(pool, game_id).await?;
    let covers = download_covers(pool, covers_dir, igdb, game_id, data, &releases).await?;
    let changed = db::write_covers(pool, game_id, &covers).await?;
    clean_up(covers_dir, changed).await;
    Ok(())
}

/// Downloads the covers; uploads are skipped.
async fn download_covers(
    pool: &Pool,
    covers_dir: &Path,
    igdb: &Igdb,
    game_id: i64,
    data: &GameData,
    releases: &[Release],
) -> ApiResult<Covers> {
    let source = db::cover_source(pool, game_id)
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

async fn clean_up(covers_dir: &Path, changed: Vec<(Owner, Option<String>)>) {
    for (owner, kept) in changed {
        covers::keep_only(covers_dir, owner, kept.as_deref()).await;
    }
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
    let image = crate::images::clean(&bytes)
        .ok()
        .filter(|image| image.extension == "jpg")
        .ok_or("IGDB sent a cover that is not a JPEG of a usable size")?;
    Ok(covers::write(covers_dir, owner, &file_name, &image.bytes).await?)
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
