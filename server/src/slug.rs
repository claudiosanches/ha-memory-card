//! URL names for platforms, devices, stores, genres, game modes and labels: lowercase
//! ASCII letters, digits and hyphens, starting and ending with a letter or digit.

use serde::{Deserialize, Deserializer};
use sqlx::{AssertSqlSafe, SqliteConnection};

use crate::error::{ApiError, ApiResult};

pub const MAX_LEN: usize = 64;

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

pub fn is_valid(slug: &str) -> bool {
    !slug.is_empty()
        && slug.len() <= MAX_LEN
        && slug
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        && !slug.starts_with('-')
        && !slug.ends_with('-')
}

/// Accents dropped, lowercase, and each run of other characters made one
/// hyphen. Empty when nothing usable is left.
pub fn slugify(text: &str) -> String {
    use unicode_normalization::UnicodeNormalization;
    use unicode_normalization::char::is_combining_mark;
    let mut slug = String::new();
    for c in text.nfkd().flat_map(char::to_lowercase) {
        if c.is_ascii_alphanumeric() {
            slug.push(c);
        } else if !is_combining_mark(c) && !slug.is_empty() && !slug.ends_with('-') {
            slug.push('-');
        }
    }
    slug.truncate(MAX_LEN);
    slug.trim_end_matches('-').to_owned()
}

/// `base`, or `base-2`, `base-3`... the first one not in `taken`.
pub fn unique(base: &str, taken: &[String]) -> String {
    let free = |candidate: &str| !taken.iter().any(|s| s.eq_ignore_ascii_case(candidate));
    if free(base) {
        return base.to_owned();
    }
    (2..)
        .map(|n| {
            let suffix = format!("-{n}");
            let head = &base[..base.len().min(MAX_LEN - suffix.len())];
            format!("{}{suffix}", head.trim_end_matches('-'))
        })
        .find(|candidate| free(candidate))
        .expect("unbounded range")
}

/// A slug from user input: trimmed and lowercased, `None` when blank.
pub fn input(value: Option<&str>) -> ApiResult<Option<String>> {
    match value.map(str::trim).filter(|v| !v.is_empty()) {
        None => Ok(None),
        Some(value) => {
            let value = value.to_ascii_lowercase();
            if is_valid(&value) {
                Ok(Some(value))
            } else {
                Err(ApiError::invalid("invalid_slug", "slug"))
            }
        }
    }
}

/// A slug for `text` that no row of `table` has yet.
pub async fn free(conn: &mut SqliteConnection, table: Table, text: &str) -> sqlx::Result<String> {
    let base = Some(slugify(text))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| table.fallback().to_owned());
    let taken: Vec<String> = sqlx::query_scalar(AssertSqlSafe(format!(
        "SELECT slug FROM {} WHERE slug = ?1 OR slug LIKE ?1 || '-%'",
        table.name()
    )))
    .bind(&base)
    .fetch_all(conn)
    .await?;
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
    let taken: bool = sqlx::query_scalar(AssertSqlSafe(format!(
        "SELECT EXISTS (SELECT 1 FROM {} WHERE name = ?1 AND id IS NOT ?2)",
        table.name()
    )))
    .bind(name)
    .bind(id)
    .fetch_one(conn)
    .await?;
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

/// A slug in a filter. Any case is accepted, since slugs compare without case.
#[derive(Debug, Clone, PartialEq, Eq, sqlx::Type)]
#[sqlx(transparent)]
pub struct Slug(String);

impl Slug {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for Slug {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?.to_ascii_lowercase();
        if is_valid(&raw) {
            Ok(Self(raw))
        } else {
            Err(serde::de::Error::custom("invalid slug"))
        }
    }
}

impl From<&str> for Slug {
    /// For tests and callers that already hold a stored slug.
    fn from(value: &str) -> Self {
        Self(value.to_ascii_lowercase())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugify_names() {
        assert_eq!(
            slugify("Sega Mega Drive/Genesis"),
            "sega-mega-drive-genesis"
        );
        assert_eq!(slugify("  Xbox Series X|S "), "xbox-series-x-s");
        assert_eq!(slugify("itch.io"), "itch-io");
        assert_eq!(slugify("Pokémon Mini"), "pokemon-mini");
        assert_eq!(
            slugify("Hack and slash/Beat 'em up"),
            "hack-and-slash-beat-em-up"
        );
        assert_eq!(slugify("ポケモン"), "");
        assert_eq!(slugify(&"a".repeat(70)).len(), MAX_LEN);
        assert_eq!(slugify(&format!("{}-b", "a".repeat(63))), "a".repeat(63));
    }

    #[test]
    fn validates() {
        for ok in ["switch", "ps5", "edsac--1", "3ds", "a"] {
            assert!(is_valid(ok), "{ok}");
        }
        for bad in [
            "",
            "Switch",
            "-ps5",
            "ps5-",
            "wii u",
            "itch.io",
            "pokémon",
            &"a".repeat(65),
        ] {
            assert!(!is_valid(bad), "{bad}");
        }
    }

    #[test]
    fn input_trims_lowercases_and_rejects() {
        assert_eq!(input(Some("  Steam ")).unwrap().as_deref(), Some("steam"));
        assert_eq!(input(Some("  ")).unwrap(), None);
        assert!(matches!(
            input(Some("wii u")),
            Err(ApiError::Invalid {
                code: "invalid_slug",
                field: Some("slug")
            })
        ));
    }

    #[test]
    fn unique_appends_counter() {
        let taken = vec!["dreamcast".to_owned(), "dreamcast-2".to_owned()];
        assert_eq!(unique("saturn", &taken), "saturn");
        assert_eq!(unique("dreamcast", &taken), "dreamcast-3");
        let long = "a".repeat(MAX_LEN);
        assert_eq!(
            unique(&long, std::slice::from_ref(&long)),
            format!("{}-2", "a".repeat(MAX_LEN - 2))
        );
    }

    #[test]
    fn filter_slugs_ignore_case() {
        let slug: Slug = serde_json::from_str("\"Switch\"").unwrap();
        assert_eq!(slug.as_str(), "switch");
        assert!(serde_json::from_str::<Slug>("\"wii u\"").is_err());
    }
}
