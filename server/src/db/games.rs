use serde::Serialize;
use sqlx::types::Json;
use sqlx::{SqliteConnection, SqlitePool};
use utoipa::ToSchema;

use super::found;
use crate::domain::{Condition, CopyStatus, Format, PlayStatus, Region, sort_title};
use crate::error::{ApiError, ApiResult};

#[derive(Debug, Serialize, ToSchema)]
pub struct CopyDetail {
    pub id: i64,
    pub status: CopyStatus,
    pub format: Format,
    pub device_id: Option<i64>,
    pub store_id: Option<i64>,
    pub condition: Option<Condition>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ReleaseDetail {
    pub id: i64,
    pub platform_id: i64,
    pub region: Region,
    pub title: Option<String>,
    pub product_code: Option<String>,
    pub release_date: Option<String>,
    pub local_max_players: Option<i64>,
    pub online_max_players: Option<i64>,
    pub local_coop: Option<bool>,
    pub online_coop: Option<bool>,
    pub copies: Vec<CopyDetail>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct GameDetail {
    pub id: i64,
    pub igdb_id: Option<i64>,
    pub title: String,
    pub summary: Option<String>,
    /// Relative to the covers URL.
    pub cover_path: Option<String>,
    pub first_release_date: Option<String>,
    pub developers: Vec<String>,
    pub publishers: Vec<String>,
    pub play_status: Option<PlayStatus>,
    pub last_played_at: Option<String>,
    pub last_played_copy_id: Option<i64>,
    pub local_max_players: Option<i64>,
    pub online_max_players: Option<i64>,
    pub local_coop: Option<bool>,
    pub online_coop: Option<bool>,
    pub game_mode_ids: Vec<i64>,
    pub genre_ids: Vec<i64>,
    /// In platform list order, then region.
    pub releases: Vec<ReleaseDetail>,
}

/// Player counts as stored on games and releases. `None` means unknown.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PlayerCounts {
    pub local_max_players: Option<i64>,
    pub online_max_players: Option<i64>,
    pub local_coop: Option<bool>,
    pub online_coop: Option<bool>,
}

pub struct GameFields {
    pub title: String,
    pub summary: Option<String>,
    pub play_status: Option<PlayStatus>,
    pub last_played_at: Option<String>,
    pub last_played_copy_id: Option<i64>,
    pub players: PlayerCounts,
    pub game_mode_ids: Vec<i64>,
}

pub struct CopyFields {
    pub platform_id: i64,
    pub region: Region,
    pub product_code: Option<String>,
    pub format: Format,
    pub device_id: Option<i64>,
    pub store_id: Option<i64>,
    pub condition: Option<Condition>,
    pub notes: Option<String>,
}

pub struct ReleaseFields {
    pub title: Option<String>,
    pub product_code: Option<String>,
    pub release_date: Option<String>,
    pub players: PlayerCounts,
}

pub async fn get(pool: &SqlitePool, id: i64) -> ApiResult<GameDetail> {
    let game = sqlx::query!(
        r#"SELECT g.id AS "id!", g.igdb_id, g.title, g.summary, g.cover_path, g.first_release_date,
                  g.developers AS "developers: Json<Vec<String>>",
                  g.publishers AS "publishers: Json<Vec<String>>",
                  g.play_status AS "play_status: PlayStatus", g.last_played_at, g.last_played_copy_id,
                  g.local_max_players, g.online_max_players,
                  g.local_coop AS "local_coop: bool", g.online_coop AS "online_coop: bool",
                  (SELECT json_group_array(game_mode_id) FROM game_game_modes WHERE game_id = g.id)
                      AS "game_mode_ids!: Json<Vec<i64>>",
                  (SELECT json_group_array(genre_id) FROM game_genres WHERE game_id = g.id)
                      AS "genre_ids!: Json<Vec<i64>>"
           FROM games g WHERE g.id = ?"#,
        id
    )
    .fetch_optional(pool)
    .await?
    .ok_or(ApiError::NotFound)?;

    let releases = sqlx::query!(
        r#"SELECT r.id AS "id!", r.platform_id, r.region AS "region: Region", r.title, r.product_code,
                  r.release_date, r.local_max_players, r.online_max_players,
                  r.local_coop AS "local_coop: bool", r.online_coop AS "online_coop: bool"
           FROM releases r JOIN platforms p ON p.id = r.platform_id
           WHERE r.game_id = ?
           ORDER BY p.sort_order, r.region, r.id"#,
        id
    )
    .fetch_all(pool)
    .await?;

    let mut copies = sqlx::query!(
        r#"SELECT c.id AS "id!", c.release_id, c.status AS "status: CopyStatus",
                  c.format AS "format: Format", c.device_id, c.store_id,
                  c.condition AS "condition: Condition", c.notes
           FROM copies c JOIN releases r ON r.id = c.release_id
           WHERE r.game_id = ?
           ORDER BY c.id"#,
        id
    )
    .fetch_all(pool)
    .await?;

    let releases = releases
        .into_iter()
        .map(|r| {
            let (mine, rest): (Vec<_>, Vec<_>) =
                copies.drain(..).partition(|c| c.release_id == r.id);
            copies = rest;
            ReleaseDetail {
                id: r.id,
                platform_id: r.platform_id,
                region: r.region,
                title: r.title,
                product_code: r.product_code,
                release_date: r.release_date,
                local_max_players: r.local_max_players,
                online_max_players: r.online_max_players,
                local_coop: r.local_coop,
                online_coop: r.online_coop,
                copies: mine
                    .into_iter()
                    .map(|c| CopyDetail {
                        id: c.id,
                        status: c.status,
                        format: c.format,
                        device_id: c.device_id,
                        store_id: c.store_id,
                        condition: c.condition,
                        notes: c.notes,
                    })
                    .collect(),
            }
        })
        .collect();

    let mut game_mode_ids = game.game_mode_ids.0;
    game_mode_ids.sort_unstable();
    let mut genre_ids = game.genre_ids.0;
    genre_ids.sort_unstable();

    Ok(GameDetail {
        id: game.id,
        igdb_id: game.igdb_id,
        title: game.title,
        summary: game.summary,
        cover_path: game.cover_path,
        first_release_date: game.first_release_date,
        developers: game.developers.0,
        publishers: game.publishers.0,
        play_status: game.play_status,
        last_played_at: game.last_played_at,
        last_played_copy_id: game.last_played_copy_id,
        local_max_players: game.local_max_players,
        online_max_players: game.online_max_players,
        local_coop: game.local_coop,
        online_coop: game.online_coop,
        game_mode_ids,
        genre_ids,
        releases,
    })
}

pub async fn create(
    pool: &SqlitePool,
    title: &str,
    play_status: Option<PlayStatus>,
) -> ApiResult<i64> {
    let sort = sort_title(title);
    Ok(sqlx::query_scalar!(
        r#"INSERT INTO games (title, sort_title, play_status) VALUES (?, ?, ?) RETURNING id AS "id!""#,
        title,
        sort,
        play_status
    )
    .fetch_one(pool)
    .await?)
}

pub async fn update(pool: &SqlitePool, id: i64, fields: &GameFields) -> ApiResult<()> {
    let mut tx = crate::db::write_tx(pool).await?;

    if let Some(copy_id) = fields.last_played_copy_id {
        let belongs: bool = sqlx::query_scalar!(
            r#"SELECT EXISTS (SELECT 1 FROM copies c JOIN releases r ON r.id = c.release_id
                              WHERE c.id = ? AND r.game_id = ?) AS "belongs!: bool""#,
            copy_id,
            id
        )
        .fetch_one(&mut *tx)
        .await?;
        if !belongs {
            return Err(ApiError::invalid("invalid_copy", "last_played_copy_id"));
        }
    }
    let modes = id_list(
        &mut tx,
        "game_modes",
        &fields.game_mode_ids,
        "game_mode_ids",
    )
    .await?;

    let sort = sort_title(&fields.title);
    let p = &fields.players;
    let result = sqlx::query!(
        "UPDATE games
         SET title = ?, sort_title = ?, summary = ?, play_status = ?, last_played_at = ?,
             last_played_copy_id = ?, local_max_players = ?, online_max_players = ?,
             local_coop = ?, online_coop = ?,
             updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE id = ?",
        fields.title,
        sort,
        fields.summary,
        fields.play_status,
        fields.last_played_at,
        fields.last_played_copy_id,
        p.local_max_players,
        p.online_max_players,
        p.local_coop,
        p.online_coop,
        id
    )
    .execute(&mut *tx)
    .await?;
    found(result.rows_affected())?;

    sqlx::query!("DELETE FROM game_game_modes WHERE game_id = ?", id)
        .execute(&mut *tx)
        .await?;
    sqlx::query!(
        "INSERT INTO game_game_modes (game_id, game_mode_id) SELECT ?, value FROM json_each(?)",
        id,
        modes
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}

pub async fn delete(pool: &SqlitePool, id: i64) -> ApiResult<()> {
    let result = sqlx::query!("DELETE FROM games WHERE id = ?", id)
        .execute(pool)
        .await?;
    found(result.rows_affected())
}

/// Adds a copy, creating its release when the game has none for that
/// platform, region and product code.
pub async fn add_copy(pool: &SqlitePool, game_id: i64, fields: &CopyFields) -> ApiResult<()> {
    let mut tx = crate::db::write_tx(pool).await?;
    let game_exists: bool = sqlx::query_scalar!(
        r#"SELECT EXISTS (SELECT 1 FROM games WHERE id = ?) AS "exists!: bool""#,
        game_id
    )
    .fetch_one(&mut *tx)
    .await?;
    if !game_exists {
        return Err(ApiError::NotFound);
    }
    check_references(&mut tx, fields).await?;
    let release_id = release_for(&mut tx, game_id, fields).await?;
    sqlx::query!(
        "INSERT INTO copies (release_id, format, device_id, store_id, condition, notes)
         VALUES (?, ?, ?, ?, ?, ?)",
        release_id,
        fields.format,
        fields.device_id,
        fields.store_id,
        fields.condition,
        fields.notes
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(())
}

/// Updates a copy and returns its game id. Changing platform, region or
/// product code moves it to the matching release, creating it if needed.
pub async fn update_copy(pool: &SqlitePool, copy_id: i64, fields: &CopyFields) -> ApiResult<i64> {
    let mut tx = crate::db::write_tx(pool).await?;
    let game_id = copy_game(&mut tx, copy_id).await?;
    check_references(&mut tx, fields).await?;
    let release_id = release_for(&mut tx, game_id, fields).await?;
    sqlx::query!(
        "UPDATE copies
         SET release_id = ?, format = ?, device_id = ?, store_id = ?, condition = ?, notes = ?,
             updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE id = ?",
        release_id,
        fields.format,
        fields.device_id,
        fields.store_id,
        fields.condition,
        fields.notes,
        copy_id
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(game_id)
}

/// Deletes a copy and returns its game id. The release stays.
pub async fn delete_copy(pool: &SqlitePool, copy_id: i64) -> ApiResult<i64> {
    let mut tx = crate::db::write_tx(pool).await?;
    let game_id = copy_game(&mut tx, copy_id).await?;
    sqlx::query!("DELETE FROM copies WHERE id = ?", copy_id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(game_id)
}

/// Updates a release and returns its game id.
pub async fn update_release(
    pool: &SqlitePool,
    release_id: i64,
    fields: &ReleaseFields,
) -> ApiResult<i64> {
    let game_id = release_game(pool, release_id).await?;
    let p = &fields.players;
    sqlx::query!(
        "UPDATE releases
         SET title = ?, product_code = ?, release_date = ?, local_max_players = ?,
             online_max_players = ?, local_coop = ?, online_coop = ?,
             updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE id = ?",
        fields.title,
        fields.product_code,
        fields.release_date,
        p.local_max_players,
        p.online_max_players,
        p.local_coop,
        p.online_coop,
        release_id
    )
    .execute(pool)
    .await
    .map_err(|err| ApiError::unique(err, "release_exists"))?;
    Ok(game_id)
}

/// Deletes a release with its copies and returns its game id.
pub async fn delete_release(pool: &SqlitePool, release_id: i64) -> ApiResult<i64> {
    let game_id = release_game(pool, release_id).await?;
    sqlx::query!("DELETE FROM releases WHERE id = ?", release_id)
        .execute(pool)
        .await?;
    Ok(game_id)
}

async fn release_game(pool: &SqlitePool, release_id: i64) -> ApiResult<i64> {
    sqlx::query_scalar!("SELECT game_id FROM releases WHERE id = ?", release_id)
        .fetch_optional(pool)
        .await?
        .ok_or(ApiError::NotFound)
}

async fn copy_game(conn: &mut SqliteConnection, copy_id: i64) -> ApiResult<i64> {
    sqlx::query_scalar!(
        "SELECT r.game_id FROM copies c JOIN releases r ON r.id = c.release_id WHERE c.id = ?",
        copy_id
    )
    .fetch_optional(&mut *conn)
    .await?
    .ok_or(ApiError::NotFound)
}

async fn check_references(conn: &mut SqliteConnection, fields: &CopyFields) -> ApiResult<()> {
    let platform: bool = sqlx::query_scalar!(
        r#"SELECT EXISTS (SELECT 1 FROM platforms WHERE id = ?) AS "exists!: bool""#,
        fields.platform_id
    )
    .fetch_one(&mut *conn)
    .await?;
    if !platform {
        return Err(ApiError::invalid("unknown_platform", "platform_id"));
    }
    if let Some(device_id) = fields.device_id {
        let exists: bool = sqlx::query_scalar!(
            r#"SELECT EXISTS (SELECT 1 FROM devices WHERE id = ?) AS "exists!: bool""#,
            device_id
        )
        .fetch_one(&mut *conn)
        .await?;
        if !exists {
            return Err(ApiError::invalid("unknown_device", "device_id"));
        }
    }
    if let Some(store_id) = fields.store_id {
        let exists: bool = sqlx::query_scalar!(
            r#"SELECT EXISTS (SELECT 1 FROM stores WHERE id = ?) AS "exists!: bool""#,
            store_id
        )
        .fetch_one(&mut *conn)
        .await?;
        if !exists {
            return Err(ApiError::invalid("unknown_store", "store_id"));
        }
    }
    Ok(())
}

async fn release_for(
    conn: &mut SqliteConnection,
    game_id: i64,
    fields: &CopyFields,
) -> ApiResult<i64> {
    let existing = sqlx::query_scalar!(
        r#"SELECT id AS "id!" FROM releases
           WHERE game_id = ? AND platform_id = ? AND region = ?
             AND lower(COALESCE(product_code, '')) = lower(COALESCE(?, ''))"#,
        game_id,
        fields.platform_id,
        fields.region,
        fields.product_code
    )
    .fetch_optional(&mut *conn)
    .await?;
    if let Some(id) = existing {
        return Ok(id);
    }
    Ok(sqlx::query_scalar!(
        r#"INSERT INTO releases (game_id, platform_id, region, product_code)
           VALUES (?, ?, ?, ?) RETURNING id AS "id!""#,
        game_id,
        fields.platform_id,
        fields.region,
        fields.product_code
    )
    .fetch_one(&mut *conn)
    .await?)
}

/// Deduplicates ids, checks they exist in `table`, returns them as a JSON array.
async fn id_list(
    conn: &mut SqliteConnection,
    table: &'static str,
    ids: &[i64],
    field: &'static str,
) -> ApiResult<String> {
    let mut ids = ids.to_vec();
    ids.sort_unstable();
    ids.dedup();
    let json = serde_json::to_string(&ids).expect("integers serialize");
    let sql = format!("SELECT COUNT(*) FROM {table} WHERE id IN (SELECT value FROM json_each(?))");
    let known: i64 = sqlx::query_scalar(sqlx::AssertSqlSafe(sql))
        .bind(&json)
        .fetch_one(&mut *conn)
        .await?;
    if known != ids.len() as i64 {
        return Err(ApiError::invalid("unknown_id", field));
    }
    Ok(json)
}
