//! CSV import and export.

mod common;

use axum::http::StatusCode;
use common::TestApp;
use serde_json::{Value, json};

const HEADER: &str = "copy_id,status,igdb_id,game_title,play_status,platform,region,release_title,product_code,format,device,store,condition,notes";

fn issues(preview: &Value, line: u64) -> Vec<(String, String)> {
    let row = preview["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["line"] == line)
        .unwrap_or_else(|| panic!("no row at line {line}: {preview}"));
    row["issues"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| {
            (
                i["column"].as_str().unwrap().to_owned(),
                i["code"].as_str().unwrap().to_owned(),
            )
        })
        .collect()
}

async fn export(app: &TestApp) -> String {
    let (status, bytes) = app.get("/api/export.csv").await;
    assert_eq!(status, StatusCode::OK);
    String::from_utf8(bytes).unwrap()
}

#[tokio::test]
async fn template_is_the_header_row() {
    let app = TestApp::new().await;
    let (status, bytes) = app.get("/api/import/template.csv").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(bytes, format!("\u{feff}{HEADER}\r\n").into_bytes());
}

#[tokio::test]
async fn preview_plans_without_writing() {
    let app = TestApp::new().await;
    let csv = format!(
        "{HEADER}\n\
         ,,,Super Mario Odyssey,completed,Nintendo Switch,na,,,physical,Switch OLED,Amazon,cib,\n\
         ,,,Super Mario Odyssey,,Switch,eu,,,digital,Switch OLED,Nintendo eShop,,\n\
         ,,,Mario Kart 8 Deluxe,,switch,eu,,,digital,Nintendo account,Nintendo eShop,,\n\
         ,,,Shenmue,,Sega Dreamcast,jp,,,physical,,,loose,Import\n"
    );
    let (status, preview) = app.post_csv("/api/import/preview", &csv).await;
    assert_eq!(status, StatusCode::OK, "{preview}");
    assert_eq!(preview["file_issue"], Value::Null);
    assert_eq!(
        preview["summary"],
        json!({
            "rows": 4, "new_games": 3, "updated_games": 0, "new_releases": 4,
            "updated_releases": 0, "new_copies": 4, "updated_copies": 0,
            "unchanged_copies": 0, "game_only_rows": 0, "error_rows": 0, "warning_rows": 0
        })
    );
    assert_eq!(preview["new_platforms"], json!(["Sega Dreamcast"]));
    assert_eq!(preview["new_stores"], json!(["Amazon"]));
    assert_eq!(
        preview["new_devices"],
        json!([
            {"name": "Switch OLED", "kind": "console", "platforms": ["Nintendo Switch"]},
            {"name": "Nintendo account", "kind": "account", "platforms": ["Nintendo Switch"]},
        ])
    );

    assert_eq!(app.scalar("SELECT COUNT(*) FROM games").await, 0);
    assert_eq!(
        app.scalar("SELECT COUNT(*) FROM stores WHERE name = 'Amazon'")
            .await,
        0
    );
}

#[tokio::test]
async fn import_then_export_round_trips() {
    let app = TestApp::new().await;
    let csv = format!(
        "{HEADER}\n\
         ,,1234,Super Mario Odyssey,completed,Nintendo Switch,na,,HAC-AAACA,physical,Switch OLED,Amazon,cib,\"Gift, 2017\"\n\
         ,,,Super Mario Odyssey,,Switch,eu,Super Mario Odyssey (PAL),,digital,,,,\n\
         ,,,Zelda: Breath of the Wild,mastered,,,,,,,,,\n"
    );
    let (status, result) = app.post_csv("/api/import", &csv).await;
    assert_eq!(status, StatusCode::OK, "{result}");
    assert_eq!(result["summary"]["new_games"], 2);
    assert_eq!(result["summary"]["new_copies"], 2);
    assert_eq!(result["summary"]["game_only_rows"], 1);

    let (igdb, status, sort): (Option<i64>, Option<String>, String) = sqlx::query_as(
        "SELECT igdb_id, play_status, sort_title FROM games WHERE title = 'Super Mario Odyssey'",
    )
    .fetch_one(&app.pool)
    .await
    .unwrap();
    assert_eq!(igdb, Some(1234));
    assert_eq!(status.as_deref(), Some("completed"));
    assert_eq!(sort, "super mario odyssey");

    let exported = export(&app).await;
    let lines: Vec<&str> = exported.trim_start_matches('\u{feff}').lines().collect();
    assert_eq!(lines[0], HEADER);
    assert_eq!(lines.len(), 4, "{exported}");
    // Same platform: rows sort by region, so eu comes before na.
    assert!(
        lines[1].ends_with(",owned,1234,Super Mario Odyssey,completed,Nintendo Switch,eu,Super Mario Odyssey (PAL),,digital,,,,"),
        "{}",
        lines[1]
    );
    assert!(
        lines[2].ends_with(",owned,1234,Super Mario Odyssey,completed,Nintendo Switch,na,,HAC-AAACA,physical,Switch OLED,Amazon,cib,\"Gift, 2017\""),
        "{}",
        lines[2]
    );
    assert_eq!(lines[3], ",,,Zelda: Breath of the Wild,mastered,,,,,,,,,");

    // Importing the export again changes nothing.
    let (_, preview) = app.post_csv("/api/import/preview", &exported).await;
    assert_eq!(
        preview["summary"],
        json!({
            "rows": 3, "new_games": 0, "updated_games": 0, "new_releases": 0,
            "updated_releases": 0, "new_copies": 0, "updated_copies": 0,
            "unchanged_copies": 2, "game_only_rows": 1, "error_rows": 0, "warning_rows": 0
        }),
        "{preview}"
    );
}

#[tokio::test]
async fn edited_export_updates_copies() {
    let app = TestApp::new().await;
    app.post_csv(
        "/api/import",
        &format!("{HEADER}\n,,,Metroid Dread,,Switch,eu,,,physical,,,,\n"),
    )
    .await;
    let exported = export(&app).await;
    let edited = exported.replace(",physical,,,,", ",physical,,Amazon,sealed,Still wrapped");

    let (_, result) = app.post_csv("/api/import", &edited).await;
    assert_eq!(result["summary"]["updated_copies"], 1, "{result}");
    assert_eq!(result["summary"]["new_copies"], 0);

    let (store, condition, notes): (String, String, String) = sqlx::query_as(
        "SELECT s.name, c.condition, c.notes FROM copies c JOIN stores s ON s.id = c.store_id",
    )
    .fetch_one(&app.pool)
    .await
    .unwrap();
    assert_eq!(
        (store.as_str(), condition.as_str(), notes.as_str()),
        ("Amazon", "sealed", "Still wrapped")
    );
}

#[tokio::test]
async fn names_and_values_match_loosely() {
    let app = TestApp::new().await;
    let wii_u = app.platform_id("wii-u").await;
    app.json(
        "POST",
        "/api/devices",
        Some(json!({"name": "Wii U", "kind": "console", "platform_ids": [wii_u]})),
    )
    .await;
    app.json("POST", "/api/stores", Some(json!({"name": "Amazon"})))
        .await;

    let csv = "game_title;platform;region;format;device;store;condition;play_status\n\
               Splatoon;WII U;PAL;Disc;wii u;AMAZON;Complete;Backlog\n\
               Gran Turismo 7;ps5;NTSC-U;Download;;;;\n\
               Pokemon X;3ds;Region free;cartridge;;;No manual;100%\n";
    let (_, preview) = app.post_csv("/api/import/preview", csv).await;
    assert_eq!(preview["summary"]["error_rows"], 0, "{preview}");
    assert_eq!(preview["new_platforms"], json!([]));
    assert_eq!(preview["new_devices"], json!([]));
    assert_eq!(preview["new_stores"], json!([]));

    app.post_csv("/api/import", csv).await;
    let values: Vec<(String, String, String, Option<String>)> = sqlx::query_as(
        "SELECT g.title, r.region, c.format, c.condition FROM copies c
         JOIN releases r ON r.id = c.release_id JOIN games g ON g.id = r.game_id ORDER BY g.title",
    )
    .fetch_all(&app.pool)
    .await
    .unwrap();
    assert_eq!(
        values,
        [
            ("Gran Turismo 7".into(), "na".into(), "digital".into(), None),
            (
                "Pokemon X".into(),
                "ww".into(),
                "physical".into(),
                Some("no_manual".into())
            ),
            (
                "Splatoon".into(),
                "eu".into(),
                "physical".into(),
                Some("cib".into())
            ),
        ]
    );
}

#[tokio::test]
async fn errors_block_the_import() {
    let app = TestApp::new().await;
    let csv = format!(
        "{HEADER}\n\
         ,,,Halo,,Xbox One,,,,physical,,,,\n\
         ,,,Fortnite,,PC,ww,,,digital,,,sealed,\n\
         ,wanted,,Hades II,,PC,ww,,,digital,,,,\n\
         999,,,Celeste,,Switch,ww,,,digital,,,,\n\
         ,,,Tetris,,Game Boy,ww,,,physical,,,mint,\n\
         ,,abc,,,,,,,,,,,\n\
         ,,,Okami,someday,,,,,,,,,\n"
    );
    let (_, preview) = app.post_csv("/api/import/preview", &csv).await;
    assert_eq!(preview["summary"]["error_rows"], 7, "{preview}");
    assert_eq!(issues(&preview, 2), [("region".into(), "required".into())]);
    assert_eq!(
        issues(&preview, 3),
        [("condition".into(), "condition_digital".into())]
    );
    assert_eq!(
        issues(&preview, 4),
        [("status".into(), "wanted_unsupported".into())]
    );
    assert_eq!(
        issues(&preview, 5),
        [("copy_id".into(), "unknown_copy".into())]
    );
    assert_eq!(
        issues(&preview, 6),
        [("condition".into(), "invalid_value".into())]
    );
    assert_eq!(
        issues(&preview, 7),
        [
            ("game_title".into(), "required".into()),
            ("igdb_id".into(), "invalid_number".into())
        ]
    );
    assert_eq!(
        issues(&preview, 8),
        [("play_status".into(), "invalid_value".into())]
    );

    let (status, body) = app.post_csv("/api/import", &csv).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body, json!({"code": "import_invalid"}));
    assert_eq!(app.scalar("SELECT COUNT(*) FROM games").await, 0);
}

#[tokio::test]
async fn conflicting_play_status_is_an_error() {
    let app = TestApp::new().await;
    let csv = "game_title,platform,region,format,play_status\n\
               Hollow Knight,Switch,ww,digital,playing\n\
               Hollow Knight,PC,ww,digital,completed\n\
               Hollow Knight,PS4,ww,digital,\n";
    let (_, preview) = app.post_csv("/api/import/preview", csv).await;
    assert_eq!(preview["summary"]["error_rows"], 1, "{preview}");
    assert_eq!(
        issues(&preview, 3),
        [("play_status".into(), "conflicting_play_status".into())]
    );
}

#[tokio::test]
async fn device_without_the_platform_is_a_warning() {
    let app = TestApp::new().await;
    app.json(
        "POST",
        "/api/devices",
        Some(json!({"name": "PS5", "kind": "console"})),
    )
    .await;
    let csv = "game_title,platform,region,format,device\nAstro Bot,PS5,eu,physical,PS5\n";
    let (_, preview) = app.post_csv("/api/import/preview", csv).await;
    assert_eq!(preview["summary"]["error_rows"], 0);
    assert_eq!(preview["summary"]["warning_rows"], 1);
    assert_eq!(
        issues(&preview, 2),
        [("device".into(), "device_platform_mismatch".into())]
    );

    let (status, _) = app.post_csv("/api/import", csv).await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn same_title_needs_an_igdb_id() {
    let app = TestApp::new().await;
    app.post_csv("/api/import", "igdb_id,game_title\n100,Doom\n200,Doom\n")
        .await;
    assert_eq!(app.scalar("SELECT COUNT(*) FROM games").await, 2);

    let (_, preview) = app
        .post_csv(
            "/api/import/preview",
            "game_title,platform,region,format\nDoom,PC,ww,digital\n",
        )
        .await;
    assert_eq!(
        issues(&preview, 2),
        [("game_title".into(), "ambiguous_game".into())]
    );

    let (_, preview) = app
        .post_csv(
            "/api/import/preview",
            "igdb_id,game_title,platform,region,format\n200,doom,PC,ww,digital\n",
        )
        .await;
    assert_eq!(preview["summary"]["error_rows"], 0);
    assert_eq!(preview["summary"]["new_games"], 0);
}

#[tokio::test]
async fn igdb_id_links_a_game_imported_without_one() {
    let app = TestApp::new().await;
    app.post_csv("/api/import", "game_title\nCeleste\n").await;
    let (_, result) = app
        .post_csv("/api/import", "igdb_id,game_title\n26226,celeste\n")
        .await;
    assert_eq!(result["summary"]["new_games"], 0, "{result}");
    assert_eq!(result["summary"]["updated_games"], 1);
    assert_eq!(app.scalar("SELECT igdb_id FROM games").await, 26226);
}

#[tokio::test]
async fn unreadable_files_are_reported() {
    let app = TestApp::new().await;
    let (status, preview) = app
        .post_csv("/api/import/preview", "title,platform\nZelda,Wii\n")
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        preview["file_issue"],
        json!({"code": "missing_column", "column": "game_title"})
    );
    let (status, _) = app.post_csv("/api/import", "").await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_writes_all_succeed() {
    let app = std::sync::Arc::new(TestApp::new().await);
    let tasks: Vec<_> = (0..12)
        .map(|i| {
            let app = app.clone();
            tokio::spawn(async move {
                let csv = format!(
                    "game_title,platform,region,format,store\nGame {i},Switch,eu,physical,Store {i}\n"
                );
                app.post_csv("/api/import", &csv).await
            })
        })
        .collect();
    for task in tasks {
        let (status, body) = task.await.unwrap();
        assert_eq!(status, StatusCode::OK, "{body}");
    }
    assert_eq!(app.scalar("SELECT COUNT(*) FROM games").await, 12);
}
