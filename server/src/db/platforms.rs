use serde::Serialize;
use sqlx::SqlitePool;
use utoipa::ToSchema;

use super::found;
use crate::error::{ApiError, ApiResult};

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
    pub release_count: i64,
    pub device_count: i64,
}

pub struct PlatformFields {
    pub name: String,
    pub abbreviation: Option<String>,
    pub active: bool,
}

pub async fn list(pool: &SqlitePool) -> sqlx::Result<Vec<Platform>> {
    sqlx::query_as!(
        Platform,
        r#"SELECT p.id AS "id!", p.igdb_platform_id, p.name, p.abbreviation, p.slug,
                  p.family, p.generation, p.active AS "active: bool", p.sort_order,
                  (SELECT COUNT(*) FROM releases r WHERE r.platform_id = p.id) AS "release_count!: i64",
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
                  (SELECT COUNT(*) FROM releases r WHERE r.platform_id = p.id) AS "release_count!: i64",
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
    let mut tx = pool.begin().await?;

    let base = slugify(&fields.name);
    let taken: Vec<String> = sqlx::query_scalar!(
        r#"SELECT slug AS "slug!" FROM platforms WHERE slug = ?1 OR slug LIKE ?1 || '-%'"#,
        base
    )
    .fetch_all(&mut *tx)
    .await?;
    let slug = unique_slug(&base, &taken);

    let id = sqlx::query_scalar!(
        r#"INSERT INTO platforms (name, abbreviation, slug, active, sort_order)
           VALUES (?, ?, ?, ?, (SELECT COALESCE(MAX(sort_order), 0) + 10 FROM platforms))
           RETURNING id AS "id!""#,
        fields.name,
        fields.abbreviation,
        slug,
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
        "UPDATE platforms SET name = ?, abbreviation = ?, active = ? WHERE id = ?",
        fields.name,
        fields.abbreviation,
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
    let mut tx = pool.begin().await?;

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
    let mut tx = pool.begin().await?;
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

    let result = sqlx::query!("DELETE FROM platforms WHERE id = ?", id)
        .execute(&mut *tx)
        .await?;
    found(result.rows_affected())?;
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
