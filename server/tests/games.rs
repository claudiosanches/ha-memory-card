//! Games, releases, copies and the library filters.

mod common;

use axum::http::StatusCode;
use common::TestApp;
use memory_card::db::library::{self, LibraryFilter};
use serde_json::{Value, json};

async fn new_game(app: &TestApp, title: &str) -> i64 {
    let (status, game) = app
        .json("POST", "/api/games", Some(json!({"title": title})))
        .await;
    assert_eq!(status, StatusCode::CREATED, "{game}");
    game["id"].as_i64().unwrap()
}

async fn add_copy(app: &TestApp, game: i64, body: Value) -> Value {
    let (status, detail) = app
        .json("POST", &format!("/api/games/{game}/copies"), Some(body))
        .await;
    assert_eq!(status, StatusCode::CREATED, "{detail}");
    detail
}

#[tokio::test]
async fn copies_find_or_create_their_release() {
    let app = TestApp::new().await;
    let switch = app.platform_id("switch").await;
    let game = new_game(&app, "Metroid Dread").await;

    add_copy(
        &app,
        game,
        json!({"platform_id": switch, "region": "eu", "format": "physical", "condition": "cib"}),
    )
    .await;
    add_copy(
        &app,
        game,
        json!({"platform_id": switch, "region": "eu", "format": "digital"}),
    )
    .await;
    let detail = add_copy(
        &app,
        game,
        json!({"platform_id": switch, "region": "jp", "format": "physical"}),
    )
    .await;

    let releases = detail["releases"].as_array().unwrap();
    assert_eq!(releases.len(), 2);
    assert_eq!(releases[0]["region"], "eu");
    assert_eq!(releases[0]["copies"].as_array().unwrap().len(), 2);
    assert_eq!(releases[0]["copies"][0]["condition"], "cib");
    assert_eq!(releases[1]["region"], "jp");
}

#[tokio::test]
async fn copy_validation() {
    let app = TestApp::new().await;
    let switch = app.platform_id("switch").await;
    let game = new_game(&app, "Metroid Dread").await;

    for (body, code, field) in [
        (
            json!({"platform_id": switch, "region": "eu", "format": "digital", "condition": "sealed"}),
            "condition_digital",
            "condition",
        ),
        (
            json!({"platform_id": 9999, "region": "eu", "format": "digital"}),
            "unknown_platform",
            "platform_id",
        ),
        (
            json!({"platform_id": switch, "region": "eu", "format": "digital", "device_id": 9999}),
            "unknown_device",
            "device_id",
        ),
        (
            json!({"platform_id": switch, "region": "eu", "format": "digital", "store_id": 9999}),
            "unknown_store",
            "store_id",
        ),
    ] {
        let (status, body) = app
            .json("POST", &format!("/api/games/{game}/copies"), Some(body))
            .await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(body, json!({"code": code, "field": field}));
    }
    let (status, _) = app
        .json(
            "POST",
            "/api/games/9999/copies",
            Some(json!({"platform_id": switch, "region": "eu", "format": "digital"})),
        )
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn moving_and_deleting_copies() {
    let app = TestApp::new().await;
    let ps4 = app.platform_id("ps4").await;
    let ps5 = app.platform_id("ps5").await;
    let game = new_game(&app, "Ghost of Tsushima").await;
    let detail = add_copy(
        &app,
        game,
        json!({"platform_id": ps4, "region": "eu", "format": "physical"}),
    )
    .await;
    let copy = detail["releases"][0]["copies"][0]["id"].as_i64().unwrap();

    let (status, detail) = app
        .json(
            "PUT",
            &format!("/api/copies/{copy}"),
            Some(json!({"platform_id": ps5, "region": "eu", "format": "digital", "notes": "Upgrade"})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{detail}");
    let releases = detail["releases"].as_array().unwrap();
    assert_eq!(releases.len(), 2, "the PS4 release stays, empty");
    let ps5_release = releases.iter().find(|r| r["platform_id"] == ps5).unwrap();
    assert_eq!(ps5_release["copies"][0]["notes"], "Upgrade");

    let (status, detail) = app
        .json("DELETE", &format!("/api/copies/{copy}"), None)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        detail["releases"]
            .as_array()
            .unwrap()
            .iter()
            .all(|r| r["copies"] == json!([]))
    );

    let release = releases[0]["id"].as_i64().unwrap();
    let (status, detail) = app
        .json("DELETE", &format!("/api/releases/{release}"), None)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(detail["releases"].as_array().unwrap().len(), 1);

    let (status, _) = app
        .json("DELETE", &format!("/api/games/{game}"), None)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert_eq!(app.scalar("SELECT COUNT(*) FROM releases").await, 0);
}

#[tokio::test]
async fn game_status_players_and_modes() {
    let app = TestApp::new().await;
    let game = new_game(&app, "Mario Kart 8 Deluxe").await;
    let modes: Vec<i64> = sqlx::query_scalar(
        "SELECT id FROM game_modes WHERE slug IN ('multiplayer', 'split-screen')",
    )
    .fetch_all(&app.pool)
    .await
    .unwrap();

    let body = json!({
        "title": "Mario Kart 8 Deluxe",
        "play_status": "playing",
        "local_max_players": 4,
        "online_max_players": 12,
        "game_mode_ids": modes,
    });
    let (status, detail) = app
        .json("PUT", &format!("/api/games/{game}"), Some(body.clone()))
        .await;
    assert_eq!(status, StatusCode::OK, "{detail}");
    assert_eq!(detail["play_status"], "playing");
    assert_eq!(detail["game_mode_ids"], json!(modes));
    assert_eq!(detail["local_max_players"], 4);

    for (patch, code, field) in [
        (
            json!({"local_max_players": 0}),
            "out_of_range",
            "local_max_players",
        ),
        (
            json!({"game_mode_ids": [9999]}),
            "unknown_id",
            "game_mode_ids",
        ),
        (json!({"title": " "}), "required", "title"),
    ] {
        let mut invalid = body.clone();
        invalid
            .as_object_mut()
            .unwrap()
            .extend(patch.as_object().unwrap().clone());
        let (status, error) = app
            .json("PUT", &format!("/api/games/{game}"), Some(invalid))
            .await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(error, json!({"code": code, "field": field}));
    }
}

#[tokio::test]
async fn sessions_log_play_and_the_latest_is_last_played() {
    let app = TestApp::new().await;
    let switch = app.platform_id("switch").await;
    let game = new_game(&app, "Mario Kart 8 Deluxe").await;
    let other = new_game(&app, "Other").await;
    let copy = add_copy(
        &app,
        game,
        json!({"platform_id": switch, "region": "ww", "format": "digital"}),
    )
    .await["releases"][0]["copies"][0]["id"]
        .as_i64()
        .unwrap();
    let other_copy = add_copy(
        &app,
        other,
        json!({"platform_id": switch, "region": "ww", "format": "digital"}),
    )
    .await["releases"][0]["copies"][0]["id"]
        .as_i64()
        .unwrap();

    let path = format!("/api/games/{game}/sessions");
    let (status, detail) = app
        .json(
            "POST",
            &path,
            Some(json!({"played_on": "2026-11-20", "copy_id": copy, "minutes": 45})),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "{detail}");
    let (_, detail) = app
        .json(
            "POST",
            &path,
            Some(json!({"played_on": "2026-11-18", "notes": " Cup run "})),
        )
        .await;
    let sessions = detail["sessions"].as_array().unwrap();
    assert_eq!(sessions.len(), 2);
    assert_eq!(sessions[0]["played_on"], "2026-11-20", "latest first");
    assert_eq!(sessions[0]["copy_id"], copy);
    assert_eq!(sessions[0]["minutes"], 45);
    assert_eq!(sessions[1]["notes"], "Cup run");
    assert_eq!(sessions[1]["created_by"]["id"], common::MANAGER);

    let (_, games) = app.json("GET", "/api/games?sort=played", None).await;
    assert_eq!(games[0]["id"], game);
    assert_eq!(games[0]["last_played_at"], "2026-11-20");

    let older = sessions[1]["id"].as_i64().unwrap();
    let (status, detail) = app
        .json(
            "PUT",
            &format!("/api/sessions/{older}"),
            Some(json!({"played_on": "2026-11-21", "copy_id": copy})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{detail}");
    assert_eq!(detail["sessions"][0]["id"], older);
    assert_eq!(detail["sessions"][0]["notes"], Value::Null);

    for (body, code, field) in [
        (
            json!({"played_on": "2026-11-21", "copy_id": other_copy}),
            "invalid_copy",
            "copy_id",
        ),
        (
            json!({"played_on": "20/11/2026"}),
            "invalid_date",
            "played_on",
        ),
        (json!({"played_on": " "}), "required", "played_on"),
        (
            json!({"played_on": "2026-11-21", "minutes": 0}),
            "out_of_range",
            "minutes",
        ),
    ] {
        let (status, error) = app.json("POST", &path, Some(body)).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(error, json!({"code": code, "field": field}));
    }
    let (status, _) = app
        .json(
            "POST",
            "/api/games/9999/sessions",
            Some(json!({"played_on": "2026-11-21"})),
        )
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let (status, detail) = app
        .json("DELETE", &format!("/api/sessions/{older}"), None)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(detail["sessions"].as_array().unwrap().len(), 1);
    let (status, _) = app
        .json("DELETE", &format!("/api/sessions/{older}"), None)
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let (_, detail) = app
        .json("DELETE", &format!("/api/copies/{copy}"), None)
        .await;
    assert_eq!(
        detail["sessions"][0]["copy_id"],
        Value::Null,
        "the session outlives its copy"
    );
}

#[tokio::test]
async fn release_details_and_conflicts() {
    let app = TestApp::new().await;
    let wii = app.platform_id("wii").await;
    let game = new_game(&app, "Mario Kart Wii").await;
    add_copy(
        &app,
        game,
        json!({"platform_id": wii, "region": "eu", "format": "physical"}),
    )
    .await;
    let detail = add_copy(
        &app,
        game,
        json!({"platform_id": wii, "region": "eu", "product_code": "RMCP01", "format": "physical"}),
    )
    .await;
    let first = detail["releases"][0]["id"].as_i64().unwrap();

    let (status, detail) = app
        .json(
            "PUT",
            &format!("/api/releases/{first}"),
            Some(json!({"title": "Mario Kart Wii (bundle)", "local_max_players": 4, "local_coop": false})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{detail}");
    assert_eq!(detail["releases"][0]["title"], "Mario Kart Wii (bundle)");
    assert_eq!(detail["releases"][0]["local_coop"], false);

    let (status, body) = app
        .json(
            "PUT",
            &format!("/api/releases/{first}"),
            Some(json!({"product_code": "RMCP01"})),
        )
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["code"], "release_exists");
}

// ---- library ----

const LIBRARY: &str = "game_title,play_status,platform,region,format,device,store\n\
    Mario Kart 8 Deluxe,playing,Switch,eu,digital,Switch OLED,Nintendo eShop\n\
    Mario Kart 8 Deluxe,,Wii U,eu,physical,Wii U,Amazon\n\
    The Last of Us Part II,completed,PS4,na,physical,PS5,Amazon\n\
    Astro Bot,backlog,PS5,eu,digital,PS5,PlayStation Store\n\
    Splatoon,backlog,Wii U,jp,physical,Wii U,\n\
    Zelda: Tears of the Kingdom,paused,,,,,\n";

async fn library_app() -> TestApp {
    let app = TestApp::new().await;
    let (status, result) = app.post_csv("/api/import", LIBRARY).await;
    assert_eq!(status, StatusCode::OK, "{result}");
    // Local players: Mario Kart 4 on the game, 2 on the Wii U release; Splatoon online only.
    sqlx::query("UPDATE games SET local_max_players = 4, online_max_players = 12 WHERE title = 'Mario Kart 8 Deluxe'")
        .execute(&app.pool)
        .await
        .unwrap();
    sqlx::query(
        "UPDATE releases SET local_max_players = 2 WHERE platform_id = (SELECT id FROM platforms WHERE slug = 'wii-u')
         AND game_id = (SELECT id FROM games WHERE title = 'Mario Kart 8 Deluxe')",
    )
    .execute(&app.pool)
    .await
    .unwrap();
    sqlx::query("UPDATE games SET online_max_players = 8 WHERE title = 'Splatoon'")
        .execute(&app.pool)
        .await
        .unwrap();
    app
}

async fn titles(app: &TestApp, query: &str) -> Vec<String> {
    let (status, list) = app.json("GET", &format!("/api/games?{query}"), None).await;
    assert_eq!(status, StatusCode::OK, "{query}: {list}");
    list.as_array()
        .unwrap()
        .iter()
        .map(|g| g["title"].as_str().unwrap().to_owned())
        .collect()
}

async fn ids(app: &TestApp, sql: &'static str) -> String {
    let ids: Vec<i64> = sqlx::query_scalar(sql).fetch_all(&app.pool).await.unwrap();
    ids.iter().map(i64::to_string).collect::<Vec<_>>().join(",")
}

#[tokio::test]
async fn library_lists_owned_games_by_default() {
    let app = library_app().await;
    assert_eq!(
        titles(&app, "").await,
        [
            "Astro Bot",
            "The Last of Us Part II",
            "Mario Kart 8 Deluxe",
            "Splatoon"
        ]
    );
    assert_eq!(titles(&app, "ownership=any").await.len(), 5);

    let (_, list) = app.json("GET", "/api/games?q=mario", None).await;
    assert_eq!(list[0]["owned_copies"], 2);
    let platforms: Vec<i64> = list[0]["platforms"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["copies"].as_i64().unwrap())
        .collect();
    assert_eq!(platforms, [1, 1]);
}

#[tokio::test]
async fn imported_titles_and_search_terms_keep_literal_punctuation() {
    let app = TestApp::new().await;
    let title = "Pilot's Log; 100%_Complete";
    let csv = format!("game_title\n{title}\nOther Game\n");
    let (status, result) = app.post_csv("/api/import", &csv).await;
    assert_eq!(status, StatusCode::OK, "{result}");
    let query = serde_urlencoded::to_string([("ownership", "any"), ("q", title)]).unwrap();
    assert_eq!(titles(&app, &query).await, [title]);
    assert_eq!(titles(&app, "ownership=any&q=%25_").await, [title]);
    assert_eq!(titles(&app, "ownership=any").await.len(), 2);
}

#[tokio::test]
async fn library_filters_by_igdb_link() {
    let app = library_app().await;
    sqlx::query("UPDATE games SET igdb_id = 7 WHERE title = 'Splatoon'")
        .execute(&app.pool)
        .await
        .unwrap();
    assert_eq!(titles(&app, "igdb=linked").await, ["Splatoon"]);
    let unlinked = titles(&app, "igdb=unlinked").await;
    assert!(!unlinked.is_empty());
    assert!(!unlinked.contains(&"Splatoon".to_owned()));
}

#[tokio::test]
async fn library_filters_combine_on_one_copy() {
    let app = library_app().await;
    let ps4 = ids(&app, "SELECT id FROM platforms WHERE slug = 'ps4'").await;
    let wii_u = ids(&app, "SELECT id FROM platforms WHERE slug = 'wii-u'").await;
    let switch = ids(&app, "SELECT id FROM platforms WHERE slug = 'switch'").await;
    let ps5_device = ids(&app, "SELECT id FROM devices WHERE name = 'PS5'").await;
    let amazon = ids(&app, "SELECT id FROM stores WHERE name = 'Amazon'").await;

    assert_eq!(
        titles(&app, &format!("platform={ps4},{wii_u}")).await,
        ["The Last of Us Part II", "Mario Kart 8 Deluxe", "Splatoon"]
    );
    assert_eq!(titles(&app, "region=jp").await, ["Splatoon"]);
    assert_eq!(
        titles(&app, &format!("device={ps5_device}")).await,
        ["Astro Bot", "The Last of Us Part II"]
    );
    assert_eq!(
        titles(&app, &format!("store={amazon}&format=physical")).await,
        ["The Last of Us Part II", "Mario Kart 8 Deluxe"]
    );
    assert_eq!(
        titles(&app, "play_status=backlog,paused&ownership=any").await,
        ["Astro Bot", "Splatoon", "Zelda: Tears of the Kingdom"]
    );

    sqlx::query("UPDATE games SET play_status = NULL WHERE title = 'Splatoon'")
        .execute(&app.pool)
        .await
        .unwrap();
    assert_eq!(titles(&app, "play_status=none").await, ["Splatoon"]);
    assert_eq!(
        titles(&app, "play_status=none,completed").await,
        ["The Last of Us Part II", "Splatoon"]
    );

    // Same copy: Mario Kart's Switch copy is digital, its Wii U copy physical.
    assert_eq!(
        titles(&app, &format!("platform={switch}&format=physical")).await,
        Vec::<String>::new()
    );
    assert_eq!(
        titles(
            &app,
            &format!("platform={wii_u}&format=physical&store={amazon}")
        )
        .await,
        ["Mario Kart 8 Deluxe"]
    );
}

#[tokio::test]
async fn copies_view_filters_each_copy() {
    let app = library_app().await;
    let wii_u = ids(&app, "SELECT id FROM platforms WHERE slug = 'wii-u'").await;
    let rows = |query: String| {
        let app = &app;
        async move {
            let (status, list) = app.json("GET", &format!("/api/copies?{query}"), None).await;
            assert_eq!(status, StatusCode::OK, "{query}: {list}");
            list.as_array()
                .unwrap()
                .iter()
                .map(|c| {
                    format!(
                        "{} {}",
                        c["title"].as_str().unwrap(),
                        c["region"].as_str().unwrap()
                    )
                })
                .collect::<Vec<_>>()
        }
    };

    assert_eq!(rows(String::new()).await.len(), 5);
    assert_eq!(
        rows(format!("platform={wii_u}&format=physical")).await,
        ["Mario Kart 8 Deluxe eu", "Splatoon jp"]
    );
    assert_eq!(
        rows("play_status=backlog".into()).await,
        ["Astro Bot eu", "Splatoon jp"]
    );

    sqlx::query(
        "UPDATE releases SET cover_path = 'release-1/cover.jpg'
         WHERE game_id = (SELECT id FROM games WHERE title = 'Splatoon')",
    )
    .execute(&app.pool)
    .await
    .unwrap();
    let (_, list) = app.json("GET", "/api/copies?region=jp", None).await;
    assert_eq!(list[0]["cover_path"], "release-1/cover.jpg");
    assert_eq!(list[0]["status"], "owned");
}

#[tokio::test]
async fn local_players_use_the_release_then_the_game() {
    let app = library_app().await;
    let wii_u = ids(&app, "SELECT id FROM platforms WHERE slug = 'wii-u'").await;
    let switch = ids(&app, "SELECT id FROM platforms WHERE slug = 'switch'").await;

    assert_eq!(
        titles(&app, "min_local_players=2").await,
        ["Mario Kart 8 Deluxe"]
    );
    assert_eq!(
        titles(&app, &format!("min_local_players=3&platform={switch}")).await,
        ["Mario Kart 8 Deluxe"]
    );
    assert_eq!(
        titles(&app, &format!("min_local_players=3&platform={wii_u}")).await,
        Vec::<String>::new()
    );
    // Online counts never count as local; 1 means no filter.
    assert!(
        !titles(&app, "min_local_players=5")
            .await
            .contains(&"Splatoon".to_owned())
    );
    assert_eq!(titles(&app, "min_local_players=1").await.len(), 4);
}

#[tokio::test]
async fn library_game_mode_filter_and_sorting() {
    let app = library_app().await;
    sqlx::query(
        "INSERT INTO game_game_modes (game_id, game_mode_id)
         SELECT g.id, m.id FROM games g, game_modes m WHERE g.title = 'Splatoon' AND m.slug = 'multiplayer'",
    )
    .execute(&app.pool)
    .await
    .unwrap();
    let multiplayer = ids(&app, "SELECT id FROM game_modes WHERE slug = 'multiplayer'").await;
    assert_eq!(
        titles(&app, &format!("game_mode={multiplayer}")).await,
        ["Splatoon"]
    );

    for (title, day) in [("Astro Bot", "2026-11-01"), ("Splatoon", "2026-11-15")] {
        sqlx::query(
            "INSERT INTO play_sessions (game_id, played_on) SELECT id, ? FROM games WHERE title = ?",
        )
        .bind(day)
        .bind(title)
        .execute(&app.pool)
        .await
        .unwrap();
    }
    assert_eq!(
        titles(&app, "sort=played").await[..2],
        ["Splatoon", "Astro Bot"]
    );

    let (status, body) = app.json("GET", "/api/games?region=mars", None).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["code"], "invalid_filter");
}

#[tokio::test]
async fn library_queries_use_indexes() {
    let app = TestApp::new().await;
    // ~2,500 games, 10,000 copies spread over four platforms and regions.
    sqlx::query(
        "WITH RECURSIVE n(i) AS (SELECT 1 UNION ALL SELECT i + 1 FROM n WHERE i < 2500)
         INSERT INTO games (title, sort_title) SELECT 'Game ' || i, 'game ' || i FROM n",
    )
    .execute(&app.pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO releases (game_id, platform_id, region)
         SELECT g.id, p.id, CASE p.id % 2 WHEN 0 THEN 'eu' ELSE 'na' END
         FROM games g, platforms p WHERE p.slug IN ('switch', 'ps5', 'wii-u', '3ds')",
    )
    .execute(&app.pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO copies (release_id, format) SELECT id, 'physical' FROM releases")
        .execute(&app.pool)
        .await
        .unwrap();
    sqlx::query("ANALYZE").execute(&app.pool).await.unwrap();

    let filters = [
        "",
        "platform=1,2",
        "region=eu",
        "format=digital",
        "device=1",
        "store=1",
        "genre=1",
        "game_mode=1",
        "play_status=backlog",
        "play_status=none",
        "min_local_players=2",
        "platform=1&format=physical&store=1",
    ];
    for query in filters {
        let filter: LibraryFilter = serde_urlencoded::from_str(query).unwrap();
        let qb = library::query(&filter);
        let sql = format!("EXPLAIN QUERY PLAN {}", qb.sql().as_str());
        // Bind placeholders get NULL; the plan does not depend on values.
        let placeholders = sql.matches('?').count();
        let mut explain = sqlx::query(sqlx::AssertSqlSafe(sql));
        for _ in 0..placeholders {
            explain = explain.bind(Option::<i64>::None);
        }
        let rows = explain.fetch_all(&app.pool).await.unwrap();
        let plan: Vec<String> = rows
            .iter()
            .map(|row| sqlx::Row::get::<String, _>(row, "detail"))
            .collect();
        for step in &plan {
            assert!(
                !(step.starts_with("SCAN c")
                    || step.starts_with("SCAN r")
                    || step.starts_with("SCAN gg")
                    || step.starts_with("SCAN gm")),
                "{query}: full scan in plan {plan:#?}"
            );
        }
    }
}

#[tokio::test]
async fn sets_only_the_play_status() {
    let app = TestApp::new().await;
    let (_, game) = app
        .json(
            "POST",
            "/api/games",
            Some(json!({"title": "Okami", "play_status": "backlog"})),
        )
        .await;
    let path = format!("/api/games/{}/play-status", game["id"]);

    let (status, detail) = app
        .json("PUT", &path, Some(json!({"play_status": "playing"})))
        .await;
    assert_eq!(status, StatusCode::OK, "{detail}");
    assert_eq!(detail["play_status"], "playing");
    assert_eq!(detail["title"], "Okami");

    let (_, detail) = app
        .json("PUT", &path, Some(json!({"play_status": null})))
        .await;
    assert_eq!(detail["play_status"], Value::Null);

    let (status, body) = app
        .json("PUT", &path, Some(json!({"play_status": "beaten"})))
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["code"], "unknown_play_status");
    let (status, _) = app
        .json(
            "PUT",
            "/api/games/999/play-status",
            Some(json!({"play_status": "playing"})),
        )
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn wanted_copies_make_a_wishlist() {
    let app = TestApp::new().await;
    let switch = app.platform_id("switch").await;
    let owned_game = new_game(&app, "Celeste").await;
    add_copy(
        &app,
        owned_game,
        json!({"platform_id": switch, "region": "ww", "format": "digital"}),
    )
    .await;
    let wished = new_game(&app, "Hades II").await;
    let detail = add_copy(
        &app,
        wished,
        json!({"status": "wanted", "platform_id": switch, "region": "ww", "format": "physical"}),
    )
    .await;
    let copy = detail["releases"][0]["copies"][0].clone();
    assert_eq!(copy["status"], "wanted");
    let copy_id = copy["id"].as_i64().unwrap();

    assert_eq!(titles(&app, "").await, ["Celeste"]);
    assert_eq!(titles(&app, "ownership=wanted").await, ["Hades II"]);
    let (_, list) = app.json("GET", "/api/games?ownership=wanted", None).await;
    assert_eq!(list[0]["owned_copies"], 0);
    assert_eq!(list[0]["wanted_copies"], 1);
    assert_eq!(
        list[0]["platforms"],
        json!([{"platform_id": switch, "copies": 1}])
    );
    let (_, status) = app.json("GET", "/api/status", None).await;
    assert_eq!(status["owned_copies"], 1);
    assert_eq!(status["games"], 1, "wishlist-only games are not counted");
    let (_, platforms) = app.json("GET", "/api/platforms", None).await;
    let on_switch = platforms
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["id"] == switch)
        .unwrap();
    assert_eq!(on_switch["game_count"], 2);
    assert_eq!(on_switch["owned_game_count"], 1);
    assert_eq!(on_switch["wanted_game_count"], 1);

    let (status, body) = app
        .json(
            "POST",
            &format!("/api/games/{wished}/sessions"),
            Some(json!({"played_on": "2026-11-21", "copy_id": copy_id})),
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["code"], "invalid_copy");

    // Leaving the status out keeps it; setting it to owned is "got it".
    let edit = json!({"platform_id": switch, "region": "ww", "format": "physical", "notes": "Limited Run"});
    let (status, detail) = app
        .json("PUT", &format!("/api/copies/{copy_id}"), Some(edit))
        .await;
    assert_eq!(status, StatusCode::OK, "{detail}");
    assert_eq!(detail["releases"][0]["copies"][0]["status"], "wanted");
    let got =
        json!({"status": "owned", "platform_id": switch, "region": "ww", "format": "physical"});
    app.json("PUT", &format!("/api/copies/{copy_id}"), Some(got))
        .await;
    assert_eq!(titles(&app, "").await, ["Celeste", "Hades II"]);
    assert!(titles(&app, "ownership=wanted").await.is_empty());
}

/// Sends JSON as a named Home Assistant user.
async fn json_as(
    app: &TestApp,
    user: (&str, &str),
    method: &str,
    path: &str,
    body: Value,
) -> (StatusCode, Value) {
    let request = axum::http::Request::builder()
        .method(method)
        .uri(path)
        .header("content-type", "application/json")
        .header("x-remote-user-id", user.0)
        .header("x-remote-user-display-name", user.1)
        .body(axum::body::Body::from(body.to_string()))
        .unwrap();
    let (status, bytes) = app.request(common::SUPERVISOR, request).await;
    (status, serde_json::from_slice(&bytes).unwrap())
}

#[tokio::test]
async fn games_and_copies_name_who_added_and_changed_them() {
    let app = TestApp::with_env(&[("MEMORY_CARD_MANAGERS", "sam,alex")]).await;
    let sam = ("sam", "Sam");
    let alex = ("alex", "Alex");
    let switch = app.platform_id("switch").await;

    let (_, game) = json_as(&app, sam, "POST", "/api/games", json!({"title": "Hades"})).await;
    let id = game["id"].as_i64().unwrap();
    assert_eq!(game["created_by"], json!({"id": "sam", "name": "Sam"}));
    assert_eq!(game["updated_by"]["id"], "sam");
    assert!(game["created_at"].as_str().unwrap().starts_with("20"));

    let (_, game) = json_as(
        &app,
        sam,
        "POST",
        &format!("/api/games/{id}/copies"),
        json!({"platform_id": switch, "region": "eu", "format": "digital"}),
    )
    .await;
    let copy = &game["releases"][0]["copies"][0];
    assert_eq!(copy["created_by"]["name"], "Sam");
    let copy_id = copy["id"].as_i64().unwrap();

    let (status, game) = json_as(
        &app,
        alex,
        "PUT",
        &format!("/api/copies/{copy_id}"),
        json!({"platform_id": switch, "region": "eu", "format": "digital", "notes": "Gift"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{game}");
    let copy = &game["releases"][0]["copies"][0];
    assert_eq!(copy["created_by"]["id"], "sam");
    assert_eq!(copy["updated_by"], json!({"id": "alex", "name": "Alex"}));
    assert_eq!(
        game["updated_by"]["id"], "sam",
        "a copy edit leaves the game alone"
    );

    let renamed = ("alex", "Alex B.");
    let (_, game) = json_as(
        &app,
        renamed,
        "PUT",
        &format!("/api/games/{id}/play-status"),
        json!({"play_status": "playing"}),
    )
    .await;
    assert_eq!(game["updated_by"]["name"], "Alex B.");
    assert_eq!(
        game["releases"][0]["copies"][0]["updated_by"]["name"], "Alex B.",
        "names follow Home Assistant"
    );
    assert_eq!(app.scalar("SELECT COUNT(*) FROM users").await, 2);
}

#[tokio::test]
async fn copies_record_edition_purchase_date_and_price() {
    let app = TestApp::new().await;
    let switch = app.platform_id("switch").await;
    let game = new_game(&app, "Tetris 99").await;
    let base = json!({"platform_id": switch, "region": "eu", "format": "physical"});
    let with = |extra: Value| {
        let mut body = base.clone();
        body.as_object_mut()
            .unwrap()
            .extend(extra.as_object().unwrap().clone());
        body
    };

    let detail = add_copy(
        &app,
        game,
        with(json!({"edition": " Big Box ", "purchased_on": "2026-11-01", "price": "1.299,90", "currency": "brl"})),
    )
    .await;
    let copy = &detail["releases"][0]["copies"][0];
    assert_eq!(copy["edition"], "Big Box");
    assert_eq!(copy["purchased_on"], "2026-11-01");
    assert_eq!(copy["price"], "1299.90");
    assert_eq!(copy["currency"], "BRL");
    let id = copy["id"].as_i64().unwrap();
    let (_, copies) = app.json("GET", "/api/copies", None).await;
    assert_eq!(copies[0]["edition"], "Big Box");
    assert_eq!(copies[0]["price"], "1299.90");

    let (status, detail) = app
        .json(
            "PUT",
            &format!("/api/copies/{id}"),
            Some(with(json!({"currency": "USD"}))),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let copy = &detail["releases"][0]["copies"][0];
    assert_eq!(copy["price"], Value::Null);
    assert_eq!(copy["currency"], Value::Null, "a currency alone is dropped");

    for (extra, code, field) in [
        (json!({"price": "10"}), "required", "currency"),
        (
            json!({"price": "10", "currency": "R$"}),
            "invalid_currency",
            "currency",
        ),
        (
            json!({"price": "ten", "currency": "USD"}),
            "invalid_price",
            "price",
        ),
        (
            json!({"purchased_on": "01/11/2026"}),
            "invalid_date",
            "purchased_on",
        ),
    ] {
        let (status, error) = app
            .json(
                "POST",
                &format!("/api/games/{game}/copies"),
                Some(with(extra)),
            )
            .await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(error, json!({"code": code, "field": field}));
    }
}

#[tokio::test]
async fn search_matches_other_names_and_ignores_accents() {
    let app = TestApp::new().await;
    let switch = app.platform_id("switch").await;
    let pokemon = new_game(&app, "Pokémon Legends: Arceus").await;
    let zelda = new_game(&app, "The Legend of Zelda: Breath of the Wild").await;
    new_game(&app, "Hades").await;
    add_copy(
        &app,
        pokemon,
        json!({"platform_id": switch, "region": "jp", "format": "physical"}),
    )
    .await;
    let release: i64 = app.scalar("SELECT id FROM releases").await;
    app.json(
        "PUT",
        &format!("/api/releases/{release}"),
        Some(json!({"title": "Pokémon LEGENDS アルセウス"})),
    )
    .await;

    let body = json!({"title": "The Legend of Zelda: Breath of the Wild", "names": [" BotW ", "botw", "", "Zelda BOTW"]});
    let (status, detail) = app
        .json("PUT", &format!("/api/games/{zelda}"), Some(body))
        .await;
    assert_eq!(status, StatusCode::OK, "{detail}");
    assert_eq!(
        detail["names"],
        json!([
            {"name": "BotW", "from_igdb": false, "comment": null},
            {"name": "Zelda BOTW", "from_igdb": false, "comment": null},
        ])
    );
    let keep = json!({"title": "The Legend of Zelda: Breath of the Wild"});
    let (_, detail) = app
        .json("PUT", &format!("/api/games/{zelda}"), Some(keep))
        .await;
    assert_eq!(
        detail["names"].as_array().unwrap().len(),
        2,
        "left out, names stay"
    );

    for (q, expected) in [
        ("pokemon", vec!["Pokémon Legends: Arceus"]),
        ("アルセウス", vec!["Pokémon Legends: Arceus"]),
        ("botw", vec!["The Legend of Zelda: Breath of the Wild"]),
        (
            "LEGEND",
            vec![
                "The Legend of Zelda: Breath of the Wild",
                "Pokémon Legends: Arceus",
            ],
        ),
        ("ｈａｄｅｓ", vec!["Hades"]),
        ("mario", vec![]),
    ] {
        let query = serde_urlencoded::to_string([("ownership", "any"), ("q", q)]).unwrap();
        assert_eq!(titles(&app, &query).await, expected, "q={q}");
    }
    let query = serde_urlencoded::to_string([("q", "pokemon")]).unwrap();
    let (_, copies) = app.json("GET", &format!("/api/copies?{query}"), None).await;
    assert_eq!(copies.as_array().unwrap().len(), 1);
}
