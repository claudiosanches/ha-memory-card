//! Background sync: links unlinked games by exact title, then refreshes every
//! linked game. One run at a time; progress is read through `SyncStatus`.

use std::sync::Arc;

use serde::Serialize;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use utoipa::ToSchema;

use super::{GameData, Igdb, store};
use crate::AppState;
use crate::db::igdb::{self as db, Expected};
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
        // Its own task, so a panic still ends the run below.
        let sync = tokio::spawn({
            let state = state.clone();
            async move { run(&state, &igdb).await }
        });
        match sync.await {
            Ok(Ok(())) => {}
            Ok(Err(err)) => tracing::error!(error = %err, "IGDB sync stopped"),
            Err(err) => tracing::error!(error = %err, "IGDB sync failed"),
        }
        state.changes.announce();
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
    let unlinked = db::unlinked(&state.db).await?;
    let linked = db::linked(&state.db).await?;
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
                        Expected::Link(Some(*igdb_id)),
                    )
                    .await
                }
                None => Err(ApiError::NotFound),
            };
            update(state, |s| {
                s.done += 1;
                match result {
                    Ok(()) => s.updated += 1,
                    // Relinked while the sync ran; the new link stands.
                    Err(ApiError::Conflict("igdb_changed")) => {}
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
    let platforms = db::igdb_platforms(&state.db, game_id).await?;
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
        Expected::Link(None),
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
