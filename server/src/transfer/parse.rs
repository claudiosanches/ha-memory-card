use std::collections::HashMap;

use serde::Serialize;
use utoipa::ToSchema;

use super::{COLUMNS, MAX_COLUMNS, MAX_ROWS, UTF8_BOM};
use crate::domain::loose;

/// Header names other spreadsheets use, compared loosely, for each column.
const ALIASES: [(&str, &[&str]); 18] = [
    ("game_title", &["title", "name", "game", "gamename"]),
    ("igdb_id", &["igdb"]),
    ("platform", &["system", "console", "platformname"]),
    ("region", &["territory", "zone"]),
    ("release_title", &["localtitle", "regionaltitle"]),
    ("product_code", &["serial", "serialnumber", "productid"]),
    ("format", &["type", "media", "medium"]),
    (
        "play_status",
        &["progress", "completion", "completionstatus", "playstate"],
    ),
    (
        "labels",
        &["label", "tags", "tag", "categories", "category"],
    ),
    ("device", &["account"]),
    (
        "store",
        &["shop", "retailer", "seller", "boughtat", "purchasedat"],
    ),
    ("condition", &["completeness"]),
    ("notes", &["note", "comments", "comment", "remarks"]),
    (
        "purchased_on",
        &[
            "purchasedate",
            "purchased",
            "datepurchased",
            "bought",
            "boughton",
            "acquired",
        ],
    ),
    ("price", &["paid", "cost", "pricepaid", "amount"]),
    (
        "estimated_value",
        &["value", "marketvalue", "currentvalue", "worth", "estimate"],
    ),
    ("barcode", &["upc", "ean", "gtin"]),
    ("edition", &["variant"]),
];

/// The column a header names: its own name, ignoring case, or a common alias.
pub fn column_for(header: &str) -> Option<&'static str> {
    let header = header.trim();
    if let Some(column) = COLUMNS.iter().find(|c| header.eq_ignore_ascii_case(c)) {
        return Some(column);
    }
    let key = loose(header);
    ALIASES
        .iter()
        .find(|(_, aliases)| aliases.contains(&key.as_str()))
        .map(|(column, _)| *column)
}

/// How a header was read. `column` is empty for headers that match nothing
/// or repeat a column an earlier header took.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct HeaderMatch {
    pub header: String,
    pub column: Option<&'static str>,
}

/// The header row as read, and the separator, without reading the rows.
/// `None` for a file [`parse`] refuses before its rows.
pub fn headers(bytes: &[u8]) -> Option<(Vec<HeaderMatch>, char)> {
    let bytes = bytes.strip_prefix(UTF8_BOM).unwrap_or(bytes);
    let text = std::str::from_utf8(bytes).ok()?;
    let delimiter = detect_delimiter(text);
    if too_wide(text.as_bytes(), delimiter) {
        return None;
    }
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(delimiter)
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(text.as_bytes());
    let headers = reader.headers().ok()?;
    let mut taken = Vec::new();
    let matches = headers
        .iter()
        .map(|header| {
            let column = column_for(header).filter(|c| !taken.contains(c));
            if let Some(column) = column {
                taken.push(column);
            }
            HeaderMatch {
                header: header.to_owned(),
                column,
            }
        })
        .collect();
    Some((matches, char::from(delimiter)))
}

/// One CSV row, every cell trimmed; blank cells are `None`.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct RawRow {
    pub line: u64,
    pub copy_id: Option<String>,
    pub status: Option<String>,
    pub igdb_id: Option<String>,
    pub game_title: Option<String>,
    pub play_status: Option<String>,
    /// Label names, separated by commas or semicolons.
    pub labels: Option<String>,
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
    pub estimated_value: Option<String>,
    pub estimated_currency: Option<String>,
    pub barcode: Option<String>,
    pub bundle_copy_id: Option<String>,
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
            &self.estimated_value,
            &self.estimated_currency,
            &self.barcode,
            &self.bundle_copy_id,
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

/// Drops the `'` the export puts before text that would start a formula.
fn unquote(value: &str) -> &str {
    match value.strip_prefix('\'') {
        Some(rest) if rest.starts_with(super::export::FORMULA_START) => rest,
        _ => value,
    }
}

pub fn parse(bytes: &[u8]) -> Result<Vec<RawRow>, FileIssue> {
    let bytes = bytes.strip_prefix(UTF8_BOM).unwrap_or(bytes);
    let text = std::str::from_utf8(bytes).map_err(|_| FileIssue::new("not_utf8"))?;
    if text.trim().is_empty() {
        return Err(FileIssue::new("empty_file"));
    }

    let delimiter = detect_delimiter(text);
    if too_wide(text.as_bytes(), delimiter) {
        return Err(FileIssue::new("too_many_columns"));
    }
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(delimiter)
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
                .position(|header| column_for(header) == Some(column))
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
                .map(unquote)
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
            labels: cell("labels"),
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
            estimated_value: cell("estimated_value"),
            estimated_currency: cell("estimated_currency"),
            barcode: cell("barcode"),
            bundle_copy_id: cell("bundle_copy_id"),
        };
        if row.game_title.is_none()
            && row.igdb_id.is_none()
            && !row.has_copy_columns()
            && row.play_status.is_none()
            && row.labels.is_none()
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
/// Whether any record has more than [`MAX_COLUMNS`] fields. Counted field by
/// field, since the csv reader holds a whole record, at 8 bytes a field,
/// before its width can be checked.
fn too_wide(bytes: &[u8], delimiter: u8) -> bool {
    let mut reader = csv_core::ReaderBuilder::new().delimiter(delimiter).build();
    let mut output = [0u8; 1024];
    let mut input = bytes;
    let mut fields = 0;
    loop {
        let (result, read, _) = reader.read_field(input, &mut output);
        input = &input[read..];
        match result {
            csv_core::ReadFieldResult::Field { record_end } => {
                fields += 1;
                if fields > MAX_COLUMNS {
                    return true;
                }
                if record_end {
                    fields = 0;
                }
            }
            csv_core::ReadFieldResult::End => return false,
            csv_core::ReadFieldResult::InputEmpty | csv_core::ReadFieldResult::OutputFull => {}
        }
    }
}

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
    fn refuses_more_columns_than_the_import_screen_lists() {
        let csv = |columns: usize| format!("game_title{}\nZelda\n", ",".repeat(columns - 1));
        assert!(parse(csv(MAX_COLUMNS).as_bytes()).is_ok());
        assert!(headers(csv(MAX_COLUMNS).as_bytes()).is_some());
        let wide = csv(MAX_COLUMNS + 1);
        assert_eq!(parse(wide.as_bytes()).unwrap_err().code, "too_many_columns");
        assert_eq!(headers(wide.as_bytes()), None);

        // A row wider than its header, and fields that hold the separator.
        let row = format!("game_title,notes\nZelda{}\n", ",".repeat(MAX_COLUMNS));
        assert_eq!(parse(row.as_bytes()).unwrap_err().code, "too_many_columns");
        assert_eq!(headers(row.as_bytes()), None);
        let quoted = format!("game_title,notes\nZelda,\"{}\"\n", ",".repeat(1000));
        assert_eq!(parse(quoted.as_bytes()).unwrap().len(), 1);
    }

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
    fn reads_other_spreadsheets_headers() {
        let rows = parse(
            b"Name,System,UPC,Price Paid,Market Value,Comments\nZelda,Wii,045496590024,10,25,Gift\n",
        )
        .unwrap();
        assert_eq!(rows[0].game_title.as_deref(), Some("Zelda"));
        assert_eq!(rows[0].platform.as_deref(), Some("Wii"));
        assert_eq!(rows[0].barcode.as_deref(), Some("045496590024"));
        assert_eq!(rows[0].price.as_deref(), Some("10"));
        assert_eq!(rows[0].estimated_value.as_deref(), Some("25"));
        assert_eq!(rows[0].notes.as_deref(), Some("Gift"));

        let (headers, delimiter) = headers(b"Title;Game;Rating\nx;y;z\n").unwrap();
        assert_eq!(delimiter, ';');
        assert_eq!(
            headers.iter().map(|h| h.column).collect::<Vec<_>>(),
            [Some("game_title"), None, None],
            "a second title column is left out"
        );
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
            parse(b"what,platform\nZelda,Wii\n").unwrap_err().code,
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
