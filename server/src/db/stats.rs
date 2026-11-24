//! Collection totals for the stats page. Counts cover owned copies unless named
//! otherwise; a bundle counts as one copy, not one per game inside it.

use serde::Serialize;
use sqlx::SqlitePool;
use utoipa::ToSchema;

use crate::domain::{Condition, Region};
use crate::money;

/// Played games listed under most played.
const MOST_PLAYED: i64 = 10;
/// Copies listed under most valuable, per currency.
const MOST_VALUABLE: i64 = 5;

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
    /// What owned copies would sell for now, per currency, largest first.
    pub estimated: Vec<Amount>,
    /// Owned copies without an estimated value.
    pub unvalued: i64,
    /// By the year of the purchase date; copies without one are left out.
    pub spent_by_year: Vec<YearAmount>,
    /// Owned games per the person's play status, in play status order; `None`
    /// is not set.
    pub play_statuses: Vec<StatusCount>,
    /// In platform list order.
    pub platforms: Vec<PlatformTotal>,
    /// Largest first; `None` is copies with no store.
    pub stores: Vec<StoreCount>,
    /// Largest first.
    pub regions: Vec<RegionCount>,
    /// Physical copies only, largest first; `None` is not set.
    pub conditions: Vec<ConditionCount>,
    /// Copies added in the last 12 months, by `YYYY-MM` of the purchase date
    /// or, without one, the day the copy was added. Empty months are left out.
    pub added_by_month: Vec<PeriodCount>,
    /// The same, by `YYYY`, for every year.
    pub added_by_year: Vec<PeriodCount>,
    /// Copies priced and valued in the same currency: what they cost against
    /// what they are worth now.
    pub value_change: Vec<ValueChange>,
    /// The copies worth the most, up to five per currency, largest first.
    pub most_valuable: Vec<ValuableCopy>,
    /// The person's play in the last 53 weeks, by `YYYY-MM-DD`; days without
    /// play are left out.
    pub play_days: Vec<DayPlay>,
    /// The person's sessions and minutes, all time.
    pub total_sessions: i64,
    pub total_minutes: i64,
    /// The person's, by session count, then minutes.
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
    pub physical: i64,
    pub digital: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct StoreCount {
    pub store_id: Option<i64>,
    pub copies: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RegionCount {
    pub region: Region,
    pub copies: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ConditionCount {
    pub condition: Option<Condition>,
    pub copies: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PeriodCount {
    pub period: String,
    pub copies: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ValueChange {
    pub currency: String,
    pub paid: String,
    pub estimated: String,
    /// Estimated minus paid; negative when the copies are worth less.
    pub change: String,
    pub copies: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ValuableCopy {
    pub copy_id: i64,
    pub game_id: i64,
    pub title: String,
    pub platform_id: i64,
    pub currency: String,
    pub estimated: String,
    /// What it cost, when priced in the same currency.
    pub paid: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DayPlay {
    pub day: String,
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

/// The household's collection, with `person`'s own play.
pub async fn get(pool: &SqlitePool, person: &str) -> sqlx::Result<Stats> {
    let counts = sqlx::query!(
        r#"SELECT
             (SELECT COUNT(DISTINCT r.game_id) FROM releases r
                JOIN copies c ON c.release_id = r.id WHERE c.status = 'owned') AS "games!: i64",
             COUNT(*) FILTER (WHERE status = 'owned') AS "copies!: i64",
             COUNT(*) FILTER (WHERE status = 'owned' AND format = 'physical') AS "physical!: i64",
             COUNT(*) FILTER (WHERE status = 'owned' AND format = 'digital') AS "digital!: i64",
             COUNT(*) FILTER (WHERE status = 'wanted') AS "wishlist!: i64",
             COUNT(*) FILTER (WHERE status = 'owned' AND price IS NULL) AS "unpriced!: i64",
             COUNT(*) FILTER (WHERE status = 'owned' AND estimated_value IS NULL)
               AS "unvalued!: i64"
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
    let estimated = sqlx::query!(
        r#"SELECT estimated_currency AS "currency!", SUM(estimated_value) AS "minor!: i64",
                  COUNT(*) AS "copies!: i64"
           FROM copies
           WHERE status = 'owned' AND estimated_value IS NOT NULL
             AND estimated_currency IS NOT NULL
           GROUP BY estimated_currency
           ORDER BY COUNT(*) DESC, estimated_currency"#
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|row| amount(row.currency, row.minor, row.copies))
    .collect();

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
        r#"SELECT p.play_status AS "play_status: String", COUNT(*) AS "games!: i64"
           FROM games g
           LEFT JOIN play_progress p ON p.game_id = g.id AND p.user_id = ?
           LEFT JOIN play_statuses s ON s.key = p.play_status
           WHERE EXISTS (SELECT 1 FROM releases r JOIN copies c ON c.release_id = r.id
                         WHERE r.game_id = g.id AND c.status = 'owned')
           GROUP BY p.play_status
           ORDER BY p.play_status IS NULL, MIN(s.sort_order)"#,
        person
    )
    .fetch_all(pool)
    .await?;

    let platforms = sqlx::query_as!(
        PlatformTotal,
        r#"SELECT r.platform_id AS "platform_id!", COUNT(DISTINCT r.game_id) AS "games!: i64",
                  COUNT(*) AS "copies!: i64",
                  COUNT(*) FILTER (WHERE c.format = 'physical') AS "physical!: i64",
                  COUNT(*) FILTER (WHERE c.format = 'digital') AS "digital!: i64"
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

    let regions = sqlx::query_as!(
        RegionCount,
        r#"SELECT r.region AS "region!: Region", COUNT(*) AS "copies!: i64"
           FROM copies c JOIN releases r ON r.id = c.release_id
           WHERE c.status = 'owned' AND c.bundle_id IS NULL
           GROUP BY r.region
           ORDER BY COUNT(*) DESC, r.region"#
    )
    .fetch_all(pool)
    .await?;

    let conditions = sqlx::query_as!(
        ConditionCount,
        r#"SELECT condition AS "condition?: Condition", COUNT(*) AS "copies!: i64"
           FROM copies
           WHERE status = 'owned' AND format = 'physical' AND bundle_id IS NULL
           GROUP BY condition
           ORDER BY COUNT(*) DESC, condition IS NULL, condition"#
    )
    .fetch_all(pool)
    .await?;

    let added_by_month = sqlx::query_as!(
        PeriodCount,
        r#"SELECT substr(COALESCE(purchased_on, created_at), 1, 7) AS "period!: String",
                  COUNT(*) AS "copies!: i64"
           FROM copies
           WHERE status = 'owned' AND bundle_id IS NULL
             AND COALESCE(purchased_on, created_at) >= date('now', 'start of month', '-11 months')
           GROUP BY 1 ORDER BY 1"#
    )
    .fetch_all(pool)
    .await?;

    let added_by_year = sqlx::query_as!(
        PeriodCount,
        r#"SELECT substr(COALESCE(purchased_on, created_at), 1, 4) AS "period!: String",
                  COUNT(*) AS "copies!: i64"
           FROM copies
           WHERE status = 'owned' AND bundle_id IS NULL
           GROUP BY 1 ORDER BY 1"#
    )
    .fetch_all(pool)
    .await?;

    let value_change = sqlx::query!(
        r#"SELECT currency AS "currency!", SUM(price) AS "paid!: i64",
                  SUM(estimated_value) AS "estimated!: i64", COUNT(*) AS "copies!: i64"
           FROM copies
           WHERE status = 'owned' AND price IS NOT NULL AND estimated_value IS NOT NULL
             AND currency = estimated_currency
           GROUP BY currency
           ORDER BY COUNT(*) DESC, currency"#
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|row| ValueChange {
        paid: money::format(row.paid, &row.currency),
        estimated: money::format(row.estimated, &row.currency),
        change: money::format(row.estimated - row.paid, &row.currency),
        currency: row.currency,
        copies: row.copies,
    })
    .collect();

    let most_valuable = sqlx::query!(
        r#"SELECT copy_id AS "copy_id!", game_id AS "game_id!", title AS "title!",
                  platform_id AS "platform_id!", currency AS "currency!",
                  estimated AS "estimated!: i64", paid AS "paid?: i64"
           FROM (
               SELECT c.id AS copy_id, g.id AS game_id, g.title, r.platform_id,
                      c.estimated_currency AS currency, c.estimated_value AS estimated,
                      CASE WHEN c.currency = c.estimated_currency THEN c.price END AS paid,
                      ROW_NUMBER() OVER (PARTITION BY c.estimated_currency
                                         ORDER BY c.estimated_value DESC, g.sort_title) AS rank
               FROM copies c
               JOIN releases r ON r.id = c.release_id
               JOIN games g ON g.id = r.game_id
               WHERE c.status = 'owned' AND c.estimated_value IS NOT NULL
                 AND c.estimated_currency IS NOT NULL
           )
           WHERE rank <= ?
           ORDER BY currency, rank"#,
        MOST_VALUABLE
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|row| ValuableCopy {
        estimated: money::format(row.estimated, &row.currency),
        paid: row.paid.map(|minor| money::format(minor, &row.currency)),
        copy_id: row.copy_id,
        game_id: row.game_id,
        title: row.title,
        platform_id: row.platform_id,
        currency: row.currency,
    })
    .collect();

    let play_days = sqlx::query_as!(
        DayPlay,
        r#"SELECT played_on AS "day!: String", COUNT(*) AS "sessions!: i64",
                  COALESCE(SUM(minutes), 0) AS "minutes!: i64"
           FROM play_sessions
           WHERE user_id = ? AND played_on >= date('now', '-371 days')
           GROUP BY played_on ORDER BY played_on"#,
        person
    )
    .fetch_all(pool)
    .await?;

    let play = sqlx::query!(
        r#"SELECT COUNT(*) AS "sessions!: i64", COALESCE(SUM(minutes), 0) AS "minutes!: i64"
           FROM play_sessions WHERE user_id = ?"#,
        person
    )
    .fetch_one(pool)
    .await?;

    let most_played = sqlx::query_as!(
        PlayedGame,
        r#"SELECT g.id AS "game_id!", g.title, COUNT(*) AS "sessions!: i64",
                  COALESCE(SUM(s.minutes), 0) AS "minutes!: i64"
           FROM play_sessions s JOIN games g ON g.id = s.game_id
           WHERE s.user_id = ?
           GROUP BY g.id
           ORDER BY COUNT(*) DESC, SUM(s.minutes) DESC, g.sort_title
           LIMIT ?"#,
        person,
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
        estimated,
        unvalued: counts.unvalued,
        spent_by_year,
        play_statuses,
        platforms,
        stores,
        regions,
        conditions,
        added_by_month,
        added_by_year,
        value_change,
        most_valuable,
        play_days,
        total_sessions: play.sessions,
        total_minutes: play.minutes,
        most_played,
    })
}
