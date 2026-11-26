use super::{COLUMNS, UTF8_BOM};
use crate::db::Pool;
use crate::db::library::LibraryFilter;
use crate::db::transfer::export::rows;
use crate::money;

/// A cell that starts with one of these runs as a formula in a spreadsheet.
const FORMULA_START: [char; 6] = ['=', '+', '-', '@', '\t', '\r'];
/// Where another cell starts inside a field for a spreadsheet that splits the
/// file on semicolons or tabs, as Excel does where `;` separates lists.
const CELL_BREAKS: [char; 4] = [';', '\t', '\n', '\r'];

/// Whether a cell starting with `text` would run as a formula, past the
/// spaces and quotes a spreadsheet skips. Apostrophes are skipped too, so
/// text already starting with one gets another and reads back whole.
fn opens_formula(text: &str) -> bool {
    text.trim_start_matches([' ', '"', '\''])
        .starts_with(FORMULA_START)
}

/// Whether a field starting with `text` would run as a formula, past the
/// apostrophes [`plain`] adds.
fn starts_formula(text: &str) -> bool {
    text.trim_start_matches('\'').starts_with(FORMULA_START)
}

/// Free text that would start a formula, at its start or after a cell break,
/// gets a `'` there, which spreadsheets show as text and [`unplain`] drops.
pub(crate) fn plain(text: String) -> String {
    let mut out = String::with_capacity(text.len() + 1);
    if starts_formula(&text) {
        out.push('\'');
    }
    for (at, c) in text.char_indices() {
        out.push(c);
        if CELL_BREAKS.contains(&c) && opens_formula(&text[at + c.len_utf8()..]) {
            out.push('\'');
        }
    }
    out
}

/// Undoes [`plain`] for a cell the import reads.
pub fn unplain(text: &str) -> String {
    let text = match text.strip_prefix('\'') {
        Some(rest) if starts_formula(rest) => rest,
        _ => text,
    };
    let mut out = String::with_capacity(text.len());
    let mut after_break = false;
    for (at, c) in text.char_indices() {
        if after_break && c == '\'' && opens_formula(&text[at + 1..]) {
            after_break = false;
            continue;
        }
        after_break = CELL_BREAKS.contains(&c);
        out.push(c);
    }
    out
}

pub async fn export(pool: &Pool, filter: &LibraryFilter) -> Result<Vec<u8>, ExportError> {
    let rows = rows(pool, filter).await?;

    let mut out = UTF8_BOM.to_vec();
    {
        let mut writer = csv::Writer::from_writer(&mut out);
        writer.write_record(COLUMNS)?;
        for row in rows {
            let id = |v: Option<i64>| v.map(|v| v.to_string()).unwrap_or_default();
            let text = |v: Option<String>| v.unwrap_or_default();
            let free = |v: Option<String>| plain(v.unwrap_or_default());
            let price = row
                .price
                .zip(row.currency.as_deref())
                .map(|(minor, currency)| money::format(minor, currency));
            let estimated_value = row
                .estimated_value
                .zip(row.estimated_currency.as_deref())
                .map(|(minor, currency)| money::format(minor, currency));
            writer.write_record([
                id(row.copy_id),
                text(row.status.map(|v| v.as_str().to_owned())),
                id(row.igdb_id),
                plain(row.game_title),
                text(row.play_status),
                free(row.labels),
                free(row.platform),
                text(row.region.map(|v| v.as_str().to_owned())),
                free(row.release_title),
                free(row.product_code),
                text(row.format.map(|v| v.as_str().to_owned())),
                free(row.edition),
                free(row.device),
                free(row.store),
                text(row.condition.map(|v| v.as_str().to_owned())),
                text(row.purchased_on),
                text(price),
                text(row.currency),
                text(estimated_value),
                text(row.estimated_currency),
                text(row.barcode),
                free(row.notes),
                id(row.bundle_copy_id),
            ])?;
        }
        writer.flush()?;
    }
    Ok(out)
}

#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error(transparent)]
    Csv(#[from] csv::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formulas_at_any_cell_break_export_as_text_and_read_back() {
        for (text, exported) in [
            ("=1+1", "'=1+1"),
            ("Zelda;=1+1", "Zelda;'=1+1"),
            ("a; \"=1", "a;' \"=1"),
            ("Box\n- manual\r\n+ map", "Box\n'- manual\r\n'+ map"),
            ("a\t\tb", "a\t'\tb"),
            ("x; y", "x; y"),
            ("Mario & Luigi", "Mario & Luigi"),
            ("'Splosion Man", "'Splosion Man"),
            ("'+ edition", "''+ edition"),
            ("Box;'=x", "Box;''=x"),
            ("", ""),
        ] {
            assert_eq!(plain(text.to_owned()), exported, "{text:?}");
            assert_eq!(unplain(exported), text, "{exported:?}");
        }
    }
}
