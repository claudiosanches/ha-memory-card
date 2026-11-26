use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use sqlx::{SqliteConnection, SqlitePool};
use utoipa::ToSchema;

use super::found;
use super::games::Price;
use super::photos::CopyPhoto;
use super::slugs::{self, Table};
use crate::domain::{Condition, CopyStatus, Region};
use crate::error::{ApiError, ApiResult};
use crate::money;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(rename_all = "snake_case")]
pub enum DeviceKind {
    Console,
    Handheld,
    Pc,
    Account,
    Controller,
    Accessory,
    Toy,
}

impl DeviceKind {
    /// Consoles, handhelds, PCs and accounts hold copies; the rest is hardware
    /// for a platform.
    pub fn holds_games(self) -> bool {
        matches!(
            self,
            Self::Console | Self::Handheld | Self::Pc | Self::Account
        )
    }
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
    /// The platform a controller, accessory or toy is for.
    pub platform_id: Option<i64>,
    pub status: CopyStatus,
    pub region: Option<Region>,
    pub condition: Option<Condition>,
    pub store_id: Option<i64>,
    pub purchased_on: Option<String>,
    /// Decimal text, paid for owned hardware or expected for wanted.
    pub price: Option<String>,
    pub currency: Option<String>,
    /// Decimal text: what it would sell for now.
    pub estimated_value: Option<String>,
    pub estimated_currency: Option<String>,
    pub serial_number: Option<String>,
    pub barcode: Option<String>,
    /// Oldest first.
    pub photos: Vec<CopyPhoto>,
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
    pub platform_id: Option<i64>,
    pub status: CopyStatus,
    pub region: Option<Region>,
    pub condition: Option<Condition>,
    pub store_id: Option<i64>,
    pub purchased_on: Option<String>,
    pub price: Option<Price>,
    pub estimated_value: Option<Price>,
    pub serial_number: Option<String>,
    pub barcode: Option<String>,
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
    platform_id: Option<i64>,
    status: CopyStatus,
    region: Option<Region>,
    condition: Option<Condition>,
    store_id: Option<i64>,
    purchased_on: Option<String>,
    price: Option<i64>,
    currency: Option<String>,
    estimated_value: Option<i64>,
    estimated_currency: Option<String>,
    serial_number: Option<String>,
    barcode: Option<String>,
    photos: Json<Vec<CopyPhoto>>,
}

fn amount(minor: Option<i64>, currency: Option<&String>) -> Option<String> {
    minor
        .zip(currency)
        .map(|(minor, currency)| money::format(minor, currency))
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
            platform_id: row.platform_id,
            status: row.status,
            region: row.region,
            condition: row.condition,
            store_id: row.store_id,
            purchased_on: row.purchased_on,
            price: amount(row.price, row.currency.as_ref()),
            currency: row.currency,
            estimated_value: amount(row.estimated_value, row.estimated_currency.as_ref()),
            estimated_currency: row.estimated_currency,
            serial_number: row.serial_number,
            barcode: row.barcode,
            photos: row.photos.0,
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
                  (SELECT COUNT(*) FROM copies c WHERE c.device_id = d.id) AS "copy_count!: i64",
                  d.platform_id, d.status AS "status: CopyStatus", d.region AS "region: Region",
                  d.condition AS "condition: Condition", d.store_id, d.purchased_on,
                  d.price, d.currency, d.estimated_value, d.estimated_currency,
                  d.serial_number, d.barcode,
                  (SELECT json_group_array(json_object('id', id, 'path', path, 'created_at', created_at))
                   FROM (SELECT id, path, created_at FROM device_photos
                         WHERE device_id = d.id ORDER BY id)) AS "photos!: Json<Vec<CopyPhoto>>"
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
                  (SELECT COUNT(*) FROM copies c WHERE c.device_id = d.id) AS "copy_count!: i64",
                  d.platform_id, d.status AS "status: CopyStatus", d.region AS "region: Region",
                  d.condition AS "condition: Condition", d.store_id, d.purchased_on,
                  d.price, d.currency, d.estimated_value, d.estimated_currency,
                  d.serial_number, d.barcode,
                  (SELECT json_group_array(json_object('id', id, 'path', path, 'created_at', created_at))
                   FROM (SELECT id, path, created_at FROM device_photos
                         WHERE device_id = d.id ORDER BY id)) AS "photos!: Json<Vec<CopyPhoto>>"
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
    check_references(&mut tx, fields).await?;
    slugs::ensure_name_free(&mut tx, Table::Devices, &fields.name, None).await?;
    let slug = slugs::for_new(
        &mut tx,
        Table::Devices,
        fields.slug.as_deref(),
        &fields.name,
    )
    .await?;

    let (price, currency) = split(fields.price.as_ref());
    let (estimated, estimated_currency) = split(fields.estimated_value.as_ref());
    let id = sqlx::query_scalar!(
        r#"INSERT INTO devices (name, slug, kind, model, notes, active, platform_id, status,
                               region, condition, store_id, purchased_on, price, currency,
                               estimated_value, estimated_currency, serial_number, barcode)
           VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
           RETURNING id AS "id!""#,
        fields.name,
        slug,
        fields.kind,
        fields.model,
        fields.notes,
        fields.active,
        fields.platform_id,
        fields.status,
        fields.region,
        fields.condition,
        fields.store_id,
        fields.purchased_on,
        price,
        currency,
        estimated,
        estimated_currency,
        fields.serial_number,
        fields.barcode
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
    check_references(&mut tx, fields).await?;
    if !fields.kind.holds_games() {
        let copies: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "n!: i64" FROM copies WHERE device_id = ?"#,
            id
        )
        .fetch_one(&mut *tx)
        .await?;
        if copies > 0 {
            return Err(ApiError::invalid("device_has_copies", "kind"));
        }
    }

    let (price, currency) = split(fields.price.as_ref());
    let (estimated, estimated_currency) = split(fields.estimated_value.as_ref());
    let result = sqlx::query!(
        "UPDATE devices
         SET name = ?, slug = COALESCE(?, slug), kind = ?, model = ?, notes = ?, active = ?,
             platform_id = ?, status = ?, region = ?, condition = ?, store_id = ?,
             purchased_on = ?, price = ?, currency = ?, estimated_value = ?,
             estimated_currency = ?, serial_number = ?, barcode = ?,
             updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE id = ?",
        fields.name,
        fields.slug,
        fields.kind,
        fields.model,
        fields.notes,
        fields.active,
        fields.platform_id,
        fields.status,
        fields.region,
        fields.condition,
        fields.store_id,
        fields.purchased_on,
        price,
        currency,
        estimated,
        estimated_currency,
        fields.serial_number,
        fields.barcode,
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

/// Copies on the device keep existing with no device. Returns the paths of
/// its photos, for the caller to remove.
pub async fn delete(pool: &SqlitePool, id: i64) -> ApiResult<Vec<String>> {
    let mut tx = crate::db::write_tx(pool).await?;
    let photos = sqlx::query_scalar!("SELECT path FROM device_photos WHERE device_id = ?", id)
        .fetch_all(&mut *tx)
        .await?;
    let result = sqlx::query!("DELETE FROM devices WHERE id = ?", id)
        .execute(&mut *tx)
        .await?;
    found(result.rows_affected())?;
    tx.commit().await?;
    Ok(photos)
}

fn split(price: Option<&Price>) -> (Option<i64>, Option<&str>) {
    price.map(|p| (p.minor, p.currency.as_str())).unzip()
}

async fn check_references(conn: &mut SqliteConnection, fields: &DeviceFields) -> ApiResult<()> {
    if let Some(platform_id) = fields.platform_id {
        let exists: bool = sqlx::query_scalar!(
            r#"SELECT EXISTS (SELECT 1 FROM platforms WHERE id = ?) AS "exists!: bool""#,
            platform_id
        )
        .fetch_one(&mut *conn)
        .await?;
        if !exists {
            return Err(ApiError::invalid("unknown_platform", "platform_id"));
        }
    }
    if let Some(store_id) = fields.store_id {
        let exists: bool = sqlx::query_scalar!(
            r#"SELECT EXISTS (SELECT 1 FROM stores WHERE id = ?) AS "exists!: bool""#,
            store_id
        )
        .fetch_one(&mut *conn)
        .await?;
        if !exists {
            return Err(ApiError::invalid("unknown_store", "store_id"));
        }
    }
    Ok(())
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
