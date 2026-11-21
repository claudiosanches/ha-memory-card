use serde::Serialize;
use sqlx::SqlitePool;
use utoipa::ToSchema;

use crate::error::ApiResult;

/// A detail a game can lack. Copy details count when any owned copy lacks them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum MissingField {
    PlayStatus,
    Cover,
    GameModes,
    /// No player counts on the game or its releases. A game whose only mode
    /// is single player has its answer.
    Players,
    /// No owned copies.
    Copies,
    Device,
    Store,
    /// A physical copy without a condition.
    Condition,
    /// Not linked to IGDB. Only reported when IGDB is set up.
    Igdb,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MissingDetails {
    pub game_id: i64,
    pub title: String,
    pub fields: Vec<MissingField>,
}

/// Games lacking at least one detail, in title order.
pub async fn list(pool: &SqlitePool, igdb: bool) -> ApiResult<Vec<MissingDetails>> {
    let rows = sqlx::query!(
        r#"WITH owned AS (
             SELECT r.game_id, c.device_id, c.store_id, c.format, c.condition,
                    c.bundle_id IS NOT NULL AS in_bundle
             FROM copies c JOIN releases r ON r.id = c.release_id
             WHERE c.status = 'owned'
           )
           SELECT g.id AS "id!", g.title,
             g.play_status IS NULL AS "play_status!: bool",
             g.cover_path IS NULL AS "cover!: bool",
             NOT EXISTS (SELECT 1 FROM game_game_modes gm WHERE gm.game_id = g.id) AS "game_modes!: bool",
             (g.local_max_players IS NULL AND g.online_max_players IS NULL
              AND g.local_coop IS NULL AND g.online_coop IS NULL
              AND NOT EXISTS (
                SELECT 1 FROM releases r WHERE r.game_id = g.id
                  AND (r.local_max_players IS NOT NULL OR r.online_max_players IS NOT NULL
                       OR r.local_coop IS NOT NULL OR r.online_coop IS NOT NULL))
              AND NOT (
                EXISTS (SELECT 1 FROM game_game_modes gm WHERE gm.game_id = g.id)
                AND NOT EXISTS (
                  SELECT 1 FROM game_game_modes gm JOIN game_modes m ON m.id = gm.game_mode_id
                  WHERE gm.game_id = g.id AND m.slug != 'single-player'))
             ) AS "players!: bool",
             NOT EXISTS (SELECT 1 FROM owned o WHERE o.game_id = g.id) AS "copies!: bool",
             -- A bundle's contents take these from the bundle copy.
             EXISTS (SELECT 1 FROM owned o WHERE o.game_id = g.id AND NOT o.in_bundle
                       AND o.device_id IS NULL) AS "device!: bool",
             EXISTS (SELECT 1 FROM owned o WHERE o.game_id = g.id AND NOT o.in_bundle
                       AND o.store_id IS NULL) AS "store!: bool",
             EXISTS (SELECT 1 FROM owned o WHERE o.game_id = g.id AND NOT o.in_bundle
                       AND o.format = 'physical' AND o.condition IS NULL) AS "condition!: bool",
             g.igdb_id IS NULL AS "igdb!: bool"
           FROM games g
           -- Games only on the wishlist are not in the collection yet.
           WHERE EXISTS (SELECT 1 FROM owned o WHERE o.game_id = g.id)
              OR NOT EXISTS (SELECT 1 FROM releases r JOIN copies c ON c.release_id = r.id
                             WHERE r.game_id = g.id)
           ORDER BY g.sort_title, g.id"#
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .filter_map(|row| {
            let fields: Vec<MissingField> = [
                (row.play_status, MissingField::PlayStatus),
                (row.cover, MissingField::Cover),
                (row.game_modes, MissingField::GameModes),
                (row.players, MissingField::Players),
                (row.copies, MissingField::Copies),
                (row.device, MissingField::Device),
                (row.store, MissingField::Store),
                (row.condition, MissingField::Condition),
                (row.igdb && igdb, MissingField::Igdb),
            ]
            .into_iter()
            .filter_map(|(missing, field)| missing.then_some(field))
            .collect();
            (!fields.is_empty()).then_some(MissingDetails {
                game_id: row.id,
                title: row.title,
                fields,
            })
        })
        .collect())
}
