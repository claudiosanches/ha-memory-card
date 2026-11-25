//! URL names for platforms, devices, stores, genres, game modes and labels: lowercase
//! ASCII letters, digits and hyphens, starting and ending with a letter or digit.

use std::collections::HashSet;

use serde::{Deserialize, Deserializer};

use crate::error::{ApiError, ApiResult};

pub const MAX_LEN: usize = 64;

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
    let taken: HashSet<String> = taken.iter().map(|s| s.to_ascii_lowercase()).collect();
    first_free(base, &taken, 2).0
}

/// The first of `base`, then `base-{from}`, `base-{from + 1}`... not in
/// `taken` (lowercase), with the counter to try next.
pub fn first_free(base: &str, taken: &HashSet<String>, from: u64) -> (String, u64) {
    if !taken.contains(&base.to_ascii_lowercase()) {
        return (base.to_owned(), from);
    }
    (from..)
        .map(|n| {
            let suffix = format!("-{n}");
            let head = &base[..base.len().min(MAX_LEN - suffix.len())];
            (format!("{}{suffix}", head.trim_end_matches('-')), n + 1)
        })
        .find(|(candidate, _)| !taken.contains(candidate))
        .expect("unbounded range")
}

/// The start every slug `first_free` can give for `base` shares: the head
/// left beside the longest counter.
pub fn numbered_prefix(base: &str) -> &str {
    let longest = format!("-{}", u64::MAX).len();
    base[..base.len().min(MAX_LEN - longest)].trim_end_matches('-')
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
