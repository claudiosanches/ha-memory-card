//! CSV import and export.

mod common;

use axum::http::StatusCode;
use common::TestApp;
use serde_json::{Value, json};

/// The columns most rows here use; imports read columns by name.
const HEADER: &str = "copy_id,status,igdb_id,game_title,play_status,platform,region,release_title,product_code,format,device,store,condition,notes";
/// Every column, in export order.
const COLUMNS: &str = "copy_id,status,igdb_id,game_title,play_status,labels,platform,region,release_title,product_code,format,edition,device,store,condition,purchased_on,price,currency,estimated_value,estimated_currency,barcode,notes,bundle_copy_id";

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
async fn export_follows_the_library_filters() {
    let app = TestApp::new().await;
    let csv = format!(
        "{HEADER}\n\
         ,,,Mario Kart 8 Deluxe,playing,Switch,eu,,,digital,,,,\n\
         ,,,Mario Kart 8 Deluxe,playing,Wii U,eu,,,physical,,,,\n\
         ,,,Zelda: Tears of the Kingdom,paused,,,,,,,,,\n"
    );
    let (status, result) = app.post_csv("/api/import", &csv).await;
    assert_eq!(status, StatusCode::OK, "{result}");
    let lines = |body: Vec<u8>| {
        String::from_utf8(body)
            .unwrap()
            .lines()
            .skip(1)
            .map(|line| line.split(',').take(7).collect::<Vec<_>>().join(","))
            .collect::<Vec<_>>()
    };

    let (status, body) = app.get("/api/export.csv").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(lines(body).len(), 3, "the plain URL exports everything");

    let (_, body) = app.get("/api/export.csv?format=physical").await;
    let rows = lines(body);
    assert_eq!(rows.len(), 1, "{rows:?}");
    assert!(
        rows[0].ends_with("Mario Kart 8 Deluxe,playing,,Wii U"),
        "{rows:?}"
    );

    let (_, body) = app.get("/api/export.csv?play_status=paused").await;
    assert_eq!(lines(body), [",,,Zelda: Tears of the Kingdom,paused,,"]);

    let (_, body) = app
        .get("/api/export.csv?ownership=owned&play_status=paused")
        .await;
    assert!(
        lines(body).is_empty(),
        "owned copies only leaves out games without copies"
    );

    let (status, _) = app.get("/api/export.csv?region=mars").await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn wanted_copies_import_and_export() {
    let app = TestApp::new().await;
    let csv = format!("{HEADER}\n,wishlist,,Hades II,,PC,ww,,,digital,,,,\n");
    let (status, result) = app.post_csv("/api/import", &csv).await;
    assert_eq!(status, StatusCode::OK, "{result}");
    assert_eq!(
        app.scalar("SELECT COUNT(*) FROM copies WHERE status = 'wanted'")
            .await,
        1
    );
    assert!(export(&app).await.contains(",wanted,,Hades II,"));
}

#[tokio::test]
async fn template_is_the_header_row() {
    let app = TestApp::new().await;
    let (status, bytes) = app.get("/api/import/template.csv").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(bytes, format!("\u{feff}{COLUMNS}\r\n").into_bytes());
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
    assert_eq!(result["unlinked_games"], 1, "Zelda has no IGDB id");
    assert_eq!(
        app.scalar(
            "SELECT COUNT(*) FROM games g JOIN copies c ON 1 WHERE g.created_by IS NULL OR c.created_by IS NULL"
        )
        .await,
        0,
        "the importer added them"
    );

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
    assert_eq!(lines[0], COLUMNS);
    assert_eq!(lines.len(), 4, "{exported}");
    // Same platform: rows sort by region, so eu comes before na.
    assert!(
        lines[1].ends_with(",owned,1234,Super Mario Odyssey,completed,,Nintendo Switch,eu,Super Mario Odyssey (PAL),,digital,,,,,,,,,,,,"),
        "{}",
        lines[1]
    );
    assert!(
        lines[2].ends_with(",owned,1234,Super Mario Odyssey,completed,,Nintendo Switch,na,,HAC-AAACA,physical,,Switch OLED,Amazon,cib,,,,,,,\"Gift, 2017\","),
        "{}",
        lines[2]
    );
    assert_eq!(
        lines[3],
        ",,,Zelda: Breath of the Wild,mastered,,,,,,,,,,,,,,,,,,"
    );

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
    let edited = exported.replace(
        ",physical,,,,,,,,,,,",
        ",physical,,,Amazon,sealed,,,,,,,Still wrapped",
    );

    let (_, result) = app.post_csv("/api/import", &edited).await;
    assert_eq!(result["summary"]["updated_copies"], 1, "{result}");
    assert_eq!(result["summary"]["new_copies"], 0);
    assert_eq!(
        app.scalar("SELECT COUNT(*) FROM copies WHERE source = 'csv'")
            .await,
        1
    );

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
    assert_eq!(preview["summary"]["error_rows"], 6, "{preview}");
    assert_eq!(issues(&preview, 2), [("region".into(), "required".into())]);
    assert_eq!(
        issues(&preview, 3),
        [("condition".into(), "condition_digital".into())]
    );
    assert!(issues(&preview, 4).is_empty(), "wanted copies import");
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
        .post_csv("/api/import/preview", "rating,platform\nZelda,Wii\n")
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

#[tokio::test]
async fn purchase_columns_import_and_export() {
    let app = TestApp::new().await;
    let csv = format!(
        "{COLUMNS}\n\
         ,,,Hades,,,PC,ww,,,digital,Deluxe,,Steam,,2024-09-17,\"R$ 1.299,90\",brl,,,,\n\
         ,,,Tetris,,,Switch,eu,,,physical,,,,,,5980,JPY,,,4902370536485,\n\
         ,wanted,,Celeste,,,Switch,eu,,,physical,Collector's,,,,,59.9,USD,,,,\n"
    );
    let (status, result) = app.post_csv("/api/import", &csv).await;
    assert_eq!(status, StatusCode::OK, "{result}");

    let exported = export(&app).await;
    let lines: Vec<&str> = exported.trim_start_matches('\u{feff}').lines().collect();
    assert!(
        lines[2].ends_with(",digital,Deluxe,,Steam,,2024-09-17,1299.90,BRL,,,,,"),
        "{}",
        lines[2]
    );
    assert!(
        lines[3].ends_with(",physical,,,,,,5980,JPY,,,4902370536485,,"),
        "{}",
        lines[3]
    );
    assert!(
        lines[1].ends_with(",Collector's,,,,,59.90,USD,,,,,"),
        "{}",
        lines[1]
    );

    let (_, preview) = app.post_csv("/api/import/preview", &exported).await;
    assert_eq!(preview["summary"]["unchanged_copies"], 3, "{preview}");

    let bad = format!(
        "{COLUMNS}\n\
         ,,,A,,,PC,ww,,,digital,,,,,yesterday,,,,,,\n\
         ,,,B,,,PC,ww,,,digital,,,,,,10,,,,,\n\
         ,,,C,,,PC,ww,,,digital,,,,,,ten,USD,,,,\n\
         ,,,D,,,PC,ww,,,digital,,,,,,10,dollars,,,,\n\
         ,,,E,,,PC,ww,,,digital,,,,,,,,,,123,\n\
         ,,,F,,,PC,ww,,,digital,,,,,,,,25,,,\n\
         ,,,G,,,PC,ww,,,digital,,,,,,,,lots,EUR,,\n"
    );
    let (_, preview) = app.post_csv("/api/import/preview", &bad).await;
    assert_eq!(
        issues(&preview, 2),
        [("purchased_on".into(), "invalid_value".into())]
    );
    assert_eq!(
        issues(&preview, 3),
        [("currency".into(), "required".into())]
    );
    assert_eq!(
        issues(&preview, 4),
        [("price".into(), "invalid_value".into())]
    );
    assert_eq!(
        issues(&preview, 5),
        [("currency".into(), "invalid_value".into())]
    );
    assert_eq!(
        issues(&preview, 6),
        [("barcode".into(), "invalid_value".into())]
    );
    assert_eq!(
        issues(&preview, 7),
        [("estimated_currency".into(), "required".into())]
    );
    assert_eq!(
        issues(&preview, 8),
        [("estimated_value".into(), "invalid_value".into())]
    );
}

#[tokio::test]
async fn estimated_values_import_and_export() {
    let app = TestApp::new().await;
    let csv = "game_title,platform,region,format,price,currency,market value\n\
               Hades,PC,ww,digital,20,USD,35\n\
               Tetris,Switch,eu,physical,,BRL,\"1.000,50\"\n";
    let (status, result) = app.post_csv("/api/import", csv).await;
    assert_eq!(status, StatusCode::OK, "{result}");

    let exported = export(&app).await;
    let lines: Vec<&str> = exported.trim_start_matches('\u{feff}').lines().collect();
    assert!(
        lines[1].ends_with(",20.00,USD,35.00,USD,,,"),
        "{}",
        lines[1]
    );
    assert!(lines[2].ends_with(",,,1000.50,BRL,,,"), "{}", lines[2]);

    let edited = exported.replace("35.00,USD", "4000,JPY");
    let (_, preview) = app.post_csv("/api/import/preview", &edited).await;
    assert_eq!(preview["summary"]["updated_copies"], 1, "{preview}");
    app.post_csv("/api/import", &edited).await;
    assert!(export(&app).await.contains(",20.00,USD,4000,JPY,,"));
}

#[tokio::test]
async fn preview_says_how_each_header_was_read() {
    let app = TestApp::new().await;
    let (_, preview) = app
        .post_csv("/api/import/preview", "Name;Console;Stars\nZelda;Wii;5\n")
        .await;
    assert_eq!(preview["delimiter"], ";");
    assert_eq!(
        preview["headers"],
        json!([
            {"header": "Name", "column": "game_title"},
            {"header": "Console", "column": "platform"},
            {"header": "Stars", "column": null},
        ])
    );
    assert_eq!(preview["summary"]["game_only_rows"], 0);
    assert_eq!(preview["rows"][0]["issues"][0]["column"], "region");

    let (_, preview) = app
        .post_csv("/api/import/preview", "Stars,Console\n5,Wii\n")
        .await;
    assert_eq!(preview["file_issue"]["code"], "missing_column");
    assert_eq!(
        preview["headers"][1]["column"], "platform",
        "headers come back to fix the file"
    );
}

#[tokio::test]
async fn formulas_export_as_text_and_import_back_unchanged() {
    let app = TestApp::new().await;
    let csv = format!(
        "{HEADER}\n,,,\"=HYPERLINK(\"\"http://x\"\")\",,Switch,eu,,,physical,,,,+1 spare\n"
    );
    let (status, result) = app.post_csv("/api/import", &csv).await;
    assert_eq!(status, StatusCode::OK, "{result}");

    let exported = export(&app).await;
    assert!(
        exported.contains(",\"'=HYPERLINK(\"\"http://x\"\")\","),
        "{exported}"
    );
    assert!(exported.contains(",'+1 spare,"), "{exported}");

    let (_, result) = app.post_csv("/api/import", &exported).await;
    assert_eq!(result["summary"]["updated_copies"], 0, "{result}");
    assert_eq!(result["summary"]["new_copies"], 0, "{result}");
    assert_eq!(result["summary"]["new_games"], 0, "{result}");
    let (title, notes): (String, String) = sqlx::query_as(
        "SELECT g.title, c.notes FROM games g JOIN releases r ON r.game_id = g.id JOIN copies c ON c.release_id = r.id",
    )
    .fetch_one(&app.pool)
    .await
    .unwrap();
    assert_eq!(
        (title.as_str(), notes.as_str()),
        ("=HYPERLINK(\"http://x\")", "+1 spare")
    );
}

#[tokio::test]
async fn many_games_that_share_a_title_plan_quickly() {
    let app = TestApp::new().await;
    let csv = |from: usize| -> String {
        let rows: String = (from..from + 1500)
            .map(|igdb_id| format!("{igdb_id},Tetris,Switch,eu,digital\n"))
            .collect();
        format!("igdb_id,game_title,platform,region,format\n{rows}")
    };
    let (status, result) = app.post_csv("/api/import", &csv(1)).await;
    assert_eq!(status, StatusCode::OK, "{result}");

    let started = std::time::Instant::now();
    let (status, result) = app.post_csv("/api/import", &csv(100_001)).await;
    assert_eq!(status, StatusCode::OK, "{result}");
    assert!(started.elapsed().as_secs() < 10, "{:?}", started.elapsed());
    assert_eq!(app.scalar("SELECT COUNT(*) FROM games").await, 3000);
}

#[tokio::test]
async fn formulas_after_a_semicolon_or_line_break_export_as_text() {
    let app = TestApp::new().await;
    let csv = format!("{HEADER}\n,,,Zelda;=1+1,,Switch,eu,,,physical,,,,\"Box\n- manual\"\n");
    let (status, result) = app.post_csv("/api/import", &csv).await;
    assert_eq!(status, StatusCode::OK, "{result}");

    let exported = export(&app).await;
    assert!(exported.contains(",Zelda;'=1+1,"), "{exported}");
    assert!(exported.contains(",\"Box\n'- manual\","), "{exported}");

    let (_, result) = app.post_csv("/api/import", &exported).await;
    assert_eq!(result["summary"]["updated_copies"], 0, "{result}");
    assert_eq!(result["summary"]["new_games"], 0, "{result}");
    let (title, notes): (String, String) = sqlx::query_as(
        "SELECT g.title, c.notes FROM games g JOIN releases r ON r.game_id = g.id JOIN copies c ON c.release_id = r.id",
    )
    .fetch_one(&app.pool)
    .await
    .unwrap();
    assert_eq!(
        (title.as_str(), notes.as_str()),
        ("Zelda;=1+1", "Box\n- manual")
    );
}

#[tokio::test]
async fn a_game_gets_at_most_50_labels_across_rows_and_imports() {
    let app = TestApp::new().await;
    let labels =
        |range: std::ops::Range<u32>| range.map(|n| format!("l{n}")).collect::<Vec<_>>().join(";");
    let csv = format!(
        "game_title,labels\nOkami,{}\nOkami,{}\nOkami,{}\n",
        labels(0..30),
        labels(20..45),
        labels(45..55)
    );
    let (_, preview) = app.post_csv("/api/import/preview", &csv).await;
    assert_eq!(preview["summary"]["error_rows"], 1, "{preview}");
    assert_eq!(
        issues(&preview, 4),
        [("labels".into(), "too_many_labels".into())]
    );

    let csv = format!("game_title,labels\nOkami,{}\n", labels(0..45));
    let (status, _) = app.post_csv("/api/import", &csv).await;
    assert_eq!(status, StatusCode::OK);
    let csv = format!("game_title,labels\nOkami,{}\n", labels(40..46));
    let (_, preview) = app.post_csv("/api/import/preview", &csv).await;
    assert_eq!(preview["summary"]["error_rows"], 0, "{preview}");
    let csv = format!("game_title,labels\nOkami,{}\n", labels(40..51));
    let (_, preview) = app.post_csv("/api/import/preview", &csv).await;
    assert_eq!(
        issues(&preview, 2),
        [("labels".into(), "too_many_labels".into())],
        "counts the labels the game has"
    );
}
