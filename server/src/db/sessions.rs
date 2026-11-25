//! Each person's play sessions: only they log, change or delete them.

use serde::Serialize;
use sqlx::{SqliteConnection, SqlitePool};
use utoipa::ToSchema;

use super::users::{self, Person};
use crate::config::FinishedScan;
use crate::error::{ApiError, ApiResult};

/// Most sessions one person keeps for a game; years of daily play stay below it.
pub const PER_GAME_MAX: i64 = 5000;
/// Most sessions one person logs in a day.
pub const PER_DAY_MAX: i64 = 200;

#[derive(Debug, Serialize, ToSchema)]
pub struct PlaySession {
    pub id: i64,
    /// `YYYY-MM-DD`.
    pub played_on: String,
    /// The owned copy played, when known.
    pub copy_id: Option<i64>,
    pub minutes: Option<i64>,
    pub notes: Option<String>,
    pub created_at: String,
}

/// Someone else's play of a game, as the game page shows it.
#[derive(Debug, Serialize, ToSchema)]
pub struct PersonPlay {
    pub person: Person,
    /// Their play status key.
    pub play_status: Option<String>,
    pub sessions: i64,
    pub minutes: i64,
    /// `YYYY-MM-DD`.
    pub last_played_on: Option<String>,
}

pub struct SessionFields {
    pub played_on: String,
    pub copy_id: Option<i64>,
    pub minutes: Option<i64>,
    pub notes: Option<String>,
}

/// `user`'s sessions of a game, latest first.
pub async fn for_person(
    pool: &SqlitePool,
    game_id: i64,
    user: &str,
) -> sqlx::Result<Vec<PlaySession>> {
    sqlx::query_as!(
        PlaySession,
        r#"SELECT id AS "id!", played_on, copy_id, minutes, notes, created_at
           FROM play_sessions
           WHERE user_id = ? AND game_id = ?
           ORDER BY played_on DESC, id DESC"#,
        user,
        game_id
    )
    .fetch_all(pool)
    .await
}

/// Everyone but `viewer` who keeps play for the game, the latest player first.
pub async fn household(
    pool: &SqlitePool,
    game_id: i64,
    viewer: &str,
) -> sqlx::Result<Vec<PersonPlay>> {
    let rows = sqlx::query!(
        r#"SELECT u.id AS "id!", COALESCE(u.display_name, u.name) AS name,
                  (SELECT play_status FROM play_progress
                   WHERE user_id = u.id AND game_id = ?1) AS play_status,
                  (SELECT COUNT(*) FROM play_sessions
                   WHERE user_id = u.id AND game_id = ?1) AS "sessions!: i64",
                  (SELECT COALESCE(SUM(minutes), 0) FROM play_sessions
                   WHERE user_id = u.id AND game_id = ?1) AS "minutes!: i64",
                  (SELECT MAX(played_on) FROM play_sessions
                   WHERE user_id = u.id AND game_id = ?1) AS "last_played_on: String"
           FROM users u
           WHERE u.id != ?2
             AND (EXISTS (SELECT 1 FROM play_progress WHERE user_id = u.id AND game_id = ?1)
                  OR EXISTS (SELECT 1 FROM play_sessions WHERE user_id = u.id AND game_id = ?1))
           ORDER BY 6 DESC NULLS LAST, 2"#,
        game_id,
        viewer
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| PersonPlay {
            person: Person {
                id: row.id,
                name: row.name,
            },
            play_status: row.play_status,
            sessions: row.sessions,
            minutes: row.minutes,
            last_played_on: row.last_played_on,
        })
        .collect())
}

/// Logs a session in `by`'s play. A game in their backlog, or with no status,
/// becomes playing; paused and finished games keep their status.
pub async fn add(
    pool: &SqlitePool,
    game_id: i64,
    fields: &SessionFields,
    by: &str,
) -> ApiResult<()> {
    let mut tx = super::write_tx(pool).await?;
    let exists: bool = sqlx::query_scalar!(
        r#"SELECT EXISTS (SELECT 1 FROM games WHERE id = ?) AS "exists!: bool""#,
        game_id
    )
    .fetch_one(&mut *tx)
    .await?;
    if !exists {
        return Err(ApiError::NotFound);
    }
    ensure_game_copy(&mut tx, game_id, fields.copy_id).await?;
    users::ensure(&mut tx, by).await?;
    room_for_one_more(&mut tx, game_id, by).await?;
    sqlx::query!(
        "INSERT INTO play_sessions (user_id, game_id, copy_id, played_on, minutes, notes)
         VALUES (?, ?, ?, ?, ?, ?)",
        by,
        game_id,
        fields.copy_id,
        fields.played_on,
        fields.minutes,
        fields.notes
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!(
        "INSERT INTO play_progress (user_id, game_id, play_status) VALUES (?, ?, 'playing')
         ON CONFLICT (user_id, game_id) DO UPDATE
         SET play_status = 'playing', updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE (SELECT counts_as FROM play_statuses WHERE key = play_progress.play_status)
               = 'backlog'",
        by,
        game_id
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(())
}

/// What a tag scan does for `person`: logs today's session for the copy, once
/// a day, and sets the game to playing for them unless their status already
/// counts as playing, or counts as finished and `finished` says to keep it.
/// Returns the game id. A wanted copy is left alone.
pub async fn start_from_tag(
    pool: &SqlitePool,
    copy_id: i64,
    today: &str,
    person: &str,
    finished: FinishedScan,
) -> ApiResult<i64> {
    let mut tx = super::write_tx(pool).await?;
    let copy = sqlx::query!(
        r#"SELECT r.game_id, c.status,
                  (SELECT s.counts_as FROM play_progress p
                   JOIN play_statuses s ON s.key = p.play_status
                   WHERE p.user_id = ?2 AND p.game_id = r.game_id) AS "counts_as: String"
           FROM copies c JOIN releases r ON r.id = c.release_id
           WHERE c.id = ?1"#,
        copy_id,
        person
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(ApiError::NotFound)?;
    if copy.status != "owned" {
        return Ok(copy.game_id);
    }
    // Known by ID only until they open Memory Card.
    users::ensure(&mut tx, person).await?;
    let logged: bool = sqlx::query_scalar!(
        r#"SELECT EXISTS (SELECT 1 FROM play_sessions
                          WHERE user_id = ?1 AND game_id = ?2 AND copy_id = ?3 AND played_on = ?4)
                  AS "logged!: bool""#,
        person,
        copy.game_id,
        copy_id,
        today
    )
    .fetch_one(&mut *tx)
    .await?;
    if !logged {
        room_for_one_more(&mut tx, copy.game_id, person).await?;
        sqlx::query!(
            "INSERT INTO play_sessions (user_id, game_id, copy_id, played_on) VALUES (?, ?, ?, ?)",
            person,
            copy.game_id,
            copy_id,
            today
        )
        .execute(&mut *tx)
        .await?;
    }
    let keep = match copy.counts_as.as_deref() {
        Some("playing") => true,
        Some("finished") => finished == FinishedScan::Keep,
        _ => false,
    };
    if !keep {
        sqlx::query!(
            "INSERT INTO play_progress (user_id, game_id, play_status) VALUES (?, ?, 'playing')
             ON CONFLICT (user_id, game_id) DO UPDATE
             SET play_status = 'playing', updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
             WHERE play_status IS NOT 'playing'",
            person,
            copy.game_id
        )
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(copy.game_id)
}

/// Refuses a session past `by`'s limit for the game or for the day.
async fn room_for_one_more(conn: &mut SqliteConnection, game_id: i64, by: &str) -> ApiResult<()> {
    let logged = sqlx::query!(
        r#"SELECT (SELECT COUNT(*) FROM play_sessions WHERE user_id = ?1 AND game_id = ?2)
                    AS "game!: i64",
                  (SELECT COUNT(*) FROM play_sessions WHERE user_id = ?1
                     AND created_at > strftime('%Y-%m-%dT%H:%M:%fZ', 'now', '-1 day'))
                    AS "today!: i64""#,
        by,
        game_id
    )
    .fetch_one(&mut *conn)
    .await?;
    if logged.game >= PER_GAME_MAX {
        return Err(ApiError::Conflict("too_many_sessions"));
    }
    if logged.today >= PER_DAY_MAX {
        return Err(ApiError::Conflict("too_many_sessions_today"));
    }
    Ok(())
}

/// Updates one of `owner`'s sessions and returns its game id.
pub async fn update(
    pool: &SqlitePool,
    id: i64,
    fields: &SessionFields,
    owner: &str,
) -> ApiResult<i64> {
    let mut tx = super::write_tx(pool).await?;
    let game_id = session_game(&mut tx, id, owner).await?;
    ensure_game_copy(&mut tx, game_id, fields.copy_id).await?;
    sqlx::query!(
        "UPDATE play_sessions SET copy_id = ?, played_on = ?, minutes = ?, notes = ? WHERE id = ?",
        fields.copy_id,
        fields.played_on,
        fields.minutes,
        fields.notes,
        id
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(game_id)
}

/// Deletes one of `owner`'s sessions and returns its game id.
pub async fn delete(pool: &SqlitePool, id: i64, owner: &str) -> ApiResult<i64> {
    let mut tx = super::write_tx(pool).await?;
    let game_id = session_game(&mut tx, id, owner).await?;
    sqlx::query!("DELETE FROM play_sessions WHERE id = ?", id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(game_id)
}

/// The session's game. `not_your_session` when `owner` did not log it.
async fn session_game(conn: &mut SqliteConnection, id: i64, owner: &str) -> ApiResult<i64> {
    let row = sqlx::query!(
        "SELECT game_id, user_id FROM play_sessions WHERE id = ?",
        id
    )
    .fetch_optional(conn)
    .await?
    .ok_or(ApiError::NotFound)?;
    if row.user_id == owner {
        Ok(row.game_id)
    } else {
        Err(ApiError::Forbidden("not_your_session"))
    }
}

/// The copy played must be an owned copy of the game.
async fn ensure_game_copy(
    conn: &mut SqliteConnection,
    game_id: i64,
    copy_id: Option<i64>,
) -> ApiResult<()> {
    let Some(copy_id) = copy_id else {
        return Ok(());
    };
    let belongs: bool = sqlx::query_scalar!(
        r#"SELECT EXISTS (SELECT 1 FROM copies c JOIN releases r ON r.id = c.release_id
                          WHERE c.id = ? AND r.game_id = ? AND c.status = 'owned') AS "belongs!: bool""#,
        copy_id,
        game_id
    )
    .fetch_one(conn)
    .await?;
    if belongs {
        Ok(())
    } else {
        Err(ApiError::invalid("invalid_copy", "copy_id"))
    }
}
