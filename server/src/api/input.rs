//! Normalizes user text: trims it, treats blank optional text as absent, and
//! enforces length limits.

use crate::error::ApiError;

pub const TITLE_MAX: usize = 200;
pub const SUMMARY_MAX: usize = 5000;
pub const NAME_MAX: usize = 100;
pub const SHORT_MAX: usize = 20;
pub const NOTES_MAX: usize = 2000;
/// Most IDs one request lists.
pub const IDS_MAX: usize = 1000;

/// Text from another service cut to `N` characters, the most the API takes
/// for the same field.
pub fn cut<const N: usize>(mut text: String) -> String {
    if let Some((at, _)) = text.char_indices().nth(N) {
        text.truncate(at);
    }
    text
}

/// A JSON list of at most [`IDS_MAX`] IDs.
pub fn id_list<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<Vec<i64>, D::Error> {
    let ids: Vec<i64> = serde::Deserialize::deserialize(deserializer)?;
    if ids.len() > IDS_MAX {
        return Err(serde::de::Error::custom(format!("more than {IDS_MAX} ids")));
    }
    Ok(ids)
}

pub fn required(value: &str, field: &'static str, max: usize) -> Result<String, ApiError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(ApiError::invalid("required", field));
    }
    check_len(value, field, max)?;
    Ok(value.to_owned())
}

pub fn optional(
    value: Option<&str>,
    field: &'static str,
    max: usize,
) -> Result<Option<String>, ApiError> {
    match value.map(str::trim).filter(|v| !v.is_empty()) {
        None => Ok(None),
        Some(value) => {
            check_len(value, field, max)?;
            Ok(Some(value.to_owned()))
        }
    }
}

fn check_len(value: &str, field: &'static str, max: usize) -> Result<(), ApiError> {
    if value.chars().count() > max {
        return Err(ApiError::invalid("too_long", field));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn required_trims_and_rejects_blank() {
        assert_eq!(required("  Wii U  ", "name", 10).unwrap(), "Wii U");
        assert!(matches!(
            required("   ", "name", 10),
            Err(ApiError::Invalid {
                code: "required",
                field: Some("name")
            })
        ));
        assert!(matches!(
            required("abcdefghijk", "name", 10),
            Err(ApiError::Invalid {
                code: "too_long",
                ..
            })
        ));
    }

    #[test]
    fn optional_treats_blank_as_absent() {
        assert_eq!(optional(Some("  "), "notes", 10).unwrap(), None);
        assert_eq!(optional(None, "notes", 10).unwrap(), None);
        assert_eq!(
            optional(Some(" x "), "notes", 10).unwrap().as_deref(),
            Some("x")
        );
    }

    #[test]
    fn length_counts_characters_not_bytes() {
        assert!(required("ポケモン", "name", 4).is_ok());
    }
}
