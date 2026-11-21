use std::path::{Path, PathBuf};

use sqlx::SqlitePool;
use sqlx::migrate::Migrator;
use time::OffsetDateTime;
use time::macros::format_description;

use super::DbError;

/// Writes `VACUUM INTO <dir>/<utc timestamp>-v<latest applied>.sqlite` when the
/// database already has migrations applied and the migrator has new ones.
/// A fresh database has nothing worth keeping, so it is skipped.
pub async fn snapshot_if_pending(
    pool: &SqlitePool,
    migrator: &Migrator,
    dir: &Path,
    keep: usize,
) -> Result<Option<PathBuf>, DbError> {
    let applied = applied_versions(pool, migrator).await?;
    let Some(latest) = applied.iter().max().copied() else {
        return Ok(None);
    };

    let pending = migrator
        .iter()
        .any(|m| !m.migration_type.is_down_migration() && !applied.contains(&m.version));
    if !pending {
        return Ok(None);
    }

    std::fs::create_dir_all(dir).map_err(|source| DbError::Io {
        context: format!("cannot create {}", dir.display()),
        source,
    })?;

    let stamp = OffsetDateTime::now_utc()
        .format(format_description!(
            "[year][month][day]T[hour][minute][second]Z"
        ))
        .expect("static format description");
    let target = dir.join(format!("{stamp}-v{latest}.sqlite"));

    sqlx::query("VACUUM INTO ?")
        .bind(target.to_string_lossy().into_owned())
        .execute(pool)
        .await?;

    rotate(dir, keep).map_err(|source| DbError::Io {
        context: format!("cannot rotate snapshots in {}", dir.display()),
        source,
    })?;

    Ok(Some(target))
}

async fn applied_versions(pool: &SqlitePool, migrator: &Migrator) -> Result<Vec<i64>, DbError> {
    let table: &str = &migrator.table_name;
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?)",
    )
    .bind(table)
    .fetch_one(pool)
    .await?;
    if !exists {
        return Ok(Vec::new());
    }

    // SQLite parameters represent values only. Double embedded quotes in identifiers.
    let table = table.replace('"', "\"\"");
    let sql = format!("SELECT version FROM \"{table}\" WHERE success = 1");
    Ok(sqlx::query_scalar(sqlx::AssertSqlSafe(sql))
        .fetch_all(pool)
        .await?)
}

/// Keeps the newest `keep` snapshots. Names start with a UTC timestamp, so
/// lexical order is chronological.
pub fn rotate(dir: &Path, keep: usize) -> std::io::Result<()> {
    let mut snapshots: Vec<PathBuf> = std::fs::read_dir(dir)?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.extension().is_some_and(|ext| ext == "sqlite"))
        .collect();
    snapshots.sort();

    let excess = snapshots.len().saturating_sub(keep);
    for path in snapshots.into_iter().take(excess) {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotate_keeps_newest() {
        let dir = tempfile::tempdir().unwrap();
        for day in 1..=7 {
            std::fs::write(
                dir.path().join(format!("2026110{day}T000000Z-v1.sqlite")),
                b"",
            )
            .unwrap();
        }
        std::fs::write(dir.path().join("notes.txt"), b"").unwrap();

        rotate(dir.path(), 5).unwrap();

        let mut left: Vec<String> = std::fs::read_dir(dir.path())
            .unwrap()
            .map(|e| e.unwrap().file_name().into_string().unwrap())
            .collect();
        left.sort();
        assert_eq!(
            left,
            [
                "20261103T000000Z-v1.sqlite",
                "20261104T000000Z-v1.sqlite",
                "20261105T000000Z-v1.sqlite",
                "20261106T000000Z-v1.sqlite",
                "20261107T000000Z-v1.sqlite",
                "notes.txt",
            ]
        );
    }
}
