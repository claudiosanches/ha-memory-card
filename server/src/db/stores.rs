use serde::Serialize;
use sqlx::SqlitePool;
use utoipa::ToSchema;

use super::found;
use crate::error::{ApiError, ApiResult};
use crate::slug::{self, Table};

#[derive(Debug, Serialize, ToSchema)]
pub struct Store {
    pub id: i64,
    pub name: String,
    /// Names the store in library links.
    pub slug: String,
    pub notes: Option<String>,
    /// Copies bought at this store, of any status.
    pub copy_count: i64,
}

pub struct StoreFields {
    pub name: String,
    /// `None` makes one from the name for a new store and keeps it on update.
    pub slug: Option<String>,
    pub notes: Option<String>,
}

pub async fn list(pool: &SqlitePool) -> sqlx::Result<Vec<Store>> {
    sqlx::query_as!(
        Store,
        r#"SELECT s.id AS "id!", s.name, s.slug, s.notes,
                  (SELECT COUNT(*) FROM copies c WHERE c.store_id = s.id) AS "copy_count!: i64"
           FROM stores s
           ORDER BY s.name COLLATE NOCASE"#
    )
    .fetch_all(pool)
    .await
}

pub async fn get(pool: &SqlitePool, id: i64) -> ApiResult<Store> {
    sqlx::query_as!(
        Store,
        r#"SELECT s.id AS "id!", s.name, s.slug, s.notes,
                  (SELECT COUNT(*) FROM copies c WHERE c.store_id = s.id) AS "copy_count!: i64"
           FROM stores s
           WHERE s.id = ?"#,
        id
    )
    .fetch_optional(pool)
    .await?
    .ok_or(ApiError::NotFound)
}

pub async fn create(pool: &SqlitePool, fields: &StoreFields) -> ApiResult<i64> {
    let mut tx = crate::db::write_tx(pool).await?;
    slug::ensure_name_free(&mut tx, Table::Stores, &fields.name, None).await?;
    let slug = slug::for_new(&mut tx, Table::Stores, fields.slug.as_deref(), &fields.name).await?;
    let id = sqlx::query_scalar!(
        r#"INSERT INTO stores (name, slug, notes) VALUES (?, ?, ?) RETURNING id AS "id!""#,
        fields.name,
        slug,
        fields.notes
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(slug::conflict)?;
    tx.commit().await?;
    Ok(id)
}

pub async fn update(pool: &SqlitePool, id: i64, fields: &StoreFields) -> ApiResult<()> {
    let mut tx = crate::db::write_tx(pool).await?;
    slug::ensure_name_free(&mut tx, Table::Stores, &fields.name, Some(id)).await?;
    let result = sqlx::query!(
        "UPDATE stores
         SET name = ?, slug = COALESCE(?, slug), notes = ?,
             updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE id = ?",
        fields.name,
        fields.slug,
        fields.notes,
        id
    )
    .execute(&mut *tx)
    .await
    .map_err(slug::conflict)?;
    found(result.rows_affected())?;
    tx.commit().await?;
    Ok(())
}

/// Copies bought at the store keep existing with no store.
pub async fn delete(pool: &SqlitePool, id: i64) -> ApiResult<()> {
    let result = sqlx::query!("DELETE FROM stores WHERE id = ?", id)
        .execute(pool)
        .await?;
    found(result.rows_affected())
}
