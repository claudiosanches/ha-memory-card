//! IGDB candidates for games without a link, scored by how close the names are.

use serde::Serialize;
use sqlx::SqlitePool;
use utoipa::ToSchema;

use super::{Igdb, IgdbError, SearchHit};
use crate::domain::search_key;

/// Candidates kept per game.
const CANDIDATES: usize = 3;

#[derive(Debug, Serialize, ToSchema)]
pub struct MatchGroup {
    pub game_id: i64,
    pub title: String,
    /// Platforms of the game's copies, in platform list order.
    pub platforms: Vec<String>,
    /// Best first.
    pub candidates: Vec<MatchCandidate>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MatchCandidate {
    #[serde(flatten)]
    pub hit: SearchHit,
    /// How close the names are, 0 to 100.
    pub score: u8,
}

#[derive(Debug, thiserror::Error)]
pub enum MatchError {
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error(transparent)]
    Igdb(#[from] IgdbError),
}

/// Unlinked, unskipped games after `after`, by id, with their candidates.
pub async fn next(
    pool: &SqlitePool,
    igdb: &Igdb,
    after: i64,
    limit: i64,
) -> Result<Vec<MatchGroup>, MatchError> {
    let games = sqlx::query!(
        r#"SELECT id AS "id!", title FROM games
           WHERE igdb_id IS NULL AND igdb_skipped = 0 AND id > ?
           ORDER BY id LIMIT ?"#,
        after,
        limit
    )
    .fetch_all(pool)
    .await?;
    let linked: Vec<(i64, i64)> =
        sqlx::query_as("SELECT igdb_id, id FROM games WHERE igdb_id IS NOT NULL")
            .fetch_all(pool)
            .await?;

    let mut groups = Vec::with_capacity(games.len());
    for game in games {
        let platforms = sqlx::query_scalar!(
            r#"SELECT p.name FROM platforms p
               WHERE EXISTS (SELECT 1 FROM releases r WHERE r.platform_id = p.id AND r.game_id = ?)
               ORDER BY p.sort_order, p.name"#,
            game.id
        )
        .fetch_all(pool)
        .await?;
        let mut candidates: Vec<MatchCandidate> = igdb
            .search(&game.title)
            .await?
            .into_iter()
            .map(|mut hit| {
                hit.game_id = linked
                    .iter()
                    .find(|(igdb_id, _)| *igdb_id == hit.igdb_id)
                    .map(|(_, id)| *id);
                MatchCandidate {
                    score: similarity(&game.title, &hit.name),
                    hit,
                }
            })
            .collect();
        // Stable, so equal scores keep IGDB's popularity order.
        candidates.sort_by_key(|c| std::cmp::Reverse(c.score));
        candidates.truncate(CANDIDATES);
        groups.push(MatchGroup {
            game_id: game.id,
            title: game.title,
            platforms,
            candidates,
        });
    }
    Ok(groups)
}

/// Dice similarity of the names' letter pairs, after folding case and accents.
pub fn similarity(a: &str, b: &str) -> u8 {
    let pairs = |text: &str| -> Vec<(char, char)> {
        let chars: Vec<char> = search_key(text)
            .chars()
            .filter(|c| c.is_alphanumeric())
            .collect();
        chars.windows(2).map(|w| (w[0], w[1])).collect()
    };
    let (a, b) = (pairs(a), pairs(b));
    if a.is_empty() || b.is_empty() {
        return 0;
    }
    let mut rest = b.clone();
    let mut shared = 0usize;
    for pair in &a {
        if let Some(at) = rest.iter().position(|p| p == pair) {
            rest.swap_remove(at);
            shared += 1;
        }
    }
    u8::try_from(shared * 200 / (a.len() + b.len())).unwrap_or(100)
}

#[cfg(test)]
mod tests {
    use super::similarity;

    #[test]
    fn close_names_score_high() {
        assert_eq!(similarity("Pokémon X", "Pokemon X"), 100);
        assert!(
            similarity(
                "Zelda Breath of the Wild",
                "The Legend of Zelda: Breath of the Wild"
            ) > 70
        );
        assert!(similarity("Super Mario Odyssey", "Super Mario Odyssey 64") > 85);
        assert!(similarity("Hades", "Halo") < 50);
        assert_eq!(similarity("", "Halo"), 0);
    }
}
