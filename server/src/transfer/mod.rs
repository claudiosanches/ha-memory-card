//! CSV import and export. One row per copy; a row with only game columns
//! describes a game with no copies.

pub mod export;
pub mod parse;
pub mod plan;

/// Column order for export and the blank template.
pub const COLUMNS: [&str; 23] = [
    "copy_id",
    "status",
    "igdb_id",
    "game_title",
    "play_status",
    "labels",
    "platform",
    "region",
    "release_title",
    "product_code",
    "format",
    "edition",
    "device",
    "store",
    "condition",
    "purchased_on",
    "price",
    "currency",
    "estimated_value",
    "estimated_currency",
    "barcode",
    "notes",
    "bundle_copy_id",
];

/// Excel only reads a CSV as UTF-8 when it starts with a BOM.
pub const UTF8_BOM: &[u8] = b"\xEF\xBB\xBF";

/// Upper bound on rows per import.
pub const MAX_ROWS: usize = 20_000;

/// Upper bound on columns, each one a choice on the import screen.
pub const MAX_COLUMNS: usize = 200;

pub fn template() -> Vec<u8> {
    let mut out = UTF8_BOM.to_vec();
    out.extend_from_slice(COLUMNS.join(",").as_bytes());
    out.extend_from_slice(b"\r\n");
    out
}
