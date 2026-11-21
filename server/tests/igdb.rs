//! IGDB linking and sync against a local stand-in for Twitch and IGDB.

mod common;

use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{get, post};
use common::TestApp;
use memory_card::config::IgdbCredentials;
use memory_card::igdb::{Endpoints, Igdb};
use serde_json::{Value, json};

fn odyssey() -> Value {
    json!({
        "id": 1, "name": "Super Mario Odyssey", "summary": "Cappy.",
        "first_release_date": 1509062400, "platforms": [130],
        "genres": [{"id": 8, "name": "Platform"}],
        "game_modes": [{"id": 1, "name": "Single player", "slug": "single-player"},
                       {"id": 3, "name": "Co-operative", "slug": "co-operative"}],
        "involved_companies": [
            {"company": {"name": "Nintendo EPD"}, "developer": true, "publisher": false},
            {"company": {"name": "Nintendo"}, "developer": false, "publisher": true}
        ],
        "cover": {"image_id": "co1mxf"},
        "multiplayer_modes": [{"platform": 130, "offlinecoopmax": 2, "offlinecoop": true}],
        "release_dates": [
            {"platform": 130, "date": 1509062400, "release_region": 8},
            {"platform": 130, "date": 1666656000, "release_region": 10}
        ],
        "game_localizations": [
            {"name": "スーパーマリオ オデッセイ", "region": 3, "cover": {"image_id": "co5wrq"}},
            {"region": 4, "cover": {"image_id": "co1mxf"}}
        ]
    })
}

async fn games(headers: HeaderMap, body: String) -> (StatusCode, axum::Json<Value>) {
    if headers.get("authorization").and_then(|v| v.to_str().ok()) != Some("Bearer token-1")
        || headers.get("client-id").and_then(|v| v.to_str().ok()) != Some("client")
    {
        return (StatusCode::UNAUTHORIZED, axum::Json(json!({})));
    }
    let result = if body.starts_with("search") {
        json!([{"id": 9, "name": "Super Mario Odyssey 64", "platforms": [{"name": "Nintendo 64"}]},
               {"id": 1, "name": "Super Mario Odyssey", "first_release_date": 1509062400,
                "platforms": [{"name": "Nintendo Switch", "abbreviation": "Switch"}],
                "total_rating_count": 900}])
    } else if body.contains(r#"name ~ "super mario odyssey""#) || body.contains("where id = (1)") {
        json!([odyssey()])
    } else if body.contains(r#"name ~ "Doom""#) {
        json!([{"id": 2, "name": "Doom", "platforms": [6]}, {"id": 3, "name": "DOOM", "platforms": [48]}])
    } else {
        json!([])
    };
    (StatusCode::OK, axum::Json(result))
}

async fn with_igdb() -> TestApp {
    let stub = Router::new()
        .route(
            "/token",
            post(|| async { axum::Json(json!({"access_token": "token-1", "expires_in": 3600})) }),
        )
        .route("/v4/games", post(games))
        .route(
            "/img/t_cover_big_2x/co1mxf.jpg",
            get(|| async { "jpeg bytes" }),
        )
        .route(
            "/img/t_cover_big_2x/co5wrq.jpg",
            get(|| async { "japanese jpeg" }),
        );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, stub).await.unwrap() });

    let mut app = TestApp::new().await;
    app.state.igdb = Some(Arc::new(Igdb::new(
        IgdbCredentials {
            client_id: "client".into(),
            client_secret: "secret".into(),
        },
        Endpoints {
            token_url: format!("{base}/token"),
            api_url: format!("{base}/v4"),
            image_url: format!("{base}/img"),
        },
    )));
    app
}

#[tokio::test]
async fn without_credentials_igdb_is_unavailable() {
    let app = TestApp::new().await;
    let (status, body) = app.json("GET", "/api/igdb/search?q=zelda", None).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body["code"], "igdb_not_configured");
    let (status, _) = app.json("POST", "/api/igdb/sync", None).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
}

#[tokio::test]
async fn search_returns_year_and_platforms() {
    let app = with_igdb().await;
    let (status, hits) = app.json("GET", "/api/igdb/search?q=odyssey", None).await;
    assert_eq!(status, StatusCode::OK, "{hits}");
    assert_eq!(
        hits,
        json!([
            {"igdb_id": 1, "name": "Super Mario Odyssey", "year": 2017, "platforms": ["Switch"]},
            {"igdb_id": 9, "name": "Super Mario Odyssey 64", "year": null, "platforms": ["Nintendo 64"]}
        ])
    );
}

#[tokio::test]
async fn linking_fills_metadata_and_keeps_hand_edits() {
    let app = with_igdb().await;
    let release = app.release("Odyssey", "switch").await;
    let game: i64 = sqlx::query_scalar("SELECT game_id FROM releases WHERE id = ?")
        .bind(release)
        .fetch_one(&app.pool)
        .await
        .unwrap();
    sqlx::query("UPDATE games SET summary = 'Mine.' WHERE id = ?")
        .bind(game)
        .execute(&app.pool)
        .await
        .unwrap();
    for region in ["jp", "br"] {
        sqlx::query(
            "INSERT INTO releases (game_id, platform_id, region)
             SELECT game_id, platform_id, ? FROM releases WHERE id = ?",
        )
        .bind(region)
        .bind(release)
        .execute(&app.pool)
        .await
        .unwrap();
    }

    let (status, detail) = app
        .json(
            "POST",
            &format!("/api/games/{game}/igdb"),
            Some(json!({"igdb_id": 1})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{detail}");
    assert_eq!(detail["igdb_id"], 1);
    assert_eq!(detail["title"], "Odyssey");
    assert_eq!(detail["summary"], "Mine.");
    assert_eq!(detail["first_release_date"], "2017-10-27");
    assert_eq!(detail["developers"], json!(["Nintendo EPD"]));
    assert_eq!(detail["publishers"], json!(["Nintendo"]));
    assert_eq!(detail["genre_ids"].as_array().unwrap().len(), 1);
    // Both modes are seeded with their IGDB ids, so none is added.
    assert_eq!(detail["game_mode_ids"].as_array().unwrap().len(), 2);
    assert_eq!(app.scalar("SELECT COUNT(*) FROM game_modes").await, 6);
    assert_eq!(app.scalar("SELECT COUNT(*) FROM genres").await, 1);
    assert_eq!(detail["releases"][0]["local_max_players"], 2);
    assert_eq!(detail["releases"][0]["local_coop"], true);
    let release = |region: &str| {
        detail["releases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["region"] == region)
            .unwrap()
            .clone()
    };
    // No North America date, so the worldwide one; Brazil has its own.
    assert_eq!(release("na")["release_date"], "2017-10-27");
    assert_eq!(release("br")["release_date"], "2022-10-25");
    assert_eq!(release("na")["title"], Value::Null);
    assert_eq!(release("jp")["title"], "スーパーマリオ オデッセイ");

    let cover = format!("game-{game}/co1mxf.jpg");
    assert_eq!(detail["cover_path"], cover.as_str());
    let (status, bytes) = app.get(&format!("/covers/{cover}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(bytes, b"jpeg bytes");

    // Japan has its own cover; Europe's is the generic one, so it is not repeated.
    let jp_cover = format!("release-{}/co5wrq.jpg", release("jp")["id"]);
    assert_eq!(release("jp")["cover_path"], jp_cover.as_str());
    assert_eq!(release("na")["cover_path"], Value::Null);
    let (status, bytes) = app.get(&format!("/covers/{jp_cover}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(bytes, b"japanese jpeg");

    // A second game cannot take the same IGDB id.
    let (_, other) = app
        .json("POST", "/api/games", Some(json!({"title": "Other"})))
        .await;
    let (status, body) = app
        .json(
            "POST",
            &format!("/api/games/{}/igdb", other["id"]),
            Some(json!({"igdb_id": 1})),
        )
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["code"], "igdb_linked");

    let (status, body) = app
        .json(
            "POST",
            &format!("/api/games/{game}/igdb"),
            Some(json!({"igdb_id": 99})),
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["code"], "igdb_not_found");

    let (status, _) = app
        .json("DELETE", &format!("/api/games/{game}"), None)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let covers: Vec<_> = std::fs::read_dir(app.config.covers_dir())
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    assert!(covers.is_empty(), "left behind: {covers:?}");
}

#[tokio::test]
async fn uploads_survive_a_refresh_and_removing_one_restores_igdb_covers() {
    let app = with_igdb().await;
    let release = app.release("Odyssey", "switch").await;
    sqlx::query("UPDATE releases SET region = 'jp' WHERE id = ?")
        .bind(release)
        .execute(&app.pool)
        .await
        .unwrap();
    let game: i64 = app.scalar("SELECT id FROM games").await;
    let path = format!("/api/games/{game}/igdb");
    let link = |overwrite: bool| {
        app.json(
            "POST",
            &path,
            Some(json!({"igdb_id": 1, "overwrite": overwrite})),
        )
    };
    link(false).await;

    let png = b"\x89PNG\r\n\x1a\nmine";
    let (_, uploaded) = app
        .put_bytes(&format!("/api/games/{game}/cover"), png)
        .await;
    app.put_bytes(&format!("/api/releases/{release}/cover"), png)
        .await;
    let (status, detail) = link(true).await;
    assert_eq!(status, StatusCode::OK, "{detail}");
    assert_eq!(detail["cover_path"], uploaded["cover_path"]);
    assert_eq!(detail["releases"][0]["cover_uploaded"], true);

    let (_, detail) = app
        .json("DELETE", &format!("/api/games/{game}/cover"), None)
        .await;
    assert_eq!(detail["cover_path"], format!("game-{game}/co1mxf.jpg"));
    assert_eq!(detail["cover_uploaded"], false);
    let (_, detail) = app
        .json("DELETE", &format!("/api/releases/{release}/cover"), None)
        .await;
    assert_eq!(
        detail["releases"][0]["cover_path"],
        format!("release-{release}/co5wrq.jpg")
    );
}

#[tokio::test]
async fn lists_titles_that_differ_from_igdb_until_resolved() {
    let app = with_igdb().await;
    app.release("Odyssey", "switch").await;
    let odyssey: i64 = app.scalar("SELECT id FROM games").await;
    let link = format!("/api/games/{odyssey}/igdb");
    app.json("POST", &link, Some(json!({"igdb_id": 1}))).await;
    app.release("Bomberman 93", "switch").await;
    sqlx::query("UPDATE games SET igdb_title = 'Bomberman ''93' WHERE title = 'Bomberman 93'")
        .execute(&app.pool)
        .await
        .unwrap();
    let bomberman: i64 = app
        .scalar("SELECT id FROM games WHERE title = 'Bomberman 93'")
        .await;

    let (status, list) = app.json("GET", "/api/igdb/titles", None).await;
    assert_eq!(status, StatusCode::OK, "{list}");
    let pairs: Vec<(&str, &str)> = list
        .as_array()
        .unwrap()
        .iter()
        .map(|d| {
            (
                d["title"].as_str().unwrap(),
                d["igdb_title"].as_str().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        pairs,
        [
            ("Bomberman 93", "Bomberman '93"),
            ("Odyssey", "Super Mario Odyssey")
        ]
    );

    let (status, left) = app
        .json(
            "POST",
            "/api/igdb/titles",
            Some(json!({"use_igdb": [bomberman], "keep": [odyssey]})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{left}");
    assert_eq!(left, json!([]));
    let (_, game) = app
        .json("GET", &format!("/api/games/{bomberman}"), None)
        .await;
    assert_eq!(game["title"], "Bomberman '93");
    let sort: String = sqlx::query_scalar("SELECT sort_title FROM games WHERE id = ?")
        .bind(bomberman)
        .fetch_one(&app.pool)
        .await
        .unwrap();
    assert_eq!(sort, "bomberman '93");

    // A refresh keeps a kept title off the list; a new IGDB name brings it back.
    app.json("POST", &link, Some(json!({"igdb_id": 1}))).await;
    let (_, list) = app.json("GET", "/api/igdb/titles", None).await;
    assert_eq!(list, json!([]));
    sqlx::query("UPDATE games SET igdb_title = 'Super Mario Odyssey Deluxe' WHERE id = ?")
        .bind(odyssey)
        .execute(&app.pool)
        .await
        .unwrap();
    let (_, list) = app.json("GET", "/api/igdb/titles", None).await;
    assert_eq!(list[0]["igdb_title"], "Super Mario Odyssey Deluxe");
}

#[tokio::test]
async fn sync_links_exact_titles_and_skips_ambiguous_ones() {
    let app = with_igdb().await;
    for title in ["super mario odyssey", "Doom", "Unknown"] {
        app.json("POST", "/api/games", Some(json!({"title": title})))
            .await;
    }

    let (status, started) = app.json("POST", "/api/igdb/sync", None).await;
    assert_eq!(status, StatusCode::ACCEPTED);
    assert_eq!(started["running"], true);

    let mut status = Value::Null;
    for _ in 0..100 {
        (_, status) = app.json("GET", "/api/igdb/sync", None).await;
        if status["running"] == false {
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert_eq!(status["running"], false, "{status}");
    assert_eq!(status["total"], 3);
    assert_eq!(status["linked"], 1);
    assert_eq!(status["unmatched"], 2);
    assert_eq!(status["failed"], 0);
    assert!(status["finished_at"].is_string());
    assert_eq!(
        app.scalar(
            "SELECT COUNT(*) FROM games WHERE igdb_id = 1 AND title = 'super mario odyssey'"
        )
        .await,
        1
    );
}

#[tokio::test]
async fn overwrite_replaces_hand_edits_where_igdb_has_values() {
    let app = with_igdb().await;
    let release = app.release("Odyssey", "switch").await;
    sqlx::query(
        "UPDATE releases SET release_date = '2000-01-01', local_max_players = 3 WHERE id = ?",
    )
    .bind(release)
    .execute(&app.pool)
    .await
    .unwrap();
    sqlx::query(
        "UPDATE games SET summary = 'Mine.', local_max_players = 8
         WHERE id = (SELECT game_id FROM releases WHERE id = ?)",
    )
    .bind(release)
    .execute(&app.pool)
    .await
    .unwrap();
    let game = app.scalar("SELECT id FROM games").await;

    let (status, detail) = app
        .json(
            "POST",
            &format!("/api/games/{game}/igdb"),
            Some(json!({"igdb_id": 1, "overwrite": true})),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{detail}");
    assert_eq!(detail["title"], "Super Mario Odyssey");
    assert_eq!(detail["summary"], "Cappy.");
    // IGDB has no game-level player count, so the hand-edited one stays.
    assert_eq!(detail["local_max_players"], 8);
    assert_eq!(detail["releases"][0]["release_date"], "2017-10-27");
    assert_eq!(detail["releases"][0]["local_max_players"], 2);
    assert_eq!(
        app.scalar("SELECT COUNT(*) FROM games WHERE sort_title = 'super mario odyssey'")
            .await,
        1
    );
}
