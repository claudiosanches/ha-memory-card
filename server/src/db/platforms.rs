use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use utoipa::ToSchema;

use super::found;
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
    pub device_count: i64,
}

pub struct PlatformFields {
    pub name: String,
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
                  (SELECT COUNT(*) FROM device_platforms d WHERE d.platform_id = p.id) AS "device_count!: i64"
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
                  (SELECT COUNT(*) FROM device_platforms d WHERE d.platform_id = p.id) AS "device_count!: i64"
           FROM platforms p
           WHERE p.id = ?"#,
        id
    )
    .fetch_optional(pool)
    .await?
    .ok_or(ApiError::NotFound)
}

/// New platforms go to the end of the list with a slug derived from the name.
pub async fn create(pool: &SqlitePool, fields: &PlatformFields) -> ApiResult<i64> {
    let mut tx = crate::db::write_tx(pool).await?;

    let base = slugify(&fields.name);
    let taken: Vec<String> = sqlx::query_scalar!(
        r#"SELECT slug AS "slug!" FROM platforms WHERE slug = ?1 OR slug LIKE ?1 || '-%'"#,
        base
    )
    .fetch_all(&mut *tx)
    .await?;
    let slug = unique_slug(&base, &taken);

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
    .map_err(|err| ApiError::unique(err, "name_taken"))?;

    tx.commit().await?;
    Ok(id)
}

pub async fn update(pool: &SqlitePool, id: i64, fields: &PlatformFields) -> ApiResult<()> {
    let result = sqlx::query!(
        // A platform switched on goes to the end of the active list.
        "UPDATE platforms
         SET name = ?1, abbreviation = ?2, category = ?3, active = ?4,
             sort_order = CASE WHEN ?4 AND NOT active
                 THEN (SELECT COALESCE(MAX(sort_order), 0) + 10 FROM platforms WHERE active)
                 ELSE sort_order END
         WHERE id = ?5",
        fields.name,
        fields.abbreviation,
        fields.category,
        fields.active,
        id
    )
    .execute(pool)
    .await
    .map_err(|err| ApiError::unique(err, "name_taken"))?;
    found(result.rows_affected())
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

/// Platforms that releases or devices use cannot be deleted; deactivate them instead.
pub async fn delete(pool: &SqlitePool, id: i64) -> ApiResult<()> {
    let mut tx = crate::db::write_tx(pool).await?;
    let in_use: bool = sqlx::query_scalar!(
        r#"SELECT EXISTS (SELECT 1 FROM releases WHERE platform_id = ?1)
               OR EXISTS (SELECT 1 FROM device_platforms WHERE platform_id = ?1) AS "in_use!: bool""#,
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

pub(crate) fn slugify(name: &str) -> String {
    let mut slug = String::new();
    for c in name.chars().flat_map(char::to_lowercase) {
        if c.is_alphanumeric() {
            slug.push(c);
        } else if !slug.ends_with('-') {
            slug.push('-');
        }
    }
    let slug = slug.trim_matches('-');
    if slug.is_empty() {
        "platform".to_owned()
    } else {
        slug.to_owned()
    }
}

pub(crate) fn unique_slug(base: &str, taken: &[String]) -> String {
    if !taken.iter().any(|s| s.eq_ignore_ascii_case(base)) {
        return base.to_owned();
    }
    (2..)
        .map(|n| format!("{base}-{n}"))
        .find(|candidate| !taken.iter().any(|s| s.eq_ignore_ascii_case(candidate)))
        .expect("unbounded range")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugify_names() {
        assert_eq!(
            slugify("Sega Mega Drive/Genesis"),
            "sega-mega-drive-genesis"
        );
        assert_eq!(slugify("  Xbox Series X|S "), "xbox-series-x-s");
        assert_eq!(slugify("PC-Engine"), "pc-engine");
        assert_eq!(slugify("???"), "platform");
    }

    #[test]
    fn unique_slug_appends_counter() {
        let taken = vec!["dreamcast".to_owned(), "dreamcast-2".to_owned()];
        assert_eq!(unique_slug("saturn", &taken), "saturn");
        assert_eq!(unique_slug("dreamcast", &taken), "dreamcast-3");
    }
}
