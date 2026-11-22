use serde::Serialize;
use sqlx::{SqliteConnection, SqlitePool};
use utoipa::ToSchema;

use crate::error::{ApiError, ApiResult};

/// Photos kept per copy.
pub const MAX_PER_COPY: i64 = 20;

#[derive(Debug, Serialize, ToSchema)]
pub struct CopyPhoto {
    pub id: i64,
    /// Relative to the covers URL.
    pub path: String,
    pub created_at: String,
}

/// The copy's game, if the copy exists and has room for another photo.
pub async fn room_for(pool: &SqlitePool, copy_id: i64) -> ApiResult<i64> {
    room(&mut *pool.acquire().await?, copy_id).await
}

async fn room(conn: &mut SqliteConnection, copy_id: i64) -> ApiResult<i64> {
    let row = sqlx::query!(
        r#"SELECT r.game_id, (SELECT COUNT(*) FROM copy_photos WHERE copy_id = c.id) AS "photos!: i64"
           FROM copies c JOIN releases r ON r.id = c.release_id WHERE c.id = ?"#,
        copy_id
    )
    .fetch_optional(conn)
    .await?
    .ok_or(ApiError::NotFound)?;
    if row.photos >= MAX_PER_COPY {
        return Err(ApiError::Conflict("too_many_photos"));
    }
    Ok(row.game_id)
}

/// Adds the photo if the copy still has room, checked in the same write so
/// uploads at the same time cannot pass the limit. Returns the game id.
pub async fn add(pool: &SqlitePool, copy_id: i64, path: &str, by: &str) -> ApiResult<i64> {
    let mut tx = super::write_tx(pool).await?;
    let game_id = room(&mut tx, copy_id).await?;
    sqlx::query!(
        "INSERT INTO copy_photos (copy_id, path, created_by) VALUES (?, ?, ?)",
        copy_id,
        path,
        by
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(game_id)
}

/// Deletes a photo row and returns its game id and file path.
pub async fn delete(pool: &SqlitePool, id: i64) -> ApiResult<(i64, String)> {
    let row = sqlx::query!(
        r#"SELECT r.game_id, p.path FROM copy_photos p
           JOIN copies c ON c.id = p.copy_id JOIN releases r ON r.id = c.release_id
           WHERE p.id = ?"#,
        id
    )
    .fetch_optional(pool)
    .await?
    .ok_or(ApiError::NotFound)?;
    sqlx::query!("DELETE FROM copy_photos WHERE id = ?", id)
        .execute(pool)
        .await?;
    Ok((row.game_id, row.path))
}

/// Copies from `ids` that no longer exist.
pub async fn gone(pool: &SqlitePool, ids: &[i64]) -> sqlx::Result<Vec<i64>> {
    let json = serde_json::to_string(ids).expect("a list of numbers serializes");
    sqlx::query_scalar("SELECT value FROM json_each(?) WHERE value NOT IN (SELECT id FROM copies)")
        .bind(json)
        .fetch_all(pool)
        .await
}
