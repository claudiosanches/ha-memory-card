//! What the Home Assistant sensors report. Counts cover owned copies, like the library.

use sqlx::SqlitePool;

/// Titles listed in the Playing sensor's attributes.
const PLAYING_LIMIT: i64 = 50;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    pub games: i64,
    pub copies: i64,
    pub wishlist: i64,
    pub backlog: i64,
    /// Games in a status that counts as playing, last played first.
    pub playing: Vec<String>,
    pub playing_count: i64,
    pub last_played: Option<LastPlayed>,
    pub last_added: Option<LastAdded>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LastPlayed {
    pub title: String,
    pub date: String,
    pub platform: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LastAdded {
    pub title: String,
    pub added_at: String,
    pub platform: String,
}

pub async fn snapshot(pool: &SqlitePool) -> sqlx::Result<Snapshot> {
    let counts = sqlx::query!(
        r#"SELECT
             (SELECT COUNT(DISTINCT r.game_id) FROM copies c JOIN releases r ON r.id = c.release_id
               WHERE c.status = 'owned') AS "games!: i64",
             (SELECT COUNT(*) FROM copies WHERE status = 'owned') AS "copies!: i64",
             (SELECT COUNT(*) FROM copies WHERE status = 'wanted') AS "wishlist!: i64",
             (SELECT COUNT(*) FROM games g JOIN play_statuses s ON s.key = g.play_status
               WHERE s.counts_as = 'backlog' AND EXISTS (
                 SELECT 1 FROM releases r JOIN copies c ON c.release_id = r.id
                  WHERE r.game_id = g.id AND c.status = 'owned')) AS "backlog!: i64",
             (SELECT COUNT(*) FROM games g JOIN play_statuses s ON s.key = g.play_status
               WHERE s.counts_as = 'playing' AND EXISTS (
                 SELECT 1 FROM releases r JOIN copies c ON c.release_id = r.id
                  WHERE r.game_id = g.id AND c.status = 'owned')) AS "playing!: i64""#
    )
    .fetch_one(pool)
    .await?;

    let playing = sqlx::query_scalar!(
        r#"SELECT g.title FROM games g JOIN play_statuses s ON s.key = g.play_status
            WHERE s.counts_as = 'playing' AND EXISTS (
              SELECT 1 FROM releases r JOIN copies c ON c.release_id = r.id
               WHERE r.game_id = g.id AND c.status = 'owned')
            ORDER BY (SELECT MAX(played_on) FROM play_sessions WHERE game_id = g.id) DESC NULLS LAST,
                     g.sort_title
            LIMIT ?"#,
        PLAYING_LIMIT
    )
    .fetch_all(pool)
    .await?;

    let last_played = sqlx::query!(
        r#"SELECT g.title, s.played_on AS date, p.name AS "platform?"
             FROM play_sessions s
             JOIN games g ON g.id = s.game_id
             LEFT JOIN copies c ON c.id = s.copy_id
             LEFT JOIN releases r ON r.id = c.release_id
             LEFT JOIN platforms p ON p.id = r.platform_id
            ORDER BY s.played_on DESC, s.id DESC
            LIMIT 1"#
    )
    .fetch_optional(pool)
    .await?
    .map(|row| LastPlayed {
        title: row.title,
        date: row.date,
        platform: row.platform,
    });

    let last_added = sqlx::query!(
        r#"SELECT g.title, c.created_at AS added_at, p.name AS platform
             FROM copies c
             JOIN releases r ON r.id = c.release_id
             JOIN games g ON g.id = r.game_id
             JOIN platforms p ON p.id = r.platform_id
            WHERE c.status = 'owned'
            ORDER BY c.created_at DESC, c.id DESC
            LIMIT 1"#
    )
    .fetch_optional(pool)
    .await?
    .map(|row| LastAdded {
        title: row.title,
        added_at: row.added_at,
        platform: row.platform,
    });

    Ok(Snapshot {
        games: counts.games,
        copies: counts.copies,
        wishlist: counts.wishlist,
        backlog: counts.backlog,
        playing,
        playing_count: counts.playing,
        last_played,
        last_added,
    })
}
