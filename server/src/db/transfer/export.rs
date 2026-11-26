use sqlx::SqlitePool;

use crate::db::library::{self, LibraryFilter};
use crate::db::sql::Sql;
use crate::domain::{Condition, CopyStatus, Format, Region};

/// One CSV row: a copy, or a game with no copies.
#[derive(sqlx::FromRow)]
pub struct ExportRow {
    pub copy_id: Option<i64>,
    pub status: Option<CopyStatus>,
    pub igdb_id: Option<i64>,
    pub game_title: String,
    pub play_status: Option<String>,
    pub labels: Option<String>,
    pub platform: Option<String>,
    pub region: Option<Region>,
    pub release_title: Option<String>,
    pub product_code: Option<String>,
    pub format: Option<Format>,
    pub device: Option<String>,
    pub account: Option<String>,
    pub store: Option<String>,
    pub condition: Option<Condition>,
    pub notes: Option<String>,
    pub edition: Option<String>,
    pub purchased_on: Option<String>,
    pub price: Option<i64>,
    pub currency: Option<String>,
    pub estimated_value: Option<i64>,
    pub estimated_currency: Option<String>,
    pub barcode: Option<String>,
    pub bundle_copy_id: Option<i64>,
}

/// Copies matching the library filters. Without copy filters, games with no
/// copies get a row of their own.
pub fn query(filter: &LibraryFilter) -> Sql {
    let mut qb = Sql::new(
        "SELECT copy_id, status, igdb_id, game_title, play_status,
                (SELECT group_concat(l.name, '; ' ORDER BY l.name COLLATE NOCASE)
                 FROM game_labels gl JOIN labels l ON l.id = gl.label_id
                 WHERE gl.game_id = rows.game_id) AS labels,
                platform, region,
                release_title, product_code, format, device, account, store, condition, notes,
                edition, purchased_on, price, currency, estimated_value, estimated_currency,
                barcode, bundle_copy_id
         FROM (
             SELECT c.id AS copy_id, c.status, g.igdb_id, g.title AS game_title,
                    (SELECT play_status FROM play_progress WHERE user_id = ",
    );
    qb.push_bind(filter.person.clone()).push(
        " AND game_id = g.id) AS play_status,
                    p.name AS platform, r.region, r.title AS release_title, r.product_code,
                    c.format, d.name AS device, a.name AS account, s.name AS store, c.condition, c.notes,
                    c.edition, c.purchased_on, c.price, c.currency, c.estimated_value,
                    c.estimated_currency, c.barcode, c.bundle_id AS bundle_copy_id,
                    g.sort_title, g.id AS game_id, p.sort_order AS platform_order
             FROM copies c
             JOIN releases r ON r.id = c.release_id
             JOIN games g ON g.id = r.game_id
             JOIN platforms p ON p.id = r.platform_id
             LEFT JOIN devices d ON d.id = c.device_id
             LEFT JOIN accounts a ON a.id = c.account_id
             LEFT JOIN stores s ON s.id = c.store_id
             WHERE 1 = 1",
    );
    library::push_copy_filters(&mut qb, filter, library::DeviceMatch::Assigned);
    library::push_game_filters(&mut qb, filter);
    if !library::has_copy_filters(filter) {
        qb.push(
            " UNION ALL
             SELECT NULL, NULL, g.igdb_id, g.title,
                    (SELECT play_status FROM play_progress WHERE user_id = ",
        );
        qb.push_bind(filter.person.clone()).push(
            " AND game_id = g.id),
                    NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL,
                    NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL,
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

pub async fn rows(pool: &SqlitePool, filter: &LibraryFilter) -> sqlx::Result<Vec<ExportRow>> {
    let mut filter = filter.clone();
    library::resolve_search(pool, &mut filter).await?;
    query(&filter).build_query_as().fetch_all(pool).await
}
