mod common;

use std::path::{Path, PathBuf};

use common::TestApp;
use memory_card::db;
use sqlx::migrate::Migrator;

fn fixture(version: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/migrations")
        .join(version)
}

fn snapshots(dir: &Path) -> Vec<PathBuf> {
    match std::fs::read_dir(dir) {
        Ok(entries) => entries.map(|e| e.unwrap().path()).collect(),
        Err(_) => Vec::new(),
    }
}

#[tokio::test]
async fn fresh_database_has_schema_and_seed() {
    let app = TestApp::new().await;

    let tables: Vec<String> = sqlx::query_scalar(
        "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE '\\_%' ESCAPE '\\' ORDER BY name",
    )
    .fetch_all(&app.pool)
    .await
    .unwrap();
    assert_eq!(
        tables,
        [
            "copies",
            "copy_photos",
            "device_platforms",
            "devices",
            "game_game_modes",
            "game_genres",
            "game_links",
            "game_modes",
            "game_names",
            "games",
            "genres",
            "nfc_tags",
            "platforms",
            "play_sessions",
            "play_statuses",
            "releases",
            "settings",
            "stores",
            "suggestions",
            "users",
        ]
    );

    let modes: Vec<String> = sqlx::query_scalar("SELECT slug FROM game_modes ORDER BY id")
        .fetch_all(&app.pool)
        .await
        .unwrap();
    assert_eq!(
        modes,
        [
            "single-player",
            "multiplayer",
            "co-op",
            "split-screen",
            "mmo",
            "battle-royale"
        ]
    );

    let active: Vec<String> =
        sqlx::query_scalar("SELECT slug FROM platforms WHERE active = 1 ORDER BY sort_order")
            .fetch_all(&app.pool)
            .await
            .unwrap();
    assert_eq!(
        active,
        ["pc", "ps5", "ps4", "switch", "wii-u", "wii", "3ds"]
    );

    // A fresh database is not snapshotted.
    assert!(snapshots(&app.config.pre_migration_dir()).is_empty());
}

#[tokio::test]
async fn names_are_unique_ignoring_case() {
    let app = TestApp::new().await;
    sqlx::query("INSERT INTO stores (name, slug) VALUES ('Amazon', 'amazon')")
        .execute(&app.pool)
        .await
        .unwrap();
    let duplicate = sqlx::query("INSERT INTO stores (name, slug) VALUES ('amazon', 'amazon-2')")
        .execute(&app.pool)
        .await;
    assert!(duplicate.is_err());
    let slug = sqlx::query("INSERT INTO stores (name, slug) VALUES ('Amazon US', 'AMAZON')")
        .execute(&app.pool)
        .await;
    assert!(slug.is_err());
}

#[tokio::test]
async fn release_identity_treats_missing_product_code_as_one_value() {
    let app = TestApp::new().await;
    sqlx::query("INSERT INTO games (id, title, sort_title) VALUES (1, 'Game', 'game')")
        .execute(&app.pool)
        .await
        .unwrap();
    let insert = "INSERT INTO releases (game_id, platform_id, region) \
                  VALUES (1, (SELECT id FROM platforms WHERE slug = 'wii'), 'eu')";
    sqlx::query(insert).execute(&app.pool).await.unwrap();
    assert!(sqlx::query(insert).execute(&app.pool).await.is_err());
}

#[tokio::test]
async fn pending_migration_snapshots_existing_database() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("db/memory-card.sqlite");
    let snapshot_dir = dir.path().join("db/pre-migration");

    let v1 = Migrator::new(fixture("v1")).await.unwrap();
    let pool = db::open(&db_path, &snapshot_dir, &v1).await.unwrap();
    sqlx::query("INSERT INTO items (name) VALUES ('kept')")
        .execute(&pool)
        .await
        .unwrap();
    pool.close().await;

    // Reopening with nothing pending takes no snapshot.
    let pool = db::open(&db_path, &snapshot_dir, &v1).await.unwrap();
    pool.close().await;
    assert!(snapshots(&snapshot_dir).is_empty());

    let v2 = Migrator::new(fixture("v2")).await.unwrap();
    let pool = db::open(&db_path, &snapshot_dir, &v2).await.unwrap();
    pool.close().await;

    let taken = snapshots(&snapshot_dir);
    assert_eq!(taken.len(), 1);
    let name = taken[0].file_name().unwrap().to_string_lossy().into_owned();
    assert!(name.ends_with("-v1.sqlite"), "{name}");

    let snapshot = sqlx::SqlitePool::connect(&format!("sqlite://{}?mode=ro", taken[0].display()))
        .await
        .unwrap();
    let names: Vec<String> = sqlx::query_scalar("SELECT name FROM items")
        .fetch_all(&snapshot)
        .await
        .unwrap();
    assert_eq!(names, ["kept"]);
    let has_note: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM pragma_table_info('items') WHERE name = 'note')",
    )
    .fetch_one(&snapshot)
    .await
    .unwrap();
    assert!(!has_note, "snapshot predates the migration");
}

#[tokio::test]
async fn close_leaves_no_wal_content() {
    let app = TestApp::new().await;
    for i in 0..50 {
        sqlx::query("INSERT INTO stores (name, slug) VALUES (?, ?)")
            .bind(format!("Store {i}"))
            .bind(format!("store-{i}"))
            .execute(&app.pool)
            .await
            .unwrap();
    }

    let wal = PathBuf::from(format!("{}-wal", app.config.db_path().display()));
    assert!(std::fs::metadata(&wal).map(|m| m.len()).unwrap_or(0) > 0);

    db::close(&app.pool).await;

    let wal_len = std::fs::metadata(&wal).map(|m| m.len()).unwrap_or(0);
    assert_eq!(wal_len, 0);
}
