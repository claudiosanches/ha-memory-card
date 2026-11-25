//! Each person's play status for the games they keep track of.

use sqlx::SqlitePool;

use super::{play_statuses, users};
use crate::error::{ApiError, ApiResult};

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
            .execute(&mut *tx)
            .await?;
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
