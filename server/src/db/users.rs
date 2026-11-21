use serde::Serialize;
use sqlx::SqlitePool;
use utoipa::ToSchema;

use crate::access::User;

/// Who added or last changed something.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct Person {
    /// Home Assistant user ID.
    pub id: String,
    /// Display name, else username; `None` when Home Assistant sent neither.
    pub name: Option<String>,
}

impl Person {
    pub fn from_columns(id: Option<String>, name: Option<String>) -> Option<Self> {
        id.map(|id| Self { id, name })
    }
}

/// Stores the user's current names. Writes only when they changed.
pub async fn record(pool: &SqlitePool, user: &User) -> sqlx::Result<()> {
    sqlx::query!(
        "INSERT INTO users (id, name, display_name) VALUES (?, ?, ?)
         ON CONFLICT (id) DO UPDATE SET name = excluded.name, display_name = excluded.display_name
         WHERE name IS NOT excluded.name OR display_name IS NOT excluded.display_name",
        user.id,
        user.name,
        user.display_name
    )
    .execute(pool)
    .await?;
    Ok(())
}
