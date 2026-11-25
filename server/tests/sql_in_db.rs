//! Where SQL may be written. `clippy.toml` covers the query functions and the
//! builder types; this covers what clippy cannot: the query macros, which it
//! reports whatever a module allows, and `AssertSqlSafe(...)` calls.

use std::fs;
use std::path::{Path, PathBuf};

/// `query!(`, `query_as! {`, `query_file_scalar_unchecked![`... at `at`,
/// with spaces around the `!` or the delimiter on the next line.
fn is_query_macro(line: &str, at: usize) -> bool {
    let before = line[..at].chars().next_back();
    if before.is_some_and(|c| c.is_alphanumeric() || c == '_') {
        return false;
    }
    let rest = &line[at + "query".len()..];
    let suffix = rest.trim_start_matches(|c: char| c.is_ascii_lowercase() || c == '_');
    suffix
        .trim_start()
        .strip_prefix('!')
        .map(str::trim_start)
        .is_some_and(|after| after.is_empty() || after.starts_with(['(', '[', '{']))
}

fn has_query_macro(line: &str) -> bool {
    line.match_indices("query")
        .any(|(at, _)| is_query_macro(line, at))
}

fn rust_files(dir: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            rust_files(&path, files);
        } else if path.extension().is_some_and(|e| e == "rs") {
            files.push(path);
        }
    }
}

/// `file:line` of the lines in `src`, outside `allowed`, that `matches`.
fn lines_outside(allowed: &str, matches: fn(&str) -> bool) -> Vec<String> {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    rust_files(&src, &mut files);
    assert!(files.len() > 10, "found the sources");
    let mut found = Vec::new();
    for path in files.iter().filter(|p| !p.starts_with(src.join(allowed))) {
        let text = fs::read_to_string(path).unwrap();
        for (n, line) in text.lines().enumerate() {
            if matches(line) {
                found.push(format!("{}:{}", path.display(), n + 1));
            }
        }
    }
    found
}

#[test]
fn query_macros_live_in_db() {
    let found = lines_outside("db", has_query_macro);
    assert!(found.is_empty(), "query macros outside src/db: {found:#?}");
}

#[test]
fn sql_text_is_never_asserted_safe() {
    let found = lines_outside("db/sql.rs", |line| line.contains("AssertSqlSafe"));
    assert!(found.is_empty(), "use db::sql::Sql instead: {found:#?}");
}

#[test]
fn finds_query_macros() {
    for line in [
        "sqlx::query!(\"SELECT 1\")",
        "let rows = sqlx::query_as!(Row, \"SELECT 1\")",
        "query_file_scalar_unchecked!(\"a.sql\")",
        "sqlx::query! { \"SELECT 1\" }",
        "sqlx::query_scalar![\"SELECT 1\"]",
        "sqlx::query !(\"SELECT 1\")",
        "let row = sqlx::query_as!",
    ] {
        assert!(has_query_macro(line), "{line}");
    }
    for line in [
        "sqlx::query(\"SELECT 1\")",
        "subquery!(x)",
        "// a query",
        "query != other",
    ] {
        assert!(!has_query_macro(line), "{line}");
    }
}
