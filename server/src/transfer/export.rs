use sqlx::{QueryBuilder, Sqlite, SqlitePool};

use super::{COLUMNS, UTF8_BOM};
use crate::db::library::{self, LibraryFilter};
use crate::domain::{Condition, CopyStatus, Format, Region};
use crate::money;

#[derive(sqlx::FromRow)]
struct ExportRow {
    copy_id: Option<i64>,
    status: Option<CopyStatus>,
    igdb_id: Option<i64>,
    game_title: String,
    play_status: Option<String>,
    labels: Option<String>,
    platform: Option<String>,
    region: Option<Region>,
    release_title: Option<String>,
    product_code: Option<String>,
    format: Option<Format>,
    device: Option<String>,
    store: Option<String>,
    condition: Option<Condition>,
    notes: Option<String>,
    edition: Option<String>,
    purchased_on: Option<String>,
    price: Option<i64>,
    currency: Option<String>,
    estimated_value: Option<i64>,
    estimated_currency: Option<String>,
    barcode: Option<String>,
    bundle_copy_id: Option<i64>,
}

/// Copies matching the library filters, as CSV. Without copy filters, games
/// with no copies get a row of their own.
pub fn query(filter: &LibraryFilter) -> QueryBuilder<Sqlite> {
    let mut qb = QueryBuilder::new(
        "SELECT copy_id, status, igdb_id, game_title, play_status,
                (SELECT group_concat(l.name, '; ' ORDER BY l.name COLLATE NOCASE)
                 FROM game_labels gl JOIN labels l ON l.id = gl.label_id
                 WHERE gl.game_id = rows.game_id) AS labels,
                platform, region,
                release_title, product_code, format, device, store, condition, notes,
                edition, purchased_on, price, currency, estimated_value, estimated_currency,
                barcode, bundle_copy_id
         FROM (
             SELECT c.id AS copy_id, c.status, g.igdb_id, g.title AS game_title, g.play_status,
                    p.name AS platform, r.region, r.title AS release_title, r.product_code,
                    c.format, d.name AS device, s.name AS store, c.condition, c.notes,
                    c.edition, c.purchased_on, c.price, c.currency, c.estimated_value,
                    c.estimated_currency, c.barcode, c.bundle_id AS bundle_copy_id,
                    g.sort_title, g.id AS game_id, p.sort_order AS platform_order
             FROM copies c
             JOIN releases r ON r.id = c.release_id
             JOIN games g ON g.id = r.game_id
             JOIN platforms p ON p.id = r.platform_id
             LEFT JOIN devices d ON d.id = c.device_id
             LEFT JOIN stores s ON s.id = c.store_id
             WHERE 1 = 1",
    );
    library::push_copy_filters(&mut qb, filter, library::DeviceMatch::Assigned);
    library::push_game_filters(&mut qb, filter);
    if !library::has_copy_filters(filter) {
        qb.push(
            " UNION ALL
             SELECT NULL, NULL, g.igdb_id, g.title, g.play_status,
                    NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL,
                    NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL,
                    g.sort_title, g.id, NULL
             FROM games g
             WHERE NOT EXISTS (
                 SELECT 1 FROM releases r JOIN copies c ON c.release_id = r.id
                 WHERE r.game_id = g.id)",
        );
        library::push_game_filters(&mut qb, filter);
    }
    qb.push(") AS rows ORDER BY sort_title, game_id, platform_order, region, copy_id");
    qb
}

/// A cell that starts with one of these runs as a formula in a spreadsheet.
pub const FORMULA_START: [char; 6] = ['=', '+', '-', '@', '\t', '\r'];

/// Free text that would start a formula gets a leading `'`, which
/// spreadsheets show as text and the import drops again.
fn plain(text: String) -> String {
    if text.starts_with(FORMULA_START) {
        format!("'{text}")
    } else {
        text
    }
}

pub async fn export(pool: &SqlitePool, filter: &LibraryFilter) -> Result<Vec<u8>, ExportError> {
    let mut filter = filter.clone();
    library::resolve_search(pool, &mut filter).await?;
    let rows: Vec<ExportRow> = query(&filter).build_query_as().fetch_all(pool).await?;

    let mut out = UTF8_BOM.to_vec();
    {
        let mut writer = csv::Writer::from_writer(&mut out);
        writer.write_record(COLUMNS)?;
        for row in rows {
            let id = |v: Option<i64>| v.map(|v| v.to_string()).unwrap_or_default();
            let text = |v: Option<String>| v.unwrap_or_default();
            let free = |v: Option<String>| plain(v.unwrap_or_default());
            let price = row
                .price
                .zip(row.currency.as_deref())
                .map(|(minor, currency)| money::format(minor, currency));
            let estimated_value = row
                .estimated_value
                .zip(row.estimated_currency.as_deref())
                .map(|(minor, currency)| money::format(minor, currency));
            writer.write_record([
                id(row.copy_id),
                text(row.status.map(|v| v.as_str().to_owned())),
                id(row.igdb_id),
                plain(row.game_title),
                text(row.play_status),
                free(row.labels),
                free(row.platform),
                text(row.region.map(|v| v.as_str().to_owned())),
                free(row.release_title),
                free(row.product_code),
                text(row.format.map(|v| v.as_str().to_owned())),
                free(row.edition),
                free(row.device),
                free(row.store),
                text(row.condition.map(|v| v.as_str().to_owned())),
                text(row.purchased_on),
                text(price),
                text(row.currency),
                text(estimated_value),
                text(row.estimated_currency),
                text(row.barcode),
                free(row.notes),
                id(row.bundle_copy_id),
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
