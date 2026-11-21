//! Collection totals for the stats page. Counts cover owned copies unless named
//! otherwise; a bundle counts as one copy, not one per game inside it.

use serde::Serialize;
use sqlx::SqlitePool;
use utoipa::ToSchema;

use crate::money;

/// Played games listed under most played.
const MOST_PLAYED: i64 = 10;

#[derive(Debug, Serialize, ToSchema)]
pub struct Stats {
    /// Games with an owned copy.
    pub games: i64,
    pub copies: i64,
    pub physical: i64,
    pub digital: i64,
    /// Wanted copies.
    pub wishlist: i64,
    /// What owned copies cost, per currency, largest first.
    pub spent: Vec<Amount>,
    /// Owned copies without a price.
    pub unpriced: i64,
    /// What wanted copies are expected to cost, per currency.
    pub wishlist_value: Vec<Amount>,
    /// By the year of the purchase date; copies without one are left out.
    pub spent_by_year: Vec<YearAmount>,
    /// Owned games per play status, in play status order; `None` is not set.
    pub play_statuses: Vec<StatusCount>,
    /// In platform list order.
    pub platforms: Vec<PlatformTotal>,
    /// Largest first; `None` is copies with no store.
    pub stores: Vec<StoreCount>,
    /// Sessions in the last 12 months, by `YYYY-MM`; months without play are left out.
    pub sessions_by_month: Vec<MonthPlay>,
    pub total_sessions: i64,
    pub total_minutes: i64,
    /// By session count, then minutes.
    pub most_played: Vec<PlayedGame>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct Amount {
    pub currency: String,
    /// Decimal text, such as `59.99`.
    pub amount: String,
    pub copies: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct YearAmount {
    pub year: String,
    pub currency: String,
    pub amount: String,
    pub copies: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct StatusCount {
    pub play_status: Option<String>,
    pub games: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PlatformTotal {
    pub platform_id: i64,
    pub games: i64,
    pub copies: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct StoreCount {
    pub store_id: Option<i64>,
    pub copies: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MonthPlay {
    pub month: String,
    pub sessions: i64,
    pub minutes: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PlayedGame {
    pub game_id: i64,
    pub title: String,
    pub sessions: i64,
    pub minutes: i64,
}

fn amount(currency: String, minor: i64, copies: i64) -> Amount {
    Amount {
        amount: money::format(minor, &currency),
        currency,
        copies,
    }
}

pub async fn get(pool: &SqlitePool) -> sqlx::Result<Stats> {
    let counts = sqlx::query!(
        r#"SELECT
             (SELECT COUNT(DISTINCT r.game_id) FROM releases r
                JOIN copies c ON c.release_id = r.id WHERE c.status = 'owned') AS "games!: i64",
             COUNT(*) FILTER (WHERE status = 'owned') AS "copies!: i64",
             COUNT(*) FILTER (WHERE status = 'owned' AND format = 'physical') AS "physical!: i64",
             COUNT(*) FILTER (WHERE status = 'owned' AND format = 'digital') AS "digital!: i64",
             COUNT(*) FILTER (WHERE status = 'wanted') AS "wishlist!: i64",
             COUNT(*) FILTER (WHERE status = 'owned' AND price IS NULL) AS "unpriced!: i64"
           FROM copies WHERE bundle_id IS NULL"#
    )
    .fetch_one(pool)
    .await?;

    let totals = sqlx::query!(
        r#"SELECT status, currency AS "currency!", SUM(price) AS "minor!: i64",
                  COUNT(*) AS "copies!: i64"
           FROM copies WHERE price IS NOT NULL AND currency IS NOT NULL
           GROUP BY status, currency
           ORDER BY COUNT(*) DESC, currency"#
    )
    .fetch_all(pool)
    .await?;
    let (mut spent, mut wishlist_value) = (Vec::new(), Vec::new());
    for row in totals {
        let target = if row.status == "owned" {
            &mut spent
        } else {
            &mut wishlist_value
        };
        target.push(amount(row.currency, row.minor, row.copies));
    }

    let spent_by_year = sqlx::query!(
        r#"SELECT substr(purchased_on, 1, 4) AS "year!: String", currency AS "currency!",
                  SUM(price) AS "minor!: i64", COUNT(*) AS "copies!: i64"
           FROM copies
           WHERE status = 'owned' AND purchased_on IS NOT NULL
             AND price IS NOT NULL AND currency IS NOT NULL
           GROUP BY 1, currency
           ORDER BY 1, currency"#
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|row| YearAmount {
        amount: money::format(row.minor, &row.currency),
        year: row.year,
        currency: row.currency,
        copies: row.copies,
    })
    .collect();

    let play_statuses = sqlx::query_as!(
        StatusCount,
        r#"SELECT g.play_status, COUNT(*) AS "games!: i64"
           FROM games g LEFT JOIN play_statuses s ON s.key = g.play_status
           WHERE EXISTS (SELECT 1 FROM releases r JOIN copies c ON c.release_id = r.id
                         WHERE r.game_id = g.id AND c.status = 'owned')
           GROUP BY g.play_status
           ORDER BY g.play_status IS NULL, MIN(s.sort_order)"#
    )
    .fetch_all(pool)
    .await?;

    let platforms = sqlx::query_as!(
        PlatformTotal,
        r#"SELECT r.platform_id AS "platform_id!", COUNT(DISTINCT r.game_id) AS "games!: i64",
                  COUNT(*) AS "copies!: i64"
           FROM copies c JOIN releases r ON r.id = c.release_id
           JOIN platforms p ON p.id = r.platform_id
           WHERE c.status = 'owned' AND c.bundle_id IS NULL
           GROUP BY r.platform_id
           ORDER BY MIN(p.sort_order), MIN(p.name)"#
    )
    .fetch_all(pool)
    .await?;

    let stores = sqlx::query_as!(
        StoreCount,
        r#"SELECT store_id AS "store_id?", COUNT(*) AS "copies!: i64"
           FROM copies WHERE status = 'owned' AND bundle_id IS NULL
           GROUP BY store_id
           ORDER BY COUNT(*) DESC, store_id IS NULL, store_id"#
    )
    .fetch_all(pool)
    .await?;

    let sessions_by_month = sqlx::query_as!(
        MonthPlay,
        r#"SELECT substr(played_on, 1, 7) AS "month!: String", COUNT(*) AS "sessions!: i64",
                  COALESCE(SUM(minutes), 0) AS "minutes!: i64"
           FROM play_sessions
           WHERE played_on >= date('now', 'start of month', '-11 months')
           GROUP BY 1 ORDER BY 1"#
    )
    .fetch_all(pool)
    .await?;

    let play = sqlx::query!(
        r#"SELECT COUNT(*) AS "sessions!: i64", COALESCE(SUM(minutes), 0) AS "minutes!: i64"
           FROM play_sessions"#
    )
    .fetch_one(pool)
    .await?;

    let most_played = sqlx::query_as!(
        PlayedGame,
        r#"SELECT g.id AS "game_id!", g.title, COUNT(*) AS "sessions!: i64",
                  COALESCE(SUM(s.minutes), 0) AS "minutes!: i64"
           FROM play_sessions s JOIN games g ON g.id = s.game_id
           GROUP BY g.id
           ORDER BY COUNT(*) DESC, SUM(s.minutes) DESC, g.sort_title
           LIMIT ?"#,
        MOST_PLAYED
    )
    .fetch_all(pool)
    .await?;

    Ok(Stats {
        games: counts.games,
        copies: counts.copies,
        physical: counts.physical,
        digital: counts.digital,
        wishlist: counts.wishlist,
        spent,
        unpriced: counts.unpriced,
        wishlist_value,
        spent_by_year,
        play_statuses,
        platforms,
        stores,
        sessions_by_month,
        total_sessions: play.sessions,
        total_minutes: play.minutes,
        most_played,
    })
}
