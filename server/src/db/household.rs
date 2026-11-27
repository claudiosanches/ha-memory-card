//! Everyone's play side by side, for the Household page.

use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use utoipa::{IntoParams, ToSchema};

use super::users::Person;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Period {
    #[default]
    Month,
    Year,
    All,
}

impl Period {
    /// The first day counted, as SQLite `date()` modifiers.
    fn start(self) -> &'static str {
        match self {
            Self::Month => "start of month",
            Self::Year => "start of year",
            Self::All => "-1000 years",
        }
    }
}

#[derive(Debug, Default, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct HouseholdQuery {
    #[param(inline)]
    pub period: Option<Period>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PersonTotals {
    pub person: Person,
    pub sessions: i64,
    pub minutes: i64,
    /// Days with at least one session.
    pub days: i64,
    /// Different games played.
    pub games: i64,
    /// Games they gave a status that counts as finished during the period,
    /// even if it changed since. An import's statuses do not count.
    pub finished: i64,
    /// Their most played game in the period, by sessions.
    pub top_game: Option<TopGame>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TopGame {
    pub game_id: i64,
    pub title: String,
    /// Relative to the covers URL.
    pub cover_path: Option<String>,
    pub sessions: i64,
}

/// Everyone who keeps play, the most minutes first.
pub async fn totals(pool: &SqlitePool, period: Period) -> sqlx::Result<Vec<PersonTotals>> {
    let start = period.start();
    let rows = sqlx::query!(
        r#"WITH played AS (
             SELECT * FROM play_sessions WHERE played_on >= date('now', ?1)
           )
           SELECT u.id AS "id!", COALESCE(u.display_name, u.name) AS name,
                  (SELECT COUNT(*) FROM played WHERE user_id = u.id) AS "sessions!: i64",
                  (SELECT COALESCE(SUM(minutes), 0) FROM played WHERE user_id = u.id) AS "minutes!: i64",
                  (SELECT COUNT(DISTINCT played_on) FROM played WHERE user_id = u.id) AS "days!: i64",
                  (SELECT COUNT(DISTINCT game_id) FROM played WHERE user_id = u.id) AS "games!: i64",
                  (SELECT COUNT(DISTINCT c.game_id)
                   FROM play_status_changes c JOIN play_statuses s ON s.key = c.play_status
                   WHERE c.user_id = u.id AND s.counts_as = 'finished' AND c.source != 'csv'
                     AND c.changed_at >= strftime('%Y-%m-%dT%H:%M:%fZ', 'now', ?1)) AS "finished!: i64"
           FROM users u
           WHERE EXISTS (SELECT 1 FROM play_progress WHERE user_id = u.id)
              OR EXISTS (SELECT 1 FROM play_sessions WHERE user_id = u.id)
           ORDER BY 4 DESC, 3 DESC, 2"#,
        start
    )
    .fetch_all(pool)
    .await?;

    let mut totals = Vec::with_capacity(rows.len());
    for row in rows {
        let top_game = sqlx::query_as!(
            TopGame,
            r#"SELECT g.id AS "game_id!", g.title, g.cover_path, COUNT(*) AS "sessions!: i64"
               FROM play_sessions s JOIN games g ON g.id = s.game_id
               WHERE s.user_id = ? AND s.played_on >= date('now', ?)
               GROUP BY g.id
               ORDER BY COUNT(*) DESC, SUM(s.minutes) DESC, g.sort_title
               LIMIT 1"#,
            row.id,
            start
        )
        .fetch_optional(pool)
        .await?;
        totals.push(PersonTotals {
            person: Person {
                id: row.id,
                name: row.name,
            },
            sessions: row.sessions,
            minutes: row.minutes,
            days: row.days,
            games: row.games,
            finished: row.finished,
            top_game,
        });
    }
    Ok(totals)
}
