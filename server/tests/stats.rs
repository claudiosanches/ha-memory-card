//! The stats page totals.

mod common;

use axum::http::StatusCode;
use common::TestApp;
use serde_json::json;

const COLLECTION: &str = "game_title,play_status,platform,region,format,status,store,purchased_on,price,currency\n\
    Tetris,backlog,Switch,eu,physical,owned,Amazon,2025-03-01,30,USD\n\
    Tetris,backlog,PS4,na,digital,owned,,2026-01-10,10.50,USD\n\
    Hades,playing,PS4,na,digital,owned,PS Store,2026-02-01,100,BRL\n\
    Celeste,,Switch,ww,digital,owned,,,,\n\
    Silksong,,Switch,ww,physical,wanted,,,20,USD\n";

#[tokio::test]
async fn stats_total_the_collection() {
    let app = TestApp::new().await;
    let (status, result) = app.post_csv("/api/import", COLLECTION).await;
    assert_eq!(status, StatusCode::OK, "{result}");

    let hades: i64 = app
        .scalar("SELECT id FROM games WHERE title = 'Hades'")
        .await;
    for day in ["2026-11-01", "2026-11-02"] {
        app.json(
            "POST",
            &format!("/api/games/{hades}/sessions"),
            Some(json!({"played_on": day, "minutes": 30})),
        )
        .await;
    }

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
    assert_eq!(stats["stores"][0], json!({"store_id": null, "copies": 2}));
    assert_eq!(stats["total_sessions"], 2);
    assert_eq!(stats["total_minutes"], 60);
    assert_eq!(
        stats["most_played"],
        json!([{"game_id": hades, "title": "Hades", "sessions": 2, "minutes": 60}])
    );
}
