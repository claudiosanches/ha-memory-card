pub mod devices;
pub mod duplicates;
pub mod games;
pub mod labels;
pub mod library;
pub mod missing;
pub mod photos;
pub mod platforms;
pub mod play_statuses;
pub mod premigration;
pub mod sensors;
pub mod sessions;
pub mod stats;
pub mod stores;
pub mod suggestions;
pub mod tags;
pub mod titles;
pub mod users;

use std::path::Path;
use std::time::Duration;

use sqlx::SqlitePool;
use sqlx::migrate::Migrator;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};

use crate::error::{ApiError, ApiResult};

pub static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

/// Pre-migration snapshots kept on disk.
pub const PRE_MIGRATION_KEEP: usize = 5;

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Migrate(#[from] sqlx::migrate::MigrateError),
    #[error("{context}: {source}")]
    Io {
        context: String,
        source: std::io::Error,
    },
}

/// Opens the database, snapshots it if migrations are pending, then migrates.
pub async fn open(
    path: &Path,
    pre_migration_dir: &Path,
    migrator: &Migrator,
) -> Result<SqlitePool, DbError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|source| DbError::Io {
            context: format!("cannot create {}", parent.display()),
            source,
        })?;
    }

    let options = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal)
        .foreign_keys(true)
        .busy_timeout(Duration::from_secs(5));

    let pool = SqlitePoolOptions::new()
        .max_connections(4)
        .connect_with(options)
        .await?;

    if let Some(snapshot) =
        premigration::snapshot_if_pending(&pool, migrator, pre_migration_dir, PRE_MIGRATION_KEEP)
            .await?
    {
        tracing::info!(path = %snapshot.display(), "saved pre-migration snapshot");
    }
    migrator.run(&pool).await?;

    Ok(pool)
}

/// Folds the WAL back into the main file so a stopped app leaves one self-contained database.
pub async fn checkpoint(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query("PRAGMA wal_checkpoint(TRUNCATE)")
        .execute(pool)
        .await
        .map(|_| ())
}

pub async fn close(pool: &SqlitePool) {
    match checkpoint(pool).await {
        Ok(()) => tracing::info!("database checkpointed"),
        Err(err) => tracing::warn!(error = %err, "database checkpoint failed"),
    }
    pool.close().await;
}

/// Starts a transaction that takes the write lock up front. A deferred
/// transaction that reads first fails with SQLITE_BUSY, without waiting, when
/// another connection commits before its first write.
pub async fn write_tx(pool: &SqlitePool) -> sqlx::Result<sqlx::Transaction<'static, sqlx::Sqlite>> {
    pool.begin_with("BEGIN IMMEDIATE").await
}

/// `NotFound` when a write touched no row.
pub(crate) fn found(rows: u64) -> ApiResult<()> {
    if rows == 0 {
        Err(ApiError::NotFound)
    } else {
        Ok(())
    }
}
