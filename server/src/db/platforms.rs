use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use utoipa::ToSchema;

use super::found;
use super::slugs::{self, Table};
use crate::error::{ApiError, ApiResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(rename_all = "snake_case")]
pub enum PlatformCategory {
    Console,
    Handheld,
    Computer,
    Mobile,
    Arcade,
    Other,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct Platform {
    pub id: i64,
    pub igdb_platform_id: Option<i64>,
    pub name: String,
    pub abbreviation: Option<String>,
    pub slug: String,
    pub family: Option<String>,
    pub generation: Option<i64>,
    pub active: bool,
    pub sort_order: i64,
    /// `None` for platforms added by hand without one.
    pub category: Option<PlatformCategory>,
    /// Added by hand or by an import rather than shipped with the app. Only
    /// these can be deleted.
    pub custom: bool,
    /// Games with a release on this platform.
    pub game_count: i64,
    /// Games with an owned copy on this platform.
    pub owned_game_count: i64,
    /// Games with a wishlist copy on this platform.
    pub wanted_game_count: i64,
    pub device_count: i64,
    pub account_count: i64,
}

pub struct PlatformFields {
    pub name: String,
    /// `None` makes one from the name for a new platform and keeps it on update.
    pub slug: Option<String>,
    pub abbreviation: Option<String>,
    pub category: Option<PlatformCategory>,
    pub active: bool,
}

pub async fn list(pool: &SqlitePool) -> sqlx::Result<Vec<Platform>> {
    sqlx::query_as!(
        Platform,
        r#"SELECT p.id AS "id!", p.igdb_platform_id, p.name, p.abbreviation, p.slug,
                  p.family, p.generation, p.active AS "active: bool", p.sort_order,
                  p.category AS "category: PlatformCategory", p.custom AS "custom: bool",
                  (SELECT COUNT(DISTINCT r.game_id) FROM releases r WHERE r.platform_id = p.id) AS "game_count!: i64",
                  (SELECT COUNT(DISTINCT r.game_id) FROM releases r JOIN copies c ON c.release_id = r.id
                    WHERE r.platform_id = p.id AND c.status = 'owned') AS "owned_game_count!: i64",
                  (SELECT COUNT(DISTINCT r.game_id) FROM releases r JOIN copies c ON c.release_id = r.id
                    WHERE r.platform_id = p.id AND c.status = 'wanted') AS "wanted_game_count!: i64",
                  (SELECT COUNT(*) FROM device_platforms d WHERE d.platform_id = p.id) AS "device_count!: i64",
                  (SELECT COUNT(*) FROM account_platforms a WHERE a.platform_id = p.id) AS "account_count!: i64"
           FROM platforms p
           ORDER BY p.sort_order, p.name COLLATE NOCASE"#
    )
    .fetch_all(pool)
    .await
}

pub async fn get(pool: &SqlitePool, id: i64) -> ApiResult<Platform> {
    sqlx::query_as!(
        Platform,
        r#"SELECT p.id AS "id!", p.igdb_platform_id, p.name, p.abbreviation, p.slug,
                  p.family, p.generation, p.active AS "active: bool", p.sort_order,
                  p.category AS "category: PlatformCategory", p.custom AS "custom: bool",
                  (SELECT COUNT(DISTINCT r.game_id) FROM releases r WHERE r.platform_id = p.id) AS "game_count!: i64",
                  (SELECT COUNT(DISTINCT r.game_id) FROM releases r JOIN copies c ON c.release_id = r.id
                    WHERE r.platform_id = p.id AND c.status = 'owned') AS "owned_game_count!: i64",
                  (SELECT COUNT(DISTINCT r.game_id) FROM releases r JOIN copies c ON c.release_id = r.id
                    WHERE r.platform_id = p.id AND c.status = 'wanted') AS "wanted_game_count!: i64",
                  (SELECT COUNT(*) FROM device_platforms d WHERE d.platform_id = p.id) AS "device_count!: i64",
                  (SELECT COUNT(*) FROM account_platforms a WHERE a.platform_id = p.id) AS "account_count!: i64"
           FROM platforms p
           WHERE p.id = ?"#,
        id
    )
    .fetch_optional(pool)
    .await?
    .ok_or(ApiError::NotFound)
}

/// New platforms go to the end of the list.
pub async fn create(pool: &SqlitePool, fields: &PlatformFields) -> ApiResult<i64> {
    let mut tx = crate::db::write_tx(pool).await?;

    slugs::ensure_name_free(&mut tx, Table::Platforms, &fields.name, None).await?;
    let slug = slugs::for_new(
        &mut tx,
        Table::Platforms,
        fields.slug.as_deref(),
        &fields.name,
    )
    .await?;

    let id = sqlx::query_scalar!(
        r#"INSERT INTO platforms (name, abbreviation, slug, category, active, custom, sort_order)
           VALUES (?, ?, ?, ?, ?, 1, (SELECT COALESCE(MAX(sort_order), 0) + 10 FROM platforms WHERE active))
           RETURNING id AS "id!""#,
        fields.name,
        fields.abbreviation,
        slug,
        fields.category,
        fields.active
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(slugs::conflict)?;

    tx.commit().await?;
    Ok(id)
}

pub async fn update(pool: &SqlitePool, id: i64, fields: &PlatformFields) -> ApiResult<()> {
    let mut tx = crate::db::write_tx(pool).await?;
    slugs::ensure_name_free(&mut tx, Table::Platforms, &fields.name, Some(id)).await?;
    let result = sqlx::query!(
        // A platform switched on goes to the end of the active list.
        "UPDATE platforms
         SET name = ?1, abbreviation = ?2, category = ?3, active = ?4, slug = COALESCE(?6, slug),
             sort_order = CASE WHEN ?4 AND NOT active
                 THEN (SELECT COALESCE(MAX(sort_order), 0) + 10 FROM platforms WHERE active)
                 ELSE sort_order END
         WHERE id = ?5",
        fields.name,
        fields.abbreviation,
        fields.category,
        fields.active,
        id,
        fields.slug
    )
    .execute(&mut *tx)
    .await
    .map_err(slugs::conflict)?;
    found(result.rows_affected())?;
    tx.commit().await?;
    Ok(())
}

/// `ids` must list every platform exactly once, in the new order.
pub async fn reorder(pool: &SqlitePool, ids: &[i64]) -> ApiResult<()> {
    let mut tx = crate::db::write_tx(pool).await?;

    let mut existing: Vec<i64> = sqlx::query_scalar!(r#"SELECT id AS "id!" FROM platforms"#)
        .fetch_all(&mut *tx)
        .await?;
    let mut requested = ids.to_vec();
    existing.sort_unstable();
    requested.sort_unstable();
    if existing != requested {
        return Err(ApiError::invalid("invalid_order", "ids"));
    }

    for (position, id) in ids.iter().enumerate() {
        let sort_order = (position as i64 + 1) * 10;
        sqlx::query!(
            "UPDATE platforms SET sort_order = ? WHERE id = ?",
            sort_order,
            id
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

/// Platforms that releases, devices or accounts use cannot be deleted; deactivate them instead.
pub async fn delete(pool: &SqlitePool, id: i64) -> ApiResult<()> {
    let mut tx = crate::db::write_tx(pool).await?;
    let in_use: bool = sqlx::query_scalar!(
        r#"SELECT EXISTS (SELECT 1 FROM releases WHERE platform_id = ?1)
               OR EXISTS (SELECT 1 FROM device_platforms WHERE platform_id = ?1)
               OR EXISTS (SELECT 1 FROM account_platforms WHERE platform_id = ?1) AS "in_use!: bool""#,
        id
    )
    .fetch_one(&mut *tx)
    .await?;
    if in_use {
        return Err(ApiError::Conflict("platform_in_use"));
    }
    let custom: Option<bool> = sqlx::query_scalar!(
        r#"SELECT custom AS "custom: bool" FROM platforms WHERE id = ?"#,
        id
    )
    .fetch_optional(&mut *tx)
    .await?;
    match custom {
        None => return Err(ApiError::NotFound),
        Some(false) => return Err(ApiError::Conflict("platform_builtin")),
        Some(true) => {}
    }

    sqlx::query!("DELETE FROM platforms WHERE id = ?", id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(())
}
