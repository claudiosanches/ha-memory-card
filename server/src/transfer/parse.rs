use std::collections::HashMap;

use serde::Serialize;
use utoipa::ToSchema;

use super::{COLUMNS, MAX_ROWS, UTF8_BOM};

/// One CSV row, every cell trimmed; blank cells are `None`.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct RawRow {
    pub line: u64,
    pub copy_id: Option<String>,
    pub status: Option<String>,
    pub igdb_id: Option<String>,
    pub game_title: Option<String>,
    pub play_status: Option<String>,
    pub platform: Option<String>,
    pub region: Option<String>,
    pub release_title: Option<String>,
    pub product_code: Option<String>,
    pub format: Option<String>,
    pub device: Option<String>,
    pub store: Option<String>,
    pub condition: Option<String>,
    pub notes: Option<String>,
    pub edition: Option<String>,
    pub purchased_on: Option<String>,
    pub price: Option<String>,
    pub currency: Option<String>,
}

impl RawRow {
    /// True when the row carries anything beyond the game columns.
    pub fn has_copy_columns(&self) -> bool {
        [
            &self.copy_id,
            &self.status,
            &self.platform,
            &self.region,
            &self.release_title,
            &self.product_code,
            &self.format,
            &self.device,
            &self.store,
            &self.condition,
            &self.notes,
            &self.edition,
            &self.purchased_on,
            &self.price,
            &self.currency,
        ]
        .iter()
        .any(|cell| cell.is_some())
    }
}

/// A problem with the file as a whole; no rows are read when there is one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct FileIssue {
    pub code: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u64>,
}

impl FileIssue {
    fn new(code: &'static str) -> Self {
        Self {
            code,
            column: None,
            line: None,
        }
    }
}

pub fn parse(bytes: &[u8]) -> Result<Vec<RawRow>, FileIssue> {
    let bytes = bytes.strip_prefix(UTF8_BOM).unwrap_or(bytes);
    let text = std::str::from_utf8(bytes).map_err(|_| FileIssue::new("not_utf8"))?;
    if text.trim().is_empty() {
        return Err(FileIssue::new("empty_file"));
    }

    let mut reader = csv::ReaderBuilder::new()
        .delimiter(detect_delimiter(text))
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(text.as_bytes());

    let headers = reader
        .headers()
        .map_err(|_| FileIssue::new("malformed"))?
        .clone();
    let index: HashMap<&'static str, usize> = COLUMNS
        .iter()
        .filter_map(|column| {
            headers
                .iter()
                .position(|header| header.trim().eq_ignore_ascii_case(column))
                .map(|position| (*column, position))
        })
        .collect();
    if !index.contains_key("game_title") {
        return Err(FileIssue {
            column: Some("game_title"),
            ..FileIssue::new("missing_column")
        });
    }

    let mut rows = Vec::new();
    for record in reader.records() {
        let record = record.map_err(|err| FileIssue {
            line: err.position().map(|p| p.line()),
            ..FileIssue::new("malformed")
        })?;
        let cell = |column: &str| {
            index
                .get(column)
                .and_then(|&i| record.get(i))
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_owned)
        };
        let row = RawRow {
            line: record.position().map_or(0, |p| p.line()),
            copy_id: cell("copy_id"),
            status: cell("status"),
            igdb_id: cell("igdb_id"),
            game_title: cell("game_title"),
            play_status: cell("play_status"),
            platform: cell("platform"),
            region: cell("region"),
            release_title: cell("release_title"),
            product_code: cell("product_code"),
            format: cell("format"),
            device: cell("device"),
            store: cell("store"),
            condition: cell("condition"),
            notes: cell("notes"),
            edition: cell("edition"),
            purchased_on: cell("purchased_on"),
            price: cell("price"),
            currency: cell("currency"),
        };
        if row.game_title.is_none()
            && row.igdb_id.is_none()
            && !row.has_copy_columns()
            && row.play_status.is_none()
        {
            continue;
        }
        rows.push(row);
        if rows.len() > MAX_ROWS {
            return Err(FileIssue::new("too_many_rows"));
        }
    }

    if rows.is_empty() {
        return Err(FileIssue::new("empty_file"));
    }
    Ok(rows)
}

/// Spreadsheets in many locales save CSV with `;`. Picks whichever of `,` and
/// `;` appears more in the header line.
fn detect_delimiter(text: &str) -> u8 {
    let header = text.lines().next().unwrap_or_default();
    let commas = header.matches(',').count();
    let semicolons = header.matches(';').count();
    if semicolons > commas { b';' } else { b',' }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_rows_with_any_header_case_and_order() {
        let rows =
            parse(b"Platform,GAME_TITLE,notes\nWii, Mario Kart ,\n\n,,\nSwitch,Zelda,Gift\n")
                .unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].line, 2);
        assert_eq!(rows[0].game_title.as_deref(), Some("Mario Kart"));
        assert_eq!(rows[0].platform.as_deref(), Some("Wii"));
        assert_eq!(rows[0].notes, None);
        assert_eq!(rows[1].line, 5);
        assert_eq!(rows[1].notes.as_deref(), Some("Gift"));
    }

    #[test]
    fn strips_bom_and_reads_semicolons() {
        let mut bytes = UTF8_BOM.to_vec();
        bytes.extend_from_slice("game_title;platform\n\"Pokémon X; Y\";3DS\n".as_bytes());
        let rows = parse(&bytes).unwrap();
        assert_eq!(rows[0].game_title.as_deref(), Some("Pokémon X; Y"));
        assert_eq!(rows[0].platform.as_deref(), Some("3DS"));
    }

    #[test]
    fn rejects_bad_files() {
        assert_eq!(parse(b"").unwrap_err().code, "empty_file");
        assert_eq!(parse(b"game_title\n").unwrap_err().code, "empty_file");
        assert_eq!(
            parse(b"title,platform\nZelda,Wii\n").unwrap_err().code,
            "missing_column"
        );
        assert_eq!(
            parse(b"game_title\n\xff\xfe\n").unwrap_err().code,
            "not_utf8"
        );
    }

    #[test]
    fn template_parses_as_empty() {
        assert_eq!(
            parse(&super::super::template()).unwrap_err().code,
            "empty_file"
        );
    }
}
