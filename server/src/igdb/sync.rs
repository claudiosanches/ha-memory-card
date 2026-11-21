//! Background sync: links unlinked games by exact title, then refreshes every
//! linked game. One run at a time; progress is read through `SyncStatus`.

use std::sync::Arc;

use serde::Serialize;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use utoipa::ToSchema;

use super::{GameData, Igdb, store};
use crate::AppState;
use crate::error::{ApiError, ApiResult};

const BATCH: usize = 100;

#[derive(Debug, Clone, Default, Serialize, ToSchema)]
pub struct SyncStatus {
    pub running: bool,
    pub total: usize,
    pub done: usize,
    /// Unlinked games matched by exact title.
    pub linked: usize,
    /// Unlinked games without exactly one match; link them by hand.
    pub unmatched: usize,
    /// Linked games refreshed.
    pub updated: usize,
    pub failed: usize,
    pub finished_at: Option<String>,
}

pub fn start(state: &AppState) -> ApiResult<SyncStatus> {
    let igdb = state
        .igdb
        .clone()
        .ok_or(ApiError::Unavailable("igdb_not_configured"))?;
    let started = {
        let mut status = state.sync.lock().expect("sync status lock");
        if status.running {
            return Err(ApiError::Conflict("sync_running"));
        }
        *status = SyncStatus {
            running: true,
            ..SyncStatus::default()
        };
        status.clone()
    };
    let state = state.clone();
    tokio::spawn(async move {
        if let Err(err) = run(&state, &igdb).await {
            tracing::error!(error = %err, "IGDB sync stopped");
        }
        let mut status = state.sync.lock().expect("sync status lock");
        status.running = false;
        status.finished_at = OffsetDateTime::now_utc().format(&Rfc3339).ok();
        tracing::info!(
            linked = status.linked,
            updated = status.updated,
            unmatched = status.unmatched,
            failed = status.failed,
            "IGDB sync finished"
        );
    });
    Ok(started)
}

fn update(state: &AppState, change: impl FnOnce(&mut SyncStatus)) {
    change(&mut state.sync.lock().expect("sync status lock"));
}

async fn run(state: &AppState, igdb: &Arc<Igdb>) -> sqlx::Result<()> {
    let unlinked: Vec<(i64, String)> =
        sqlx::query_as("SELECT id, title FROM games WHERE igdb_id IS NULL ORDER BY id")
            .fetch_all(&state.db)
            .await?;
    let linked: Vec<(i64, i64)> =
        sqlx::query_as("SELECT id, igdb_id FROM games WHERE igdb_id IS NOT NULL ORDER BY id")
            .fetch_all(&state.db)
            .await?;
    update(state, |s| s.total = unlinked.len() + linked.len());

    for (game_id, title) in unlinked {
        let outcome = match_title(state, igdb, game_id, &title).await;
        update(state, |s| {
            s.done += 1;
            match outcome {
                Ok(true) => s.linked += 1,
                Ok(false) => s.unmatched += 1,
                Err(err) => {
                    tracing::warn!(game_id, error = %err, "IGDB match failed");
                    s.failed += 1;
                }
            }
        });
    }

    for chunk in linked.chunks(BATCH) {
        let ids: Vec<i64> = chunk.iter().map(|(_, igdb_id)| *igdb_id).collect();
        let found = match igdb.games(&ids).await {
            Ok(found) => found,
            Err(err) => {
                tracing::warn!(error = %err, "IGDB refresh failed");
                update(state, |s| {
                    s.done += chunk.len();
                    s.failed += chunk.len();
                });
                continue;
            }
        };
        for (game_id, igdb_id) in chunk {
            let result = match found.iter().find(|g| g.id == *igdb_id) {
                Some(data) => {
                    store::apply(
                        &state.db,
                        &state.config.covers_dir(),
                        igdb,
                        *game_id,
                        data,
                        false,
                    )
                    .await
                }
                None => Err(ApiError::NotFound),
            };
            update(state, |s| {
                s.done += 1;
                match result {
                    Ok(()) => s.updated += 1,
                    Err(err) => {
                        tracing::warn!(game_id, error = %err, "IGDB refresh failed");
                        s.failed += 1;
                    }
                }
            });
        }
    }
    Ok(())
}

/// Links the game when IGDB has exactly one game with its title, or exactly
/// one of those is on a platform the game has a release for.
async fn match_title(state: &AppState, igdb: &Igdb, game_id: i64, title: &str) -> ApiResult<bool> {
    let candidates = igdb.named(title).await?;
    let platforms: Vec<i64> = sqlx::query_scalar(
        "SELECT DISTINCT p.igdb_platform_id FROM releases r JOIN platforms p ON p.id = r.platform_id
         WHERE r.game_id = ? AND p.igdb_platform_id IS NOT NULL",
    )
    .bind(game_id)
    .fetch_all(&state.db)
    .await?;
    let Some(data) = choose(candidates, &platforms) else {
        return Ok(false);
    };
    match store::apply(
        &state.db,
        &state.config.covers_dir(),
        igdb,
        game_id,
        &data,
        false,
    )
    .await
    {
        Ok(()) => Ok(true),
        Err(ApiError::Conflict(_)) => Ok(false),
        Err(err) => Err(err),
    }
}

fn choose(mut candidates: Vec<GameData>, platforms: &[i64]) -> Option<GameData> {
    if candidates.len() > 1 {
        candidates.retain(|g| g.platforms.iter().any(|p| platforms.contains(p)));
    }
    if candidates.len() == 1 {
        candidates.pop()
    } else {
        None
    }
}
