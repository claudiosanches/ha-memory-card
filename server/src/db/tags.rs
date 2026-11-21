use serde::Serialize;
use sqlx::SqlitePool;
use utoipa::ToSchema;

use crate::error::{ApiError, ApiResult};

#[derive(Debug, Serialize, ToSchema)]
pub struct CopyTag {
    pub tag_id: String,
    pub created_at: String,
}

/// Points `tag_id` at the copy, moving it from any other copy. Returns the copy's game.
pub async fn link(pool: &SqlitePool, tag_id: &str, copy_id: i64, by: &str) -> ApiResult<i64> {
    let game_id = sqlx::query_scalar!(
        "SELECT r.game_id FROM copies c JOIN releases r ON r.id = c.release_id WHERE c.id = ?",
        copy_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or(ApiError::NotFound)?;
    sqlx::query!(
        "INSERT INTO nfc_tags (tag_id, copy_id, created_by) VALUES (?, ?, ?)
         ON CONFLICT (tag_id) DO UPDATE SET copy_id = excluded.copy_id,
             created_by = excluded.created_by, created_at = excluded.created_at",
        tag_id,
        copy_id,
        by
    )
    .execute(pool)
    .await?;
    Ok(game_id)
}

/// Removes a tag and returns the game of the copy it pointed at.
pub async fn unlink(pool: &SqlitePool, tag_id: &str) -> ApiResult<i64> {
    let game_id = sqlx::query_scalar!(
        r#"SELECT r.game_id FROM nfc_tags t JOIN copies c ON c.id = t.copy_id
           JOIN releases r ON r.id = c.release_id WHERE t.tag_id = ?"#,
        tag_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or(ApiError::NotFound)?;
    sqlx::query!("DELETE FROM nfc_tags WHERE tag_id = ?", tag_id)
        .execute(pool)
        .await?;
    Ok(game_id)
}

pub async fn copy_for(pool: &SqlitePool, tag_id: &str) -> sqlx::Result<Option<i64>> {
    sqlx::query_scalar!("SELECT copy_id FROM nfc_tags WHERE tag_id = ?", tag_id)
        .fetch_optional(pool)
        .await
}
