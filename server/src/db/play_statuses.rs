//! Play statuses: the six built-in keys plus the user's own. Built-ins can be
//! renamed and recolored but keep their key and group.

use serde::Serialize;
use sqlx::{SqliteExecutor, SqlitePool};
use utoipa::ToSchema;

use super::{found, write_tx};
use crate::domain::{BuiltinStatus, StatusColor, StatusGroup, loose};
use crate::error::{ApiError, ApiResult};

/// Reserved for "no play status" in the library filter.
pub const NONE: &str = "none";
const KEY_MAX: usize = 40;

#[derive(Debug, Serialize, ToSchema)]
pub struct PlayStatus {
    pub key: String,
    /// `null` for a built-in status that keeps its default name.
    pub label: Option<String>,
    pub color: StatusColor,
    pub counts_as: StatusGroup,
    pub builtin: bool,
    /// Games in this status, once for each person who has them in it.
    pub game_count: i64,
}

pub struct StatusFields {
    /// `None` resets a built-in to its default name; custom statuses need one.
    pub label: Option<String>,
    pub color: StatusColor,
    pub counts_as: StatusGroup,
}

/// A status's key and label, for matching names.
#[derive(Debug, Clone)]
pub struct StatusName {
    pub key: String,
    pub label: Option<String>,
}

pub async fn list(pool: &SqlitePool) -> sqlx::Result<Vec<PlayStatus>> {
    sqlx::query_as!(
        PlayStatus,
        r#"SELECT s.key AS "key!", s.label, s.color AS "color: StatusColor",
                  s.counts_as AS "counts_as: StatusGroup", s.builtin AS "builtin: bool",
                  (SELECT COUNT(*) FROM play_progress p WHERE p.play_status = s.key) AS "game_count!: i64"
           FROM play_statuses s
           ORDER BY s.sort_order, s.key"#
    )
    .fetch_all(pool)
    .await
}

pub async fn get(pool: &SqlitePool, key: &str) -> ApiResult<PlayStatus> {
    sqlx::query_as!(
        PlayStatus,
        r#"SELECT s.key AS "key!", s.label, s.color AS "color: StatusColor",
                  s.counts_as AS "counts_as: StatusGroup", s.builtin AS "builtin: bool",
                  (SELECT COUNT(*) FROM play_progress p WHERE p.play_status = s.key) AS "game_count!: i64"
           FROM play_statuses s
           WHERE s.key = ?"#,
        key
    )
    .fetch_optional(pool)
    .await?
    .ok_or(ApiError::NotFound)
}

pub async fn names<'e>(db: impl SqliteExecutor<'e>) -> sqlx::Result<Vec<StatusName>> {
    sqlx::query_as!(
        StatusName,
        r#"SELECT key AS "key!", label FROM play_statuses ORDER BY sort_order"#
    )
    .fetch_all(db)
    .await
}

/// Keys of the statuses that count as backlog.
pub async fn backlog_keys<'e>(db: impl SqliteExecutor<'e>) -> sqlx::Result<Vec<String>> {
    sqlx::query_scalar!(
        r#"SELECT key AS "key!" FROM play_statuses WHERE counts_as = 'backlog' ORDER BY sort_order"#
    )
    .fetch_all(db)
    .await
}

/// `unknown_play_status` unless `key` is absent or names a status.
pub async fn ensure_exists<'e>(db: impl SqliteExecutor<'e>, key: Option<&str>) -> ApiResult<()> {
    let Some(key) = key else { return Ok(()) };
    let exists = sqlx::query_scalar!(
        r#"SELECT EXISTS (SELECT 1 FROM play_statuses WHERE key = ?) AS "exists!: bool""#,
        key
    )
    .fetch_one(db)
    .await?;
    if exists {
        Ok(())
    } else {
        Err(ApiError::invalid("unknown_play_status", "play_status"))
    }
}

pub async fn create(pool: &SqlitePool, fields: &StatusFields) -> ApiResult<String> {
    let Some(label) = fields.label.as_deref() else {
        return Err(ApiError::invalid("required", "label"));
    };
    let mut tx = write_tx(pool).await?;
    let existing = names(&mut *tx).await?;
    if name_taken(&existing, None, label) {
        return Err(ApiError::Conflict("name_taken"));
    }
    let key = new_key(&existing, label);
    sqlx::query!(
        "INSERT INTO play_statuses (key, label, color, counts_as, sort_order)
         VALUES (?, ?, ?, ?, (SELECT COALESCE(MAX(sort_order), 0) + 10 FROM play_statuses))",
        key,
        label,
        fields.color,
        fields.counts_as
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(key)
}

pub async fn update(pool: &SqlitePool, key: &str, fields: &StatusFields) -> ApiResult<()> {
    let mut tx = write_tx(pool).await?;
    let current = sqlx::query!(
        r#"SELECT builtin AS "builtin: bool", counts_as AS "counts_as: StatusGroup"
           FROM play_statuses WHERE key = ?"#,
        key
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(ApiError::NotFound)?;

    if current.builtin && fields.counts_as != current.counts_as {
        return Err(ApiError::invalid("builtin_status", "counts_as"));
    }
    match fields.label.as_deref() {
        None if !current.builtin => return Err(ApiError::invalid("required", "label")),
        Some(label) if name_taken(&names(&mut *tx).await?, Some(key), label) => {
            return Err(ApiError::Conflict("name_taken"));
        }
        _ => {}
    }

    sqlx::query!(
        "UPDATE play_statuses SET label = ?, color = ?, counts_as = ? WHERE key = ?",
        fields.label,
        fields.color,
        fields.counts_as,
        key
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(())
}

/// Deletes a custom status and moves everyone's games in it to `move_to`, or
/// to no status.
pub async fn delete(pool: &SqlitePool, key: &str, move_to: Option<&str>) -> ApiResult<()> {
    let mut tx = write_tx(pool).await?;
    let builtin = sqlx::query_scalar!(
        r#"SELECT builtin AS "builtin: bool" FROM play_statuses WHERE key = ?"#,
        key
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(ApiError::NotFound)?;
    if builtin {
        return Err(ApiError::Conflict("builtin_status"));
    }
    if move_to == Some(key) {
        return Err(ApiError::invalid("unknown_play_status", "move_to"));
    }
    ensure_exists(&mut *tx, move_to)
        .await
        .map_err(|err| match err {
            ApiError::Invalid { code, .. } => ApiError::invalid(code, "move_to"),
            other => other,
        })?;

    match move_to {
        Some(move_to) => {
            // Keeps updated_at, and the history's dates and groups: the
            // owners did not change their play.
            sqlx::query!(
                "UPDATE play_progress SET play_status = ? WHERE play_status = ?",
                move_to,
                key
            )
            .execute(&mut *tx)
            .await?;
            sqlx::query!(
                "UPDATE play_status_changes SET play_status = ? WHERE play_status = ?",
                move_to,
                key
            )
            .execute(&mut *tx)
            .await?;
        }
        None => {
            sqlx::query!("DELETE FROM play_progress WHERE play_status = ?", key)
                .execute(&mut *tx)
                .await?;
        }
    }
    let result = sqlx::query!("DELETE FROM play_statuses WHERE key = ?", key)
        .execute(&mut *tx)
        .await?;
    found(result.rows_affected())?;
    tx.commit().await?;
    Ok(())
}

/// `keys` must list every status exactly once, in the new order.
pub async fn reorder(pool: &SqlitePool, keys: &[String]) -> ApiResult<()> {
    let mut tx = write_tx(pool).await?;
    let mut existing: Vec<String> = names(&mut *tx).await?.into_iter().map(|s| s.key).collect();
    let mut requested = keys.to_vec();
    existing.sort_unstable();
    requested.sort_unstable();
    if existing != requested {
        return Err(ApiError::invalid("invalid_order", "keys"));
    }
    for (position, key) in keys.iter().enumerate() {
        let sort_order = (position as i64 + 1) * 10;
        sqlx::query!(
            "UPDATE play_statuses SET sort_order = ? WHERE key = ?",
            sort_order,
            key
        )
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(())
}

/// Finds a status by key, built-in alias or label, ignoring case and punctuation.
pub fn resolve_name(statuses: &[StatusName], text: &str) -> Option<String> {
    let text = text.trim();
    if let Some(status) = statuses.iter().find(|s| s.key.eq_ignore_ascii_case(text)) {
        return Some(status.key.clone());
    }
    if let Some(builtin) = BuiltinStatus::parse_loose(text) {
        return Some(builtin.as_str().to_owned());
    }
    let wanted = name_key(text);
    statuses
        .iter()
        .find(|s| s.label.as_deref().map(name_key).as_deref() == Some(wanted.as_str()))
        .map(|s| s.key.clone())
}

fn name_key(text: &str) -> String {
    let key = loose(text);
    if key.is_empty() {
        text.trim().to_lowercase()
    } else {
        key
    }
}

/// True when `label` would read like another status, so names stay unambiguous
/// in the UI and in CSV files.
fn name_taken(statuses: &[StatusName], own_key: Option<&str>, label: &str) -> bool {
    let wanted = name_key(label);
    if wanted == NONE {
        return true;
    }
    if let Some(builtin) = BuiltinStatus::parse_loose(label)
        && Some(builtin.as_str()) != own_key
    {
        return true;
    }
    statuses
        .iter()
        .filter(|s| Some(s.key.as_str()) != own_key)
        .any(|s| {
            name_key(&s.key) == wanted
                || s.label.as_deref().map(name_key).as_deref() == Some(&*wanted)
        })
}

/// A lowercase ASCII key from the label, unique among `statuses`.
fn new_key(statuses: &[StatusName], label: &str) -> String {
    let mut base = String::new();
    for c in label.to_lowercase().chars() {
        if c.is_ascii_alphanumeric() {
            base.push(c);
        } else if !base.is_empty() && !base.ends_with('_') {
            base.push('_');
        }
    }
    let mut base: String = base.trim_end_matches('_').chars().take(KEY_MAX).collect();
    base = base.trim_end_matches('_').to_owned();
    if base.is_empty() {
        base = "status".to_owned();
    }
    // `order` would clash with the reorder route.
    let taken = |key: &str| key == NONE || key == "order" || statuses.iter().any(|s| s.key == key);
    if !taken(&base) {
        return base;
    }
    (2..)
        .map(|n| format!("{base}_{n}"))
        .find(|key| !taken(key))
        .expect("a free key")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn statuses() -> Vec<StatusName> {
        let mut list: Vec<StatusName> = BuiltinStatus::ALL
            .iter()
            .map(|s| StatusName {
                key: s.as_str().to_owned(),
                label: None,
            })
            .collect();
        list[0].label = Some("To play".to_owned());
        list.push(StatusName {
            key: "replaying".to_owned(),
            label: Some("Replaying".to_owned()),
        });
        list
    }

    #[test]
    fn resolves_keys_aliases_and_labels() {
        let list = statuses();
        assert_eq!(
            resolve_name(&list, "replaying").as_deref(),
            Some("replaying")
        );
        assert_eq!(
            resolve_name(&list, " Re-playing ").as_deref(),
            Some("replaying")
        );
        assert_eq!(resolve_name(&list, "On hold").as_deref(), Some("paused"));
        assert_eq!(resolve_name(&list, "to play").as_deref(), Some("backlog"));
        assert_eq!(resolve_name(&list, "Backlog").as_deref(), Some("backlog"));
        assert_eq!(resolve_name(&list, "someday"), None);
    }

    #[test]
    fn rejects_names_that_read_like_another_status() {
        let list = statuses();
        assert!(name_taken(&list, None, "Finished"));
        assert!(name_taken(&list, None, "to-play"));
        assert!(name_taken(&list, None, "REPLAYING"));
        assert!(name_taken(&list, None, "None"));
        assert!(!name_taken(&list, None, "Someday"));
        assert!(!name_taken(&list, Some("replaying"), "Replaying!"));
        assert!(!name_taken(&list, Some("paused"), "On hold"));
        assert!(name_taken(&list, Some("paused"), "Dropped"));
    }

    #[test]
    fn makes_unique_ascii_keys() {
        let list = statuses();
        assert_eq!(new_key(&list, "Want to Replay!"), "want_to_replay");
        assert_eq!(new_key(&list, "Replaying"), "replaying_2");
        assert_eq!(new_key(&list, "クリア"), "status");
        assert_eq!(new_key(&list, "Jogando de novo"), "jogando_de_novo");
        assert_eq!(new_key(&list, "Order"), "order_2");
    }
}
