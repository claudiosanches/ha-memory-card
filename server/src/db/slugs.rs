//! Slugs as stored: the tables that have them, and free ones for new rows.

use std::collections::{HashMap, HashSet};

use sqlx::SqliteConnection;

use super::sql::Sql;
use crate::error::{ApiError, ApiResult};
use crate::slug::{first_free, numbered_prefix, slugify, unique};

/// Tables with a unique `slug` column.
#[derive(Debug, Clone, Copy)]
pub enum Table {
    Platforms,
    Devices,
    Stores,
    Genres,
    GameModes,
    Labels,
}

impl Table {
    pub fn name(self) -> &'static str {
        match self {
            Self::Platforms => "platforms",
            Self::Devices => "devices",
            Self::Stores => "stores",
            Self::Genres => "genres",
            Self::GameModes => "game_modes",
            Self::Labels => "labels",
        }
    }

    /// The slug for a name with no letters or digits, before a counter.
    fn fallback(self) -> &'static str {
        match self {
            Self::Platforms => "platform",
            Self::Devices => "device",
            Self::Stores => "store",
            Self::Genres => "genre",
            Self::GameModes => "game-mode",
            Self::Labels => "label",
        }
    }
}

/// Hands out free slugs for many new rows of one table, as an import adds
/// them, reading the table once. Names that share a slug get the next
/// counter without trying the ones already handed out.
pub struct Allocator {
    table: Table,
    taken: Option<HashSet<String>>,
    next: HashMap<String, u64>,
}

impl Allocator {
    pub fn new(table: Table) -> Self {
        Self {
            table,
            taken: None,
            next: HashMap::new(),
        }
    }

    pub async fn free(&mut self, conn: &mut SqliteConnection, text: &str) -> sqlx::Result<String> {
        let taken = match &mut self.taken {
            Some(taken) => taken,
            None => {
                let mut sql = Sql::new("SELECT slug FROM ");
                sql.push(self.table.name());
                let slugs: Vec<String> = sql.build_query_scalar().fetch_all(conn).await?;
                self.taken
                    .insert(slugs.iter().map(|s| s.to_ascii_lowercase()).collect())
            }
        };
        let base = base(self.table, text);
        let from = self.next.get(&base).copied().unwrap_or(2);
        let (slug, next) = first_free(&base, taken, from);
        taken.insert(slug.to_ascii_lowercase());
        self.next.insert(base, next);
        Ok(slug)
    }
}

fn base(table: Table, text: &str) -> String {
    Some(slugify(text))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| table.fallback().to_owned())
}

/// A slug for `text` that no row of `table` has yet.
pub async fn free(conn: &mut SqliteConnection, table: Table, text: &str) -> sqlx::Result<String> {
    let base = base(table, text);
    // A numbered slug shortens a long base to fit, so look past the full base.
    let mut sql = Sql::new("SELECT slug FROM ");
    sql.push(table.name())
        .push(" WHERE slug LIKE ")
        .push_bind(numbered_prefix(&base).to_owned())
        .push(" || '%'");
    let taken: Vec<String> = sql.build_query_scalar().fetch_all(conn).await?;
    Ok(unique(&base, &taken))
}

/// The slug a new row gets: the one asked for, else a free one from its name.
pub async fn for_new(
    conn: &mut SqliteConnection,
    table: Table,
    requested: Option<&str>,
    name: &str,
) -> sqlx::Result<String> {
    match requested {
        Some(slug) => Ok(slug.to_owned()),
        None => free(conn, table, name).await,
    }
}

/// `name_taken` when a row of `table` other than `id` has `name`. Checked
/// before writing because a form fills the slug from the name, and the
/// taken name is the error that helps.
pub async fn ensure_name_free(
    conn: &mut SqliteConnection,
    table: Table,
    name: &str,
    id: Option<i64>,
) -> ApiResult<()> {
    let mut sql = Sql::new("SELECT EXISTS (SELECT 1 FROM ");
    sql.push(table.name())
        .push(" WHERE name = ")
        .push_bind(name.to_owned())
        .push(" AND id IS NOT ")
        .push_bind(id)
        .push(")");
    let taken: bool = sql.build_query_scalar().fetch_one(conn).await?;
    if taken {
        Err(ApiError::Conflict("name_taken"))
    } else {
        Ok(())
    }
}

/// Maps a unique violation on `slug` to `slug_taken` and any other to `name_taken`.
pub fn conflict(err: sqlx::Error) -> ApiError {
    let on_slug = err
        .as_database_error()
        .is_some_and(|db| db.message().ends_with(".slug"));
    ApiError::unique(err, if on_slug { "slug_taken" } else { "name_taken" })
}
