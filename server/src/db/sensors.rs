//! What the Home Assistant sensors report. Counts cover owned copies, like the library.

use sqlx::SqlitePool;

/// Titles listed in the household Playing sensor's attributes.
const PLAYING_LIMIT: i64 = 50;
/// Titles listed in each person's Playing sensor, so the feed's state line
/// stays short with every person in it.
const PERSON_PLAYING_LIMIT: i64 = 10;
/// People who get sensors of their own.
pub const PEOPLE_MAX: i64 = 16;
/// Platforms listed for the pick, as many as the integration keeps.
pub const PLATFORMS_LIMIT: i64 = 50;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Snapshot {
    pub games: i64,
    pub copies: i64,
    pub wishlist: i64,
    /// Games someone is playing, the latest played first.
    pub playing: Vec<String>,
    pub playing_count: i64,
    pub last_added: Option<LastAdded>,
    /// The latest pick, from the app or Home Assistant.
    pub pick: Option<Pick>,
    /// Everyone who keeps play, by user ID.
    pub people: Vec<PersonPlay>,
}

/// One person's play, for their own sensors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersonPlay {
    pub id: String,
    pub name: Option<String>,
    pub backlog: i64,
    /// Their games in a status that counts as playing, the latest played first.
    pub playing: Vec<String>,
    pub playing_count: i64,
    pub last_played: Option<LastPlayed>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pick {
    pub title: String,
    pub suggested_at: String,
    /// Platforms of its owned copies, in platform list order.
    pub platforms: Vec<String>,
    /// Whose pick it is.
    pub person: Option<String>,
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

/// Platforms of a game's owned copies, in platform list order.
pub async fn owned_platforms(pool: &SqlitePool, game_id: i64) -> sqlx::Result<Vec<String>> {
    sqlx::query_scalar!(
        r#"SELECT p.name FROM platforms p
           WHERE EXISTS (SELECT 1 FROM releases r JOIN copies c ON c.release_id = r.id
                         WHERE r.platform_id = p.id AND r.game_id = ? AND c.status = 'owned')
           ORDER BY p.sort_order, p.name
           LIMIT ?"#,
        game_id,
        PLATFORMS_LIMIT
    )
    .fetch_all(pool)
    .await
}

pub async fn snapshot(pool: &SqlitePool) -> sqlx::Result<Snapshot> {
    let counts = sqlx::query!(
        r#"SELECT
             (SELECT COUNT(DISTINCT r.game_id) FROM copies c JOIN releases r ON r.id = c.release_id
               WHERE c.status = 'owned') AS "games!: i64",
             (SELECT COUNT(*) FROM copies WHERE status = 'owned' AND bundle_id IS NULL) AS "copies!: i64",
             (SELECT COUNT(*) FROM copies WHERE status = 'wanted' AND bundle_id IS NULL) AS "wishlist!: i64",
             (SELECT COUNT(*) FROM games g
               WHERE EXISTS (SELECT 1 FROM play_progress p JOIN play_statuses s ON s.key = p.play_status
                              WHERE p.game_id = g.id AND s.counts_as = 'playing')
                 AND EXISTS (SELECT 1 FROM releases r JOIN copies c ON c.release_id = r.id
                              WHERE r.game_id = g.id AND c.status = 'owned')) AS "playing!: i64""#
    )
    .fetch_one(pool)
    .await?;

    let playing = sqlx::query_scalar!(
        r#"SELECT g.title FROM games g
            WHERE EXISTS (SELECT 1 FROM play_progress p JOIN play_statuses s ON s.key = p.play_status
                           WHERE p.game_id = g.id AND s.counts_as = 'playing')
              AND EXISTS (SELECT 1 FROM releases r JOIN copies c ON c.release_id = r.id
                           WHERE r.game_id = g.id AND c.status = 'owned')
            ORDER BY (SELECT MAX(played_on) FROM play_sessions WHERE game_id = g.id) DESC NULLS LAST,
                     g.sort_title
            LIMIT ?"#,
        PLAYING_LIMIT
    )
    .fetch_all(pool)
    .await?;

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

    let pick = match sqlx::query!(
        r#"SELECT g.id AS "id!", g.title, s.suggested_at,
                  COALESCE(u.display_name, u.name) AS person
           FROM suggestions s JOIN games g ON g.id = s.game_id
           LEFT JOIN users u ON u.id = s.user_id
           ORDER BY s.suggested_at DESC, s.id DESC LIMIT 1"#
    )
    .fetch_optional(pool)
    .await?
    {
        None => None,
        Some(row) => Some(Pick {
            platforms: owned_platforms(pool, row.id).await?,
            title: row.title,
            suggested_at: row.suggested_at,
            person: row.person,
        }),
    };

    let mut people = Vec::new();
    for person in sqlx::query!(
        r#"SELECT u.id AS "id!", COALESCE(u.display_name, u.name) AS name
           FROM users u
           WHERE EXISTS (SELECT 1 FROM play_progress WHERE user_id = u.id)
              OR EXISTS (SELECT 1 FROM play_sessions WHERE user_id = u.id)
           ORDER BY u.id
           LIMIT ?"#,
        PEOPLE_MAX
    )
    .fetch_all(pool)
    .await?
    {
        people.push(person_play(pool, person.id, person.name).await?);
    }

    Ok(Snapshot {
        games: counts.games,
        copies: counts.copies,
        wishlist: counts.wishlist,
        playing,
        playing_count: counts.playing,
        last_added,
        pick,
        people,
    })
}

async fn person_play(
    pool: &SqlitePool,
    id: String,
    name: Option<String>,
) -> sqlx::Result<PersonPlay> {
    let counts = sqlx::query!(
        r#"SELECT
             COUNT(*) FILTER (WHERE s.counts_as = 'backlog') AS "backlog!: i64",
             COUNT(*) FILTER (WHERE s.counts_as = 'playing') AS "playing!: i64"
           FROM play_progress p JOIN play_statuses s ON s.key = p.play_status
           WHERE p.user_id = ?
             AND EXISTS (SELECT 1 FROM releases r JOIN copies c ON c.release_id = r.id
                          WHERE r.game_id = p.game_id AND c.status = 'owned')"#,
        id
    )
    .fetch_one(pool)
    .await?;

    let playing = sqlx::query_scalar!(
        r#"SELECT g.title FROM play_progress p
            JOIN play_statuses s ON s.key = p.play_status
            JOIN games g ON g.id = p.game_id
            WHERE p.user_id = ?1 AND s.counts_as = 'playing'
              AND EXISTS (SELECT 1 FROM releases r JOIN copies c ON c.release_id = r.id
                           WHERE r.game_id = g.id AND c.status = 'owned')
            ORDER BY (SELECT MAX(played_on) FROM play_sessions
                      WHERE user_id = ?1 AND game_id = g.id) DESC NULLS LAST,
                     g.sort_title
            LIMIT ?2"#,
        id,
        PERSON_PLAYING_LIMIT
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
            WHERE s.user_id = ?
            ORDER BY s.played_on DESC, s.id DESC
            LIMIT 1"#,
        id
    )
    .fetch_optional(pool)
    .await?
    .map(|row| LastPlayed {
        title: row.title,
        date: row.date,
        platform: row.platform,
    });

    Ok(PersonPlay {
        id,
        name,
        backlog: counts.backlog,
        playing,
        playing_count: counts.playing,
        last_played,
    })
}
