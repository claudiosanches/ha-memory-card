//! Linked games whose title differs from IGDB's name for them.

use serde::Serialize;
use sqlx::SqlitePool;
use utoipa::ToSchema;

use crate::db::write_tx;
use crate::domain::sort_title;
use crate::error::ApiResult;

#[derive(Debug, Serialize, ToSchema)]
pub struct TitleDifference {
    pub game_id: i64,
    pub title: String,
    pub igdb_title: String,
    /// Relative to the covers URL.
    pub cover_path: Option<String>,
}

/// Leaves out titles the user kept, until IGDB renames the game again.
pub async fn differences(pool: &SqlitePool) -> sqlx::Result<Vec<TitleDifference>> {
    sqlx::query_as!(
        TitleDifference,
        r#"SELECT id AS "game_id!", title, igdb_title AS "igdb_title!", cover_path
           FROM games
           WHERE igdb_title IS NOT NULL AND title != igdb_title
             AND igdb_title_declined IS NOT igdb_title
           ORDER BY sort_title, id"#
    )
    .fetch_all(pool)
    .await
}

/// Renames `use_igdb` games to their IGDB title and hides `keep` games from the list.
pub async fn resolve(pool: &SqlitePool, use_igdb: &[i64], keep: &[i64]) -> ApiResult<()> {
    let mut tx = write_tx(pool).await?;
    for &id in use_igdb {
        let Some(title) = sqlx::query_scalar!("SELECT igdb_title FROM games WHERE id = ?", id)
            .fetch_optional(&mut *tx)
            .await?
            .flatten()
        else {
            continue;
        };
        let sort = sort_title(&title);
        sqlx::query!(
            "UPDATE games SET title = ?, sort_title = ?,
                 updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
             WHERE id = ?",
            title,
            sort,
            id
        )
        .execute(&mut *tx)
        .await?;
    }
    for &id in keep {
        sqlx::query!(
            "UPDATE games SET igdb_title_declined = igdb_title WHERE id = ?",
            id
        )
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(())
}
