use serde::Serialize;
use sqlx::SqlitePool;
use utoipa::ToSchema;

use super::found;
use crate::error::{ApiError, ApiResult};

#[derive(Debug, Serialize, ToSchema)]
pub struct Store {
    pub id: i64,
    pub name: String,
    pub notes: Option<String>,
    /// Copies bought at this store, of any status.
    pub copy_count: i64,
}

pub struct StoreFields {
    pub name: String,
    pub notes: Option<String>,
}

pub async fn list(pool: &SqlitePool) -> sqlx::Result<Vec<Store>> {
    sqlx::query_as!(
        Store,
        r#"SELECT s.id AS "id!", s.name, s.notes,
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
        r#"SELECT s.id AS "id!", s.name, s.notes,
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
    sqlx::query_scalar!(
        r#"INSERT INTO stores (name, notes) VALUES (?, ?) RETURNING id AS "id!""#,
        fields.name,
        fields.notes
    )
    .fetch_one(pool)
    .await
    .map_err(|err| ApiError::unique(err, "name_taken"))
}

pub async fn update(pool: &SqlitePool, id: i64, fields: &StoreFields) -> ApiResult<()> {
    let result = sqlx::query!(
        "UPDATE stores
         SET name = ?, notes = ?, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE id = ?",
        fields.name,
        fields.notes,
        id
    )
    .execute(pool)
    .await
    .map_err(|err| ApiError::unique(err, "name_taken"))?;
    found(result.rows_affected())
}

/// Copies bought at the store keep existing with no store.
pub async fn delete(pool: &SqlitePool, id: i64) -> ApiResult<()> {
    let result = sqlx::query!("DELETE FROM stores WHERE id = ?", id)
        .execute(pool)
        .await?;
    found(result.rows_affected())
}
