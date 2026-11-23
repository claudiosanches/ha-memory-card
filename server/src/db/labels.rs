use serde::Serialize;
use sqlx::{SqliteConnection, SqlitePool};
use utoipa::ToSchema;

use super::found;
use crate::domain::StatusColor;
use crate::error::{ApiError, ApiResult};
use crate::slug::{self, Table};

/// Separate labels in a CSV cell, so names cannot hold them.
pub const SEPARATORS: [char; 2] = [',', ';'];
/// Most labels one game can have. The export repeats them on every copy.
pub const PER_GAME_MAX: usize = 50;

#[derive(Debug, Serialize, ToSchema)]
pub struct Label {
    pub id: i64,
    pub name: String,
    /// Names the label in library links.
    pub slug: String,
    pub color: StatusColor,
    pub game_count: i64,
}

/// A label on a game.
#[derive(Debug, Serialize, ToSchema)]
pub struct GameLabel {
    pub id: i64,
    pub name: String,
    pub slug: String,
    pub color: StatusColor,
}

pub struct LabelFields {
    pub name: String,
    /// `None` makes one from the name for a new label and keeps it on update.
    pub slug: Option<String>,
    pub color: StatusColor,
}

pub async fn list(pool: &SqlitePool) -> sqlx::Result<Vec<Label>> {
    sqlx::query_as!(
        Label,
        r#"SELECT l.id AS "id!", l.name, l.slug, l.color AS "color: StatusColor",
                  (SELECT COUNT(*) FROM game_labels gl WHERE gl.label_id = l.id) AS "game_count!: i64"
           FROM labels l
           ORDER BY l.name COLLATE NOCASE"#
    )
    .fetch_all(pool)
    .await
}

pub async fn get(pool: &SqlitePool, id: i64) -> ApiResult<Label> {
    sqlx::query_as!(
        Label,
        r#"SELECT l.id AS "id!", l.name, l.slug, l.color AS "color: StatusColor",
                  (SELECT COUNT(*) FROM game_labels gl WHERE gl.label_id = l.id) AS "game_count!: i64"
           FROM labels l
           WHERE l.id = ?"#,
        id
    )
    .fetch_optional(pool)
    .await?
    .ok_or(ApiError::NotFound)
}

pub async fn for_game(pool: &SqlitePool, game_id: i64) -> sqlx::Result<Vec<GameLabel>> {
    sqlx::query_as!(
        GameLabel,
        r#"SELECT l.id AS "id!", l.name, l.slug, l.color AS "color: StatusColor"
           FROM game_labels gl JOIN labels l ON l.id = gl.label_id
           WHERE gl.game_id = ?
           ORDER BY l.name COLLATE NOCASE"#,
        game_id
    )
    .fetch_all(pool)
    .await
}

pub async fn create(pool: &SqlitePool, fields: &LabelFields) -> ApiResult<i64> {
    let mut tx = crate::db::write_tx(pool).await?;
    slug::ensure_name_free(&mut tx, Table::Labels, &fields.name, None).await?;
    let slug = slug::for_new(&mut tx, Table::Labels, fields.slug.as_deref(), &fields.name).await?;
    let id = sqlx::query_scalar!(
        r#"INSERT INTO labels (name, slug, color) VALUES (?, ?, ?) RETURNING id AS "id!""#,
        fields.name,
        slug,
        fields.color
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(slug::conflict)?;
    tx.commit().await?;
    Ok(id)
}

pub async fn update(pool: &SqlitePool, id: i64, fields: &LabelFields) -> ApiResult<()> {
    let mut tx = crate::db::write_tx(pool).await?;
    slug::ensure_name_free(&mut tx, Table::Labels, &fields.name, Some(id)).await?;
    let result = sqlx::query!(
        "UPDATE labels
         SET name = ?, slug = COALESCE(?, slug), color = ?,
             updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE id = ?",
        fields.name,
        fields.slug,
        fields.color,
        id
    )
    .execute(&mut *tx)
    .await
    .map_err(slug::conflict)?;
    found(result.rows_affected())?;
    tx.commit().await?;
    Ok(())
}

/// Games keep existing without it.
pub async fn delete(pool: &SqlitePool, id: i64) -> ApiResult<()> {
    let result = sqlx::query!("DELETE FROM labels WHERE id = ?", id)
        .execute(pool)
        .await?;
    found(result.rows_affected())
}

/// Replaces a game's labels. `unknown_id` when one does not exist.
pub async fn set_for_game(conn: &mut SqliteConnection, game_id: i64, ids: &[i64]) -> ApiResult<()> {
    let mut ids = ids.to_vec();
    ids.sort_unstable();
    ids.dedup();
    let json = serde_json::to_string(&ids).expect("integers serialize");
    let known = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "count!: i64" FROM labels WHERE id IN (SELECT value FROM json_each(?))"#,
        json
    )
    .fetch_one(&mut *conn)
    .await?;
    if known != ids.len() as i64 {
        return Err(ApiError::invalid("unknown_id", "label_ids"));
    }
    sqlx::query!("DELETE FROM game_labels WHERE game_id = ?", game_id)
        .execute(&mut *conn)
        .await?;
    sqlx::query!(
        "INSERT INTO game_labels (game_id, label_id) SELECT ?, value FROM json_each(?)",
        game_id,
        json
    )
    .execute(&mut *conn)
    .await?;
    Ok(())
}
