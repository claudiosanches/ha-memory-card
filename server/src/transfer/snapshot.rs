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
}

pub async fn load(conn: &mut SqliteConnection) -> sqlx::Result<Snapshot> {
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
        r#"SELECT id AS "id!", igdb_id, title, play_status FROM games"#
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
                  format AS "format: Format", store_id, condition AS "condition: Condition", notes
           FROM copies"#
    )
    .fetch_all(&mut *conn)
    .await?;

    let play_statuses = play_statuses::names(&mut *conn).await?;

    Ok(Snapshot {
        platforms,
        devices,
        stores,
        games,
        releases,
        copies,
        play_statuses,
    })
}
