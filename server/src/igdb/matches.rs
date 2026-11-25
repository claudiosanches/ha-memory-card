//! IGDB candidates for games without a link, scored by how close the names are.

use std::collections::HashMap;

use serde::Serialize;
use utoipa::ToSchema;

use super::{Igdb, IgdbError, SearchHit};
use crate::db::Pool;
use crate::db::igdb as db;
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
    pool: &Pool,
    igdb: &Igdb,
    after: i64,
    limit: i64,
) -> Result<Vec<MatchGroup>, MatchError> {
    let games = db::to_review(pool, after, limit).await?;

    let mut groups = Vec::with_capacity(games.len());
    for (game_id, title) in games {
        let platforms = db::platform_names(pool, game_id).await?;
        let hits = igdb.search(&title).await?;
        let ids: Vec<i64> = hits.iter().map(|hit| hit.igdb_id).collect();
        let linked: HashMap<i64, i64> = db::games_for(pool, &ids).await?.into_iter().collect();
        let mut candidates: Vec<MatchCandidate> = hits
            .into_iter()
            .map(|mut hit| {
                hit.game_id = linked.get(&hit.igdb_id).copied();
                MatchCandidate {
                    score: similarity(&title, &hit.name),
                    hit,
                }
            })
            .collect();
        // Stable, so equal scores keep IGDB's popularity order.
        candidates.sort_by_key(|c| std::cmp::Reverse(c.score));
        candidates.truncate(CANDIDATES);
        groups.push(MatchGroup {
            game_id,
            title,
            platforms,
            candidates,
        });
    }
    Ok(groups)
}

/// Dice similarity of the names' letter pairs, after folding case and accents.
pub fn similarity(a: &str, b: &str) -> u8 {
    let pairs = |text: &str| -> HashMap<(char, char), usize> {
        let chars: Vec<char> = search_key(text)
            .chars()
            .filter(|c| c.is_alphanumeric())
            .collect();
        let mut counts = HashMap::new();
        for w in chars.windows(2) {
            *counts.entry((w[0], w[1])).or_default() += 1;
        }
        counts
    };
    let (a, b) = (pairs(a), pairs(b));
    if a.is_empty() || b.is_empty() {
        return 0;
    }
    let shared: usize = a
        .iter()
        .map(|(pair, n)| (*n).min(b.get(pair).copied().unwrap_or(0)))
        .sum();
    let total: usize = a.values().chain(b.values()).sum();
    u8::try_from(shared * 200 / total).unwrap_or(100)
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
