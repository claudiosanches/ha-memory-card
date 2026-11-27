//! Each person's play status for the games they keep track of, and the
//! history of the statuses they gave.

use sqlx::{SqliteConnection, SqlitePool};

use super::{play_statuses, users};
use crate::error::{ApiError, ApiResult};

/// Where a status change came from, kept in the history.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// Set in Memory Card, or by logging a session.
    Manual,
    /// An NFC tag scan.
    Scan,
    /// A CSV import.
    Csv,
}

impl Source {
    fn as_str(self) -> &'static str {
        match self {
            Self::Manual => "manual",
            Self::Scan => "scan",
            Self::Csv => "csv",
        }
    }
}

/// Which status [`put`] replaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Replace {
    Any,
    /// Only one that counts as backlog.
    Backlog,
}

/// Gives `user`'s game `status`, or leaves it when `replace` keeps the one
/// they have, and adds the change to the history.
pub(super) async fn put(
    conn: &mut SqliteConnection,
    user: &str,
    game_id: i64,
    status: &str,
    source: Source,
    replace: Replace,
) -> sqlx::Result<()> {
    let changed = match replace {
        Replace::Any => {
            sqlx::query!(
                "INSERT INTO play_progress (user_id, game_id, play_status) VALUES (?, ?, ?)
                 ON CONFLICT (user_id, game_id) DO UPDATE
                 SET play_status = excluded.play_status,
                     updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
                 WHERE play_status IS NOT excluded.play_status",
                user,
                game_id,
                status
            )
            .execute(&mut *conn)
            .await?
        }
        Replace::Backlog => {
            sqlx::query!(
                "INSERT INTO play_progress (user_id, game_id, play_status) VALUES (?, ?, ?)
                 ON CONFLICT (user_id, game_id) DO UPDATE
                 SET play_status = excluded.play_status,
                     updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
                 WHERE (SELECT counts_as FROM play_statuses WHERE key = play_progress.play_status)
                       = 'backlog'",
                user,
                game_id,
                status
            )
            .execute(&mut *conn)
            .await?
        }
    }
    .rows_affected();
    if changed == 0 {
        return Ok(());
    }
    let source = source.as_str();
    // A game keeps one row a day per person, so second thoughts such as a
    // status picked by mistake count as the last one, and the history stays
    // bounded however often a status flips.
    let replaced = sqlx::query!(
        "UPDATE play_status_changes
         SET play_status = ?1,
             counts_as = (SELECT counts_as FROM play_statuses WHERE key = ?1),
             source = ?2,
             changed_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE id = (SELECT id FROM play_status_changes
                     WHERE user_id = ?3 AND game_id = ?4 ORDER BY id DESC LIMIT 1)
           AND date(changed_at) = date('now')",
        status,
        source,
        user,
        game_id
    )
    .execute(&mut *conn)
    .await?
    .rows_affected();
    if replaced == 0 {
        sqlx::query!(
            "INSERT INTO play_status_changes (user_id, game_id, play_status, counts_as, source)
             SELECT ?1, ?2, key, counts_as, ?3 FROM play_statuses WHERE key = ?4",
            user,
            game_id,
            source,
            status
        )
        .execute(&mut *conn)
        .await?;
    }
    Ok(())
}

/// Sets `user`'s status for the game; `None` clears it.
pub async fn set(
    pool: &SqlitePool,
    user: &str,
    game_id: i64,
    status: Option<&str>,
) -> ApiResult<()> {
    let mut tx = super::write_tx(pool).await?;
    play_statuses::ensure_exists(&mut *tx, status).await?;
    let exists = sqlx::query_scalar!(
        r#"SELECT EXISTS (SELECT 1 FROM games WHERE id = ?) AS "exists!: bool""#,
        game_id
    )
    .fetch_one(&mut *tx)
    .await?;
    if !exists {
        return Err(ApiError::NotFound);
    }
    users::ensure(&mut tx, user).await?;
    match status {
        Some(status) => {
            put(&mut tx, user, game_id, status, Source::Manual, Replace::Any).await?;
        }
        None => {
            sqlx::query!(
                "DELETE FROM play_progress WHERE user_id = ? AND game_id = ?",
                user,
                game_id
            )
            .execute(&mut *tx)
            .await?;
        }
    }
    tx.commit().await?;
    Ok(())
}

/// `user`'s status for the game, if they set one.
pub async fn get(pool: &SqlitePool, user: &str, game_id: i64) -> sqlx::Result<Option<String>> {
    sqlx::query_scalar!(
        "SELECT play_status FROM play_progress WHERE user_id = ? AND game_id = ?",
        user,
        game_id
    )
    .fetch_optional(pool)
    .await
}
