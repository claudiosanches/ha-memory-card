use sqlx::SqliteConnection;

use std::collections::HashSet;

use crate::db::games::sync_contents;
use crate::db::progress::{self, Replace, Source};
use crate::db::slugs::{self, Table};
use crate::domain::sort_title;
use crate::transfer::plan::{Plan, Ref};

fn resolve(r: Ref, created: &[i64]) -> i64 {
    match r {
        Ref::Existing(id) => id,
        Ref::New(i) => created[i],
    }
}

/// Writes the plan. The caller owns the transaction and must not apply a
/// plan that has errors.
pub async fn apply(conn: &mut SqliteConnection, plan: &Plan, by: &str) -> sqlx::Result<()> {
    let mut platforms = Vec::with_capacity(plan.platforms.len());
    let mut slugs = slugs::Allocator::new(Table::Platforms);
    let mut sort_order =
        sqlx::query_scalar!(r#"SELECT COALESCE(MAX(sort_order), 0) AS "max!: i64" FROM platforms"#)
            .fetch_one(&mut *conn)
            .await?;
    for name in &plan.platforms {
        let slug = slugs.free(conn, name).await?;
        sort_order += 10;
        let id = sqlx::query_scalar!(
            r#"INSERT INTO platforms (name, slug, active, custom, sort_order)
               VALUES (?, ?, 1, 1, ?)
               RETURNING id AS "id!""#,
            name,
            slug,
            sort_order
        )
        .fetch_one(&mut *conn)
        .await?;
        platforms.push(id);
    }

    let mut stores = Vec::with_capacity(plan.stores.len());
    let mut slugs = slugs::Allocator::new(Table::Stores);
    for name in &plan.stores {
        let slug = slugs.free(conn, name).await?;
        let id = sqlx::query_scalar!(
            r#"INSERT INTO stores (name, slug) VALUES (?, ?) RETURNING id AS "id!""#,
            name,
            slug
        )
        .fetch_one(&mut *conn)
        .await?;
        stores.push(id);
    }

    let mut devices = Vec::with_capacity(plan.devices.len());
    let mut slugs = slugs::Allocator::new(Table::Devices);
    for device in &plan.devices {
        let slug = slugs.free(conn, &device.name).await?;
        let id = sqlx::query_scalar!(
            r#"INSERT INTO devices (name, slug, kind) VALUES (?, ?, 'console') RETURNING id AS "id!""#,
            device.name,
            slug
        )
        .fetch_one(&mut *conn)
        .await?;
        for platform in &device.platforms {
            let platform_id = resolve(*platform, &platforms);
            sqlx::query!(
                "INSERT OR IGNORE INTO device_platforms (device_id, platform_id) VALUES (?, ?)",
                id,
                platform_id
            )
            .execute(&mut *conn)
            .await?;
        }
        devices.push(id);
    }

    let mut accounts = Vec::with_capacity(plan.accounts.len());
    let mut slugs = slugs::Allocator::new(Table::Accounts);
    for account in &plan.accounts {
        let slug = slugs.free(conn, &account.name).await?;
        let id = sqlx::query_scalar!(
            r#"INSERT INTO accounts (name, slug) VALUES (?, ?) RETURNING id AS "id!""#,
            account.name,
            slug
        )
        .fetch_one(&mut *conn)
        .await?;
        for platform in &account.platforms {
            let platform_id = resolve(*platform, &platforms);
            sqlx::query!(
                "INSERT OR IGNORE INTO account_platforms (account_id, platform_id) VALUES (?, ?)",
                id,
                platform_id
            )
            .execute(&mut *conn)
            .await?;
        }
        accounts.push(id);
    }

    let mut games = Vec::with_capacity(plan.games.len());
    for game in &plan.games {
        let sort = sort_title(&game.title);
        let id = sqlx::query_scalar!(
            r#"INSERT INTO games (igdb_id, title, sort_title, created_by, updated_by)
               VALUES (?, ?, ?, ?, ?) RETURNING id AS "id!""#,
            game.igdb_id,
            game.title,
            sort,
            by,
            by
        )
        .fetch_one(&mut *conn)
        .await?;
        if let Some(status) = &game.play_status {
            set_status(conn, by, id, status).await?;
        }
        games.push(id);
    }
    let mut labels = Vec::with_capacity(plan.labels.len());
    let mut slugs = slugs::Allocator::new(Table::Labels);
    for name in &plan.labels {
        let slug = slugs.free(conn, name).await?;
        let id = sqlx::query_scalar!(
            r#"INSERT INTO labels (name, slug) VALUES (?, ?) RETURNING id AS "id!""#,
            name,
            slug
        )
        .fetch_one(&mut *conn)
        .await?;
        labels.push(id);
    }
    for (game, label) in &plan.game_labels {
        let game_id = resolve(*game, &games);
        let label_id = resolve(*label, &labels);
        sqlx::query!(
            "INSERT OR IGNORE INTO game_labels (game_id, label_id) VALUES (?, ?)",
            game_id,
            label_id
        )
        .execute(&mut *conn)
        .await?;
    }

    for (id, status) in &plan.game_statuses {
        set_status(conn, by, *id, status).await?;
    }
    for (id, igdb_id) in &plan.game_igdb_links {
        sqlx::query!(
            "UPDATE games SET igdb_id = ?, updated_by = ?,
                              updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
             WHERE id = ?",
            igdb_id,
            by,
            id
        )
        .execute(&mut *conn)
        .await?;
    }

    let mut releases = Vec::with_capacity(plan.releases.len());
    for release in &plan.releases {
        let game_id = resolve(release.game, &games);
        let platform_id = resolve(release.platform, &platforms);
        let id = sqlx::query_scalar!(
            r#"INSERT INTO releases (game_id, platform_id, region, product_code, title)
               VALUES (?, ?, ?, ?, ?) RETURNING id AS "id!""#,
            game_id,
            platform_id,
            release.region,
            release.product_code,
            release.title
        )
        .fetch_one(&mut *conn)
        .await?;
        releases.push(id);
    }
    for (id, title) in &plan.release_titles {
        sqlx::query!(
            "UPDATE releases SET title = ?, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
             WHERE id = ?",
            title,
            id
        )
        .execute(&mut *conn)
        .await?;
    }

    for copy in &plan.copies {
        let release_id = resolve(copy.release, &releases);
        let device_id = copy.device.map(|d| resolve(d, &devices));
        let account_id = copy.account.map(|a| resolve(a, &accounts));
        let store_id = copy.store.map(|s| resolve(s, &stores));
        match copy.existing {
            None => {
                sqlx::query!(
                    "INSERT INTO copies (release_id, status, device_id, account_id, format, store_id,
                                         condition, notes, edition, purchased_on, price, currency,
                                         estimated_value, estimated_currency, barcode,
                                         bundle_id, created_by, updated_by, source)
                     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'csv')",
                    release_id,
                    copy.status,
                    device_id,
                    account_id,
                    copy.format,
                    store_id,
                    copy.condition,
                    copy.notes,
                    copy.edition,
                    copy.purchased_on,
                    copy.price,
                    copy.currency,
                    copy.estimated_value,
                    copy.estimated_currency,
                    copy.barcode,
                    copy.bundle,
                    by,
                    by
                )
                .execute(&mut *conn)
                .await?;
            }
            Some(id) => {
                sqlx::query!(
                    "UPDATE copies
                     SET release_id = ?, status = ?, device_id = ?, account_id = ?, format = ?,
                         store_id = ?,
                         condition = ?, notes = ?, edition = ?, purchased_on = ?, price = ?,
                         currency = ?, estimated_value = ?, estimated_currency = ?,
                         barcode = ?, bundle_id = ?, updated_by = ?,
                         updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
                     WHERE id = ?",
                    release_id,
                    copy.status,
                    device_id,
                    account_id,
                    copy.format,
                    store_id,
                    copy.condition,
                    copy.notes,
                    copy.edition,
                    copy.purchased_on,
                    copy.price,
                    copy.currency,
                    copy.estimated_value,
                    copy.estimated_currency,
                    copy.barcode,
                    copy.bundle,
                    by,
                    id
                )
                .execute(&mut *conn)
                .await?;
                // A copy moved to another game no longer stands for the
                // first game's sessions.
                sqlx::query!(
                    "UPDATE play_sessions SET copy_id = NULL
                     WHERE copy_id = ?
                       AND game_id IS NOT (SELECT game_id FROM releases WHERE id = ?)",
                    id,
                    release_id
                )
                .execute(&mut *conn)
                .await?;
            }
        }
    }

    // Contents follow their bundle copy, whichever side the rows changed.
    let touched: HashSet<i64> = plan
        .copies
        .iter()
        .flat_map(|copy| [copy.bundle, copy.existing])
        .flatten()
        .collect();
    let bundles: Vec<i64> = sqlx::query_scalar!(
        r#"SELECT DISTINCT bundle_id AS "bundle_id!" FROM copies WHERE bundle_id IS NOT NULL"#
    )
    .fetch_all(&mut *conn)
    .await?;
    for bundle in bundles.into_iter().filter(|b| touched.contains(b)) {
        sync_contents(conn, bundle).await?;
    }

    Ok(())
}

/// The play status column is the importing person's own.
async fn set_status(
    conn: &mut SqliteConnection,
    by: &str,
    game_id: i64,
    status: &str,
) -> sqlx::Result<()> {
    progress::put(conn, by, game_id, status, Source::Csv, Replace::Any).await
}
