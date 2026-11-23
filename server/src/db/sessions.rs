use serde::Serialize;
use sqlx::{SqliteConnection, SqlitePool};
use utoipa::ToSchema;

use super::users::Person;
use crate::config::FinishedScan;
use crate::error::{ApiError, ApiResult};

/// Most sessions one game keeps; years of daily play stay below it.
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
    pub created_by: Option<Person>,
}

pub struct SessionFields {
    pub played_on: String,
    pub copy_id: Option<i64>,
    pub minutes: Option<i64>,
    pub notes: Option<String>,
}

/// A game's sessions, latest first.
pub async fn for_game(pool: &SqlitePool, game_id: i64) -> sqlx::Result<Vec<PlaySession>> {
    let rows = sqlx::query!(
        r#"SELECT s.id AS "id!", s.played_on, s.copy_id, s.minutes, s.notes, s.created_at,
                  s.created_by, COALESCE(u.display_name, u.name) AS created_by_name
           FROM play_sessions s LEFT JOIN users u ON u.id = s.created_by
           WHERE s.game_id = ?
           ORDER BY s.played_on DESC, s.id DESC"#,
        game_id
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| PlaySession {
            id: row.id,
            played_on: row.played_on,
            copy_id: row.copy_id,
            minutes: row.minutes,
            notes: row.notes,
            created_at: row.created_at,
            created_by: Person::from_columns(row.created_by, row.created_by_name),
        })
        .collect())
}

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
    room_for_one_more(&mut tx, game_id, Some(by)).await?;
    sqlx::query!(
        "INSERT INTO play_sessions (game_id, copy_id, played_on, minutes, notes, created_by)
         VALUES (?, ?, ?, ?, ?, ?)",
        game_id,
        fields.copy_id,
        fields.played_on,
        fields.minutes,
        fields.notes,
        by
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(())
}

/// What a tag scan does: logs today's session for the copy, once a day, and
/// sets the game to playing unless its status already counts as playing, or
/// counts as finished and `finished` says to keep it.
/// Returns the game id. A wanted copy is left alone.
pub async fn start_from_tag(
    pool: &SqlitePool,
    copy_id: i64,
    today: &str,
    user: Option<&str>,
    finished: FinishedScan,
) -> ApiResult<i64> {
    let mut tx = super::write_tx(pool).await?;
    let copy = sqlx::query!(
        r#"SELECT r.game_id, c.status, g.play_status,
                  (SELECT counts_as FROM play_statuses WHERE key = g.play_status) AS counts_as
           FROM copies c JOIN releases r ON r.id = c.release_id JOIN games g ON g.id = r.game_id
           WHERE c.id = ?"#,
        copy_id
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(ApiError::NotFound)?;
    if copy.status != "owned" {
        return Ok(copy.game_id);
    }
    if let Some(user) = user {
        // Known by ID only until they open Memory Card.
        sqlx::query!("INSERT OR IGNORE INTO users (id) VALUES (?)", user)
            .execute(&mut *tx)
            .await?;
    }
    let logged: bool = sqlx::query_scalar!(
        r#"SELECT EXISTS (SELECT 1 FROM play_sessions
                          WHERE game_id = ?1 AND copy_id = ?2 AND played_on = ?3) AS "logged!: bool""#,
        copy.game_id,
        copy_id,
        today
    )
    .fetch_one(&mut *tx)
    .await?;
    if !logged {
        room_for_one_more(&mut tx, copy.game_id, user).await?;
        sqlx::query!(
            "INSERT INTO play_sessions (game_id, copy_id, played_on, created_by) VALUES (?, ?, ?, ?)",
            copy.game_id,
            copy_id,
            today,
            user
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
            "UPDATE games SET play_status = 'playing', updated_by = COALESCE(?, updated_by),
                 updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
             WHERE id = ?",
            user,
            copy.game_id
        )
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(copy.game_id)
}

/// Refuses a session past the game's limit, or past the day's limit of the
/// person logging it. Scans with no person share one day's limit.
async fn room_for_one_more(
    conn: &mut SqliteConnection,
    game_id: i64,
    by: Option<&str>,
) -> ApiResult<()> {
    let logged = sqlx::query!(
        r#"SELECT (SELECT COUNT(*) FROM play_sessions WHERE game_id = ?1) AS "game!: i64",
                  (SELECT COUNT(*) FROM play_sessions WHERE created_by IS ?2
                     AND created_at > strftime('%Y-%m-%dT%H:%M:%fZ', 'now', '-1 day'))
                    AS "today!: i64""#,
        game_id,
        by
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

/// Updates a session and returns its game id.
/// `owner` limits the change to sessions that user logged; `None` allows any.
pub async fn update(
    pool: &SqlitePool,
    id: i64,
    fields: &SessionFields,
    owner: Option<&str>,
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

/// Deletes a session and returns its game id. `owner` as for [`update`].
pub async fn delete(pool: &SqlitePool, id: i64, owner: Option<&str>) -> ApiResult<i64> {
    let mut tx = super::write_tx(pool).await?;
    let game_id = session_game(&mut tx, id, owner).await?;
    sqlx::query!("DELETE FROM play_sessions WHERE id = ?", id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(game_id)
}

/// The session's game. `not_your_session` when `owner` did not log it.
async fn session_game(conn: &mut SqliteConnection, id: i64, owner: Option<&str>) -> ApiResult<i64> {
    let row = sqlx::query!(
        "SELECT game_id, created_by FROM play_sessions WHERE id = ?",
        id
    )
    .fetch_optional(conn)
    .await?
    .ok_or(ApiError::NotFound)?;
    match owner {
        Some(user) if row.created_by.as_deref() != Some(user) => {
            Err(ApiError::Forbidden("not_your_session"))
        }
        _ => Ok(row.game_id),
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
