use sqlx::SqlitePool;

use super::{COLUMNS, UTF8_BOM};
use crate::domain::{Condition, CopyStatus, Format, Region};

struct ExportRow {
    copy_id: Option<i64>,
    status: Option<CopyStatus>,
    igdb_id: Option<i64>,
    game_title: String,
    play_status: Option<String>,
    platform: Option<String>,
    region: Option<Region>,
    release_title: Option<String>,
    product_code: Option<String>,
    format: Option<Format>,
    device: Option<String>,
    store: Option<String>,
    condition: Option<Condition>,
    notes: Option<String>,
}

/// Every copy, plus one row for each game without copies, as CSV.
pub async fn export(pool: &SqlitePool) -> Result<Vec<u8>, ExportError> {
    let rows = sqlx::query_as!(
        ExportRow,
        r#"SELECT copy_id AS "copy_id?: i64", status AS "status?: CopyStatus",
                  igdb_id AS "igdb_id?: i64", game_title AS "game_title!: String",
                  play_status AS "play_status?: String", platform AS "platform?: String",
                  region AS "region?: Region", release_title AS "release_title?: String",
                  product_code AS "product_code?: String", format AS "format?: Format",
                  device AS "device?: String", store AS "store?: String",
                  condition AS "condition?: Condition", notes AS "notes?: String"
           FROM (
               SELECT c.id AS copy_id, c.status, g.igdb_id, g.title AS game_title, g.play_status,
                      p.name AS platform, r.region, r.title AS release_title, r.product_code,
                      c.format, d.name AS device, s.name AS store, c.condition, c.notes,
                      g.sort_title, g.id AS game_id, p.sort_order AS platform_order
               FROM copies c
               JOIN releases r ON r.id = c.release_id
               JOIN games g ON g.id = r.game_id
               JOIN platforms p ON p.id = r.platform_id
               LEFT JOIN devices d ON d.id = c.device_id
               LEFT JOIN stores s ON s.id = c.store_id
               UNION ALL
               SELECT NULL, NULL, g.igdb_id, g.title, g.play_status,
                      NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL,
                      g.sort_title, g.id, NULL
               FROM games g
               WHERE NOT EXISTS (
                   SELECT 1 FROM releases r JOIN copies c ON c.release_id = r.id
                   WHERE r.game_id = g.id)
           )
           ORDER BY sort_title, game_id, platform_order, region, copy_id"#
    )
    .fetch_all(pool)
    .await?;

    let mut out = UTF8_BOM.to_vec();
    {
        let mut writer = csv::Writer::from_writer(&mut out);
        writer.write_record(COLUMNS)?;
        for row in rows {
            let id = |v: Option<i64>| v.map(|v| v.to_string()).unwrap_or_default();
            let text = |v: Option<String>| v.unwrap_or_default();
            writer.write_record([
                id(row.copy_id),
                text(row.status.map(|v| v.as_str().to_owned())),
                id(row.igdb_id),
                row.game_title,
                text(row.play_status),
                text(row.platform),
                text(row.region.map(|v| v.as_str().to_owned())),
                text(row.release_title),
                text(row.product_code),
                text(row.format.map(|v| v.as_str().to_owned())),
                text(row.device),
                text(row.store),
                text(row.condition.map(|v| v.as_str().to_owned())),
                text(row.notes),
            ])?;
        }
        writer.flush()?;
    }
    Ok(out)
}

#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error(transparent)]
    Csv(#[from] csv::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
