use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use sqlx::{SqliteConnection, SqlitePool};
use utoipa::ToSchema;

use super::found;
use super::slugs::{self, Table};
use crate::error::{ApiError, ApiResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(rename_all = "snake_case")]
pub enum DeviceKind {
    Console,
    Handheld,
    Pc,
    Account,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct Device {
    pub id: i64,
    pub name: String,
    /// Names the device in library links.
    pub slug: String,
    pub kind: DeviceKind,
    pub model: Option<String>,
    pub notes: Option<String>,
    pub active: bool,
    /// Platforms this device plays, in platform list order.
    pub platform_ids: Vec<i64>,
    /// Copies assigned to this device, of any status.
    pub copy_count: i64,
}

pub struct DeviceFields {
    pub name: String,
    /// `None` makes one from the name for a new device and keeps it on update.
    pub slug: Option<String>,
    pub kind: DeviceKind,
    pub model: Option<String>,
    pub notes: Option<String>,
    pub active: bool,
    pub platform_ids: Vec<i64>,
}

struct DeviceRow {
    id: i64,
    name: String,
    slug: String,
    kind: DeviceKind,
    model: Option<String>,
    notes: Option<String>,
    active: bool,
    platform_ids: Json<Vec<i64>>,
    copy_count: i64,
}

impl From<DeviceRow> for Device {
    fn from(row: DeviceRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            slug: row.slug,
            kind: row.kind,
            model: row.model,
            notes: row.notes,
            active: row.active,
            platform_ids: row.platform_ids.0,
            copy_count: row.copy_count,
        }
    }
}

pub async fn list(pool: &SqlitePool) -> sqlx::Result<Vec<Device>> {
    let rows = sqlx::query_as!(
        DeviceRow,
        r#"SELECT d.id AS "id!", d.name, d.slug, d.kind AS "kind: DeviceKind", d.model, d.notes,
                  d.active AS "active: bool",
                  (SELECT json_group_array(platform_id) FROM (
                       SELECT dp.platform_id FROM device_platforms dp
                       JOIN platforms p ON p.id = dp.platform_id
                       WHERE dp.device_id = d.id
                       ORDER BY p.sort_order, p.name)) AS "platform_ids!: Json<Vec<i64>>",
                  (SELECT COUNT(*) FROM copies c WHERE c.device_id = d.id) AS "copy_count!: i64"
           FROM devices d
           ORDER BY d.name COLLATE NOCASE"#
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(Device::from).collect())
}

pub async fn get(pool: &SqlitePool, id: i64) -> ApiResult<Device> {
    sqlx::query_as!(
        DeviceRow,
        r#"SELECT d.id AS "id!", d.name, d.slug, d.kind AS "kind: DeviceKind", d.model, d.notes,
                  d.active AS "active: bool",
                  (SELECT json_group_array(platform_id) FROM (
                       SELECT dp.platform_id FROM device_platforms dp
                       JOIN platforms p ON p.id = dp.platform_id
                       WHERE dp.device_id = d.id
                       ORDER BY p.sort_order, p.name)) AS "platform_ids!: Json<Vec<i64>>",
                  (SELECT COUNT(*) FROM copies c WHERE c.device_id = d.id) AS "copy_count!: i64"
           FROM devices d
           WHERE d.id = ?"#,
        id
    )
    .fetch_optional(pool)
    .await?
    .map(Device::from)
    .ok_or(ApiError::NotFound)
}

pub async fn create(pool: &SqlitePool, fields: &DeviceFields) -> ApiResult<i64> {
    let mut tx = crate::db::write_tx(pool).await?;
    let platform_ids = checked_platform_ids(&mut tx, &fields.platform_ids).await?;
    slugs::ensure_name_free(&mut tx, Table::Devices, &fields.name, None).await?;
    let slug = slugs::for_new(
        &mut tx,
        Table::Devices,
        fields.slug.as_deref(),
        &fields.name,
    )
    .await?;

    let id = sqlx::query_scalar!(
        r#"INSERT INTO devices (name, slug, kind, model, notes, active)
           VALUES (?, ?, ?, ?, ?, ?)
           RETURNING id AS "id!""#,
        fields.name,
        slug,
        fields.kind,
        fields.model,
        fields.notes,
        fields.active
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(slugs::conflict)?;

    set_platforms(&mut tx, id, &platform_ids).await?;
    tx.commit().await?;
    Ok(id)
}

pub async fn update(pool: &SqlitePool, id: i64, fields: &DeviceFields) -> ApiResult<()> {
    let mut tx = crate::db::write_tx(pool).await?;
    slugs::ensure_name_free(&mut tx, Table::Devices, &fields.name, Some(id)).await?;
    let platform_ids = checked_platform_ids(&mut tx, &fields.platform_ids).await?;

    let result = sqlx::query!(
        "UPDATE devices
         SET name = ?, slug = COALESCE(?, slug), kind = ?, model = ?, notes = ?, active = ?,
             updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE id = ?",
        fields.name,
        fields.slug,
        fields.kind,
        fields.model,
        fields.notes,
        fields.active,
        id
    )
    .execute(&mut *tx)
    .await
    .map_err(slugs::conflict)?;
    found(result.rows_affected())?;

    sqlx::query!("DELETE FROM device_platforms WHERE device_id = ?", id)
        .execute(&mut *tx)
        .await?;
    set_platforms(&mut tx, id, &platform_ids).await?;
    tx.commit().await?;
    Ok(())
}

/// Copies on the device keep existing with no device.
pub async fn delete(pool: &SqlitePool, id: i64) -> ApiResult<()> {
    let result = sqlx::query!("DELETE FROM devices WHERE id = ?", id)
        .execute(pool)
        .await?;
    found(result.rows_affected())
}

/// Deduplicates the ids and checks they all exist. Returns them as a JSON array.
async fn checked_platform_ids(conn: &mut SqliteConnection, ids: &[i64]) -> ApiResult<String> {
    let mut ids = ids.to_vec();
    ids.sort_unstable();
    ids.dedup();
    let json = serde_json::to_string(&ids).expect("integers serialize");

    let known: i64 = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "count!: i64" FROM platforms
           WHERE id IN (SELECT value FROM json_each(?))"#,
        json
    )
    .fetch_one(&mut *conn)
    .await?;
    if known != ids.len() as i64 {
        return Err(ApiError::invalid("unknown_platform", "platform_ids"));
    }
    Ok(json)
}

async fn set_platforms(
    conn: &mut SqliteConnection,
    device_id: i64,
    json_ids: &str,
) -> ApiResult<()> {
    sqlx::query!(
        "INSERT INTO device_platforms (device_id, platform_id)
         SELECT ?, value FROM json_each(?)",
        device_id,
        json_ids
    )
    .execute(&mut *conn)
    .await?;
    Ok(())
}
