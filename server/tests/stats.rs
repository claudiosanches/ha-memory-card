//! The stats page totals.
// Tests set up and read rows with SQL.
#![allow(clippy::disallowed_methods)]

mod common;

use axum::http::StatusCode;
use common::TestApp;
use serde_json::json;

const COLLECTION: &str = "game_title,play_status,platform,region,format,status,store,purchased_on,price,currency,estimated_value\n\
    Tetris,backlog,Switch,eu,physical,owned,Amazon,2025-03-01,30,USD,50\n\
    Tetris,backlog,PS4,na,digital,owned,,2026-01-10,10.50,USD,\n\
    Hades,playing,PS4,na,digital,owned,PS Store,2026-02-01,100,BRL,150\n\
    Celeste,,Switch,ww,digital,owned,,,,,\n\
    Silksong,,Switch,ww,physical,wanted,,,20,USD,25\n";

#[tokio::test]
async fn stats_total_the_collection() {
    let app = TestApp::new().await;
    let (status, result) = app.post_csv("/api/import", COLLECTION).await;
    assert_eq!(status, StatusCode::OK, "{result}");

    let hades: i64 = app
        .scalar("SELECT id FROM games WHERE title = 'Hades'")
        .await;
    let yesterday: String = sqlx::query_scalar("SELECT date('now', '-1 day')")
        .fetch_one(&app.pool)
        .await
        .unwrap();
    for day in ["2024-11-01", yesterday.as_str()] {
        app.json(
            "POST",
            &format!("/api/games/{hades}/sessions"),
            Some(json!({"played_on": day, "minutes": 30})),
        )
        .await;
    }

    // Someone else's play is not the manager's.
    sqlx::query("INSERT INTO users (id) VALUES ('someone-else')")
        .execute(&app.pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO play_sessions (user_id, game_id, played_on, minutes)
         VALUES ('someone-else', ?, date('now'), 90)",
    )
    .bind(hades)
    .execute(&app.pool)
    .await
    .unwrap();

    let (status, stats) = app.json("GET", "/api/stats", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(stats["games"], 3);
    assert_eq!(stats["copies"], 4);
    assert_eq!(
        (stats["physical"].clone(), stats["digital"].clone()),
        (json!(1), json!(3))
    );
    assert_eq!(stats["wishlist"], 1);
    assert_eq!(stats["unpriced"], 1);
    assert_eq!(
        stats["spent"],
        json!([
            {"currency": "USD", "amount": "40.50", "copies": 2},
            {"currency": "BRL", "amount": "100.00", "copies": 1},
        ])
    );
    assert_eq!(
        stats["estimated"],
        json!([
            {"currency": "BRL", "amount": "150.00", "copies": 1},
            {"currency": "USD", "amount": "50.00", "copies": 1},
        ]),
        "owned copies only"
    );
    assert_eq!(stats["unvalued"], 2);
    assert_eq!(
        stats["wishlist_value"],
        json!([{"currency": "USD", "amount": "20.00", "copies": 1}])
    );
    assert_eq!(stats["spent_by_year"].as_array().unwrap().len(), 3);
    assert_eq!(stats["spent_by_year"][0]["year"], "2025");
    assert_eq!(
        stats["play_statuses"],
        json!([
            {"play_status": "backlog", "games": 1},
            {"play_status": "playing", "games": 1},
            {"play_status": null, "games": 1},
        ])
    );
    let platforms = stats["platforms"].as_array().unwrap();
    assert_eq!(platforms.len(), 2);
    let formats: Vec<_> = platforms
        .iter()
        .map(|p| (p["physical"].clone(), p["digital"].clone()))
        .collect();
    assert!(formats.contains(&(json!(1), json!(1))), "{formats:?}");
    assert!(formats.contains(&(json!(0), json!(2))), "{formats:?}");
    assert_eq!(
        stats["regions"],
        json!([
            {"region": "na", "copies": 2},
            {"region": "eu", "copies": 1},
            {"region": "ww", "copies": 1},
        ])
    );
    assert_eq!(
        stats["conditions"],
        json!([{"condition": null, "copies": 1}]),
        "physical copies only"
    );
    let years = stats["added_by_year"].as_array().unwrap();
    assert_eq!(years[0], json!({"period": "2025", "copies": 1}));
    assert_eq!(
        years
            .iter()
            .map(|y| y["copies"].as_i64().unwrap())
            .sum::<i64>(),
        4
    );
    let this_month: String = sqlx::query_scalar("SELECT strftime('%Y-%m', 'now')")
        .fetch_one(&app.pool)
        .await
        .unwrap();
    assert_eq!(
        stats["added_by_month"].as_array().unwrap().last().unwrap(),
        &json!({"period": this_month, "copies": 1}),
        "a copy without a purchase date counts from the day it was added"
    );
    assert_eq!(
        stats["value_change"],
        json!([
            {"currency": "BRL", "paid": "100.00", "estimated": "150.00", "change": "50.00", "copies": 1},
            {"currency": "USD", "paid": "30.00", "estimated": "50.00", "change": "20.00", "copies": 1},
        ])
    );
    let valuable: Vec<_> = stats["most_valuable"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| {
            (
                c["title"].clone(),
                c["estimated"].clone(),
                c["paid"].clone(),
            )
        })
        .collect();
    assert_eq!(
        valuable,
        [
            (json!("Hades"), json!("150.00"), json!("100.00")),
            (json!("Tetris"), json!("50.00"), json!("30.00")),
        ]
    );
    assert_eq!(
        stats["play_days"],
        json!([{"day": yesterday, "sessions": 1, "minutes": 30}]),
        "the last year only"
    );
    assert_eq!(stats["stores"][0], json!({"store_id": null, "copies": 2}));
    assert_eq!(stats["total_sessions"], 2);
    assert_eq!(stats["total_minutes"], 60);
    assert_eq!(
        stats["most_played"],
        json!([{"game_id": hades, "title": "Hades", "sessions": 2, "minutes": 60}])
    );
}

#[tokio::test]
async fn platform_totals_count_bundled_games_and_the_bundle_once() {
    let app = TestApp::new().await;
    let switch = app.platform_id("switch").await;
    let mut games = Vec::new();
    for title in ["Super Mario 3D All-Stars", "Super Mario Sunshine"] {
        let (_, game) = app
            .json("POST", "/api/games", Some(json!({"title": title})))
            .await;
        games.push(game["id"].as_i64().unwrap());
    }
    let (_, detail) = app
        .json(
            "POST",
            &format!("/api/games/{}/copies", games[0]),
            Some(json!({"platform_id": switch, "region": "eu", "format": "physical"})),
        )
        .await;
    let bundle = detail["releases"][0]["copies"][0]["id"].as_i64().unwrap();
    let (status, body) = app
        .json(
            "POST",
            &format!("/api/copies/{bundle}/contents"),
            Some(json!({"game_id": games[1]})),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "{body}");

    let (_, stats) = app.json("GET", "/api/stats", None).await;
    let platform = &stats["platforms"][0];
    assert_eq!(platform["games"], stats["games"]);
    assert_eq!(platform["games"], 2);
    assert_eq!(platform["copies"], 1);
    assert_eq!(platform["physical"], 1);
}
