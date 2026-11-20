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
}
