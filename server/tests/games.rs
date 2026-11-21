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
async fn game_status_last_played_and_modes() {
    let app = TestApp::new().await;
    let switch = app.platform_id("switch").await;
    let game = new_game(&app, "Mario Kart 8 Deluxe").await;
    let other = new_game(&app, "Other").await;
    let detail = add_copy(
        &app,
        game,
        json!({"platform_id": switch, "region": "ww", "format": "digital"}),
    )
    .await;
    let copy = detail["releases"][0]["copies"][0]["id"].as_i64().unwrap();
    let other_copy = add_copy(
        &app,
        other,
        json!({"platform_id": switch, "region": "ww", "format": "digital"}),
    )
    .await["releases"][0]["copies"][0]["id"]
        .as_i64()
        .unwrap();
    let modes: Vec<i64> = sqlx::query_scalar(
        "SELECT id FROM game_modes WHERE slug IN ('multiplayer', 'split-screen')",
    )
    .fetch_all(&app.pool)
    .await
    .unwrap();

    let body = json!({
        "title": "Mario Kart 8 Deluxe",
        "play_status": "playing",
        "last_played_at": "2026-11-20",
        "last_played_copy_id": copy,
        "local_max_players": 4,
        "online_max_players": 12,
        "game_mode_ids": modes,
    });
    let (status, detail) = app
        .json("PUT", &format!("/api/games/{game}"), Some(body.clone()))
        .await;
    assert_eq!(status, StatusCode::OK, "{detail}");
    assert_eq!(detail["play_status"], "playing");
    assert_eq!(detail["last_played_copy_id"], copy);
    assert_eq!(detail["game_mode_ids"], json!(modes));
    assert_eq!(detail["local_max_players"], 4);

    for (patch, code, field) in [
        (
            json!({"last_played_copy_id": other_copy}),
            "invalid_copy",
            "last_played_copy_id",
        ),
        (
            json!({"last_played_at": "20/11/2026"}),
            "invalid_date",
            "last_played_at",
        ),
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

    sqlx::query("UPDATE games SET last_played_at = '2026-11-01' WHERE title = 'Astro Bot'")
        .execute(&app.pool)
        .await
        .unwrap();
    sqlx::query("UPDATE games SET last_played_at = '2026-11-15' WHERE title = 'Splatoon'")
        .execute(&app.pool)
        .await
        .unwrap();
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
