//! The current collection, loaded once so an import can be planned in memory.

use sqlx::SqliteConnection;

use crate::db::play_statuses::{self, StatusName};
use crate::domain::{Condition, CopyStatus, Format, Region};

#[derive(Debug, Clone)]
pub struct PlatformRow {
    pub id: i64,
    pub name: String,
    pub abbreviation: Option<String>,
    pub slug: String,
}

#[derive(Debug, Clone)]
pub struct NamedRow {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct DeviceRow {
    pub id: i64,
    pub name: String,
    pub platform_ids: Vec<i64>,
}

#[derive(Debug, Clone)]
pub struct GameRow {
    pub id: i64,
    pub igdb_id: Option<i64>,
    pub title: String,
    pub play_status: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ReleaseRow {
    pub id: i64,
    pub game_id: i64,
    pub platform_id: i64,
    pub region: Region,
    pub product_code: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CopyRow {
    pub id: i64,
    pub release_id: i64,
    pub status: CopyStatus,
    pub device_id: Option<i64>,
    pub format: Format,
    pub store_id: Option<i64>,
    pub condition: Option<Condition>,
    pub notes: Option<String>,
    pub edition: Option<String>,
    pub purchased_on: Option<String>,
    pub price: Option<i64>,
    pub currency: Option<String>,
    pub estimated_value: Option<i64>,
    pub estimated_currency: Option<String>,
    pub barcode: Option<String>,
    pub bundle_id: Option<i64>,
}

#[derive(Debug, Clone, Default)]
pub struct Snapshot {
    pub platforms: Vec<PlatformRow>,
    pub devices: Vec<DeviceRow>,
    pub stores: Vec<NamedRow>,
    pub games: Vec<GameRow>,
    pub releases: Vec<ReleaseRow>,
    pub copies: Vec<CopyRow>,
    pub play_statuses: Vec<StatusName>,
    pub labels: Vec<NamedRow>,
    /// Game and label ids.
    pub game_labels: Vec<(i64, i64)>,
}

/// The collection as an import sees it, with `person`'s play statuses.
pub async fn load(conn: &mut SqliteConnection, person: &str) -> sqlx::Result<Snapshot> {
    let platforms = sqlx::query_as!(
        PlatformRow,
        r#"SELECT id AS "id!", name, abbreviation, slug FROM platforms ORDER BY sort_order"#
    )
    .fetch_all(&mut *conn)
    .await?;

    let devices = sqlx::query!(
        r#"SELECT d.id AS "id!", d.name,
                  (SELECT json_group_array(platform_id) FROM device_platforms
                   WHERE device_id = d.id) AS "platform_ids!: sqlx::types::Json<Vec<i64>>"
           FROM devices d"#
    )
    .fetch_all(&mut *conn)
    .await?
    .into_iter()
    .map(|row| DeviceRow {
        id: row.id,
        name: row.name,
        platform_ids: row.platform_ids.0,
    })
    .collect();

    let stores = sqlx::query_as!(NamedRow, r#"SELECT id AS "id!", name FROM stores"#)
        .fetch_all(&mut *conn)
        .await?;

    let games = sqlx::query_as!(
        GameRow,
        r#"SELECT g.id AS "id!", g.igdb_id, g.title, p.play_status AS "play_status: String"
           FROM games g LEFT JOIN play_progress p ON p.game_id = g.id AND p.user_id = ?"#,
        person
    )
    .fetch_all(&mut *conn)
    .await?;

    let releases = sqlx::query_as!(
        ReleaseRow,
        r#"SELECT id AS "id!", game_id, platform_id, region AS "region: Region", product_code, title
           FROM releases"#
    )
    .fetch_all(&mut *conn)
    .await?;

    let copies = sqlx::query_as!(
        CopyRow,
        r#"SELECT id AS "id!", release_id, status AS "status: CopyStatus", device_id,
                  format AS "format: Format", store_id, condition AS "condition: Condition", notes,
                  edition, purchased_on, price, currency, estimated_value, estimated_currency,
                  barcode, bundle_id
           FROM copies"#
    )
    .fetch_all(&mut *conn)
    .await?;

    let play_statuses = play_statuses::names(&mut *conn).await?;

    let labels = sqlx::query_as!(NamedRow, r#"SELECT id AS "id!", name FROM labels"#)
        .fetch_all(&mut *conn)
        .await?;
    let game_labels = sqlx::query!("SELECT game_id, label_id FROM game_labels")
        .fetch_all(&mut *conn)
        .await?
        .into_iter()
        .map(|row| (row.game_id, row.label_id))
        .collect();

    Ok(Snapshot {
        platforms,
        devices,
        stores,
        games,
        releases,
        copies,
        play_statuses,
        labels,
        game_labels,
    })
}
