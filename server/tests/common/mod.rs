#![allow(dead_code)]

use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::extract::connect_info::MockConnectInfo;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use memory_card::config::Config;
use memory_card::{AppState, db};
use serde_json::Value;
use sqlx::SqlitePool;
use tempfile::TempDir;
use tower::ServiceExt;

pub const SUPERVISOR: &str = "172.30.32.2:50000";

pub struct TestApp {
    pub dir: TempDir,
    pub pool: SqlitePool,
    pub config: Arc<Config>,
}

impl TestApp {
    pub async fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let data_dir = dir.path().display().to_string();
        let config = Config::load(|name| match name {
            "MEMORY_CARD_DATA_DIR" => Some(data_dir.clone()),
            _ => None,
        })
        .unwrap();
        let pool = db::open(
            &config.db_path(),
            &config.pre_migration_dir(),
            &db::MIGRATOR,
        )
        .await
        .unwrap();
        Self {
            dir,
            pool,
            config: Arc::new(config),
        }
    }

    pub fn router_from(&self, peer: &str) -> Router {
        let peer: SocketAddr = peer.parse().unwrap();
        memory_card::app(AppState {
            db: self.pool.clone(),
            config: self.config.clone(),
        })
        .layer(MockConnectInfo(peer))
    }

    pub async fn get(&self, path: &str) -> (StatusCode, Vec<u8>) {
        self.request(SUPERVISOR, Request::get(path).body(Body::empty()).unwrap())
            .await
    }

    pub async fn request(&self, peer: &str, request: Request<Body>) -> (StatusCode, Vec<u8>) {
        let response = self.router_from(peer).oneshot(request).await.unwrap();
        let status = response.status();
        let body = response.into_body().collect().await.unwrap().to_bytes();
        (status, body.to_vec())
    }

    /// Sends JSON (or nothing) and parses the JSON reply; an empty reply is `Value::Null`.
    pub async fn json(&self, method: &str, path: &str, body: Option<Value>) -> (StatusCode, Value) {
        let builder = Request::builder().method(method).uri(path);
        let request = match body {
            Some(body) => builder
                .header("content-type", "application/json")
                .body(Body::from(body.to_string())),
            None => builder.body(Body::empty()),
        }
        .unwrap();
        let (status, bytes) = self.request(SUPERVISOR, request).await;
        let value = if bytes.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&bytes).unwrap()
        };
        (status, value)
    }

    /// Posts a CSV body and parses the JSON reply.
    pub async fn post_csv(&self, path: &str, csv: &str) -> (StatusCode, Value) {
        let request = Request::post(path)
            .header("content-type", "text/csv")
            .body(Body::from(csv.to_owned()))
            .unwrap();
        let (status, bytes) = self.request(SUPERVISOR, request).await;
        (status, serde_json::from_slice(&bytes).unwrap())
    }

    pub async fn scalar(&self, sql: &'static str) -> i64 {
        sqlx::query_scalar(sql).fetch_one(&self.pool).await.unwrap()
    }

    pub async fn platform_id(&self, slug: &str) -> i64 {
        sqlx::query_scalar("SELECT id FROM platforms WHERE slug = ?")
            .bind(slug)
            .fetch_one(&self.pool)
            .await
            .unwrap()
    }

    /// Inserts a game with one release on `platform_slug` and returns the release id.
    pub async fn release(&self, title: &str, platform_slug: &str) -> i64 {
        let platform = self.platform_id(platform_slug).await;
        let game: i64 = sqlx::query_scalar(
            "INSERT INTO games (title, sort_title) VALUES (?, lower(?)) RETURNING id",
        )
        .bind(title)
        .bind(title)
        .fetch_one(&self.pool)
        .await
        .unwrap();
        sqlx::query_scalar(
            "INSERT INTO releases (game_id, platform_id, region) VALUES (?, ?, 'na') RETURNING id",
        )
        .bind(game)
        .bind(platform)
        .fetch_one(&self.pool)
        .await
        .unwrap()
    }
}
