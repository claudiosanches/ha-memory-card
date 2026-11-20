pub mod api;
pub mod assets;
pub mod config;
pub mod db;
pub mod domain;
pub mod error;
pub mod igdb;
pub mod ingress;
pub mod logging;
pub mod shutdown;
pub mod transfer;

use std::net::IpAddr;
use std::sync::{Arc, Mutex};

use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::middleware;
use sqlx::SqlitePool;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

use crate::config::Config;
use crate::igdb::sync::SyncStatus;
use crate::igdb::{Endpoints, Igdb};

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub config: Arc<Config>,
    /// `None` without credentials.
    pub igdb: Option<Arc<Igdb>>,
    pub sync: Arc<Mutex<SyncStatus>>,
}

impl AppState {
    pub fn new(db: SqlitePool, config: Arc<Config>) -> Self {
        let igdb = config
            .igdb
            .clone()
            .map(|credentials| Arc::new(Igdb::new(credentials, Endpoints::default())));
        Self {
            db,
            config,
            igdb,
            sync: Arc::default(),
        }
    }
}

pub fn app(state: AppState) -> Router {
    let trusted: Arc<[IpAddr]> = state.config.trusted_peers.clone().into();
    let (api, _) = api::router().split_for_parts();

    let covers = ServeDir::new(state.config.covers_dir());

    api.nest_service("/covers", covers)
        .merge(assets::router())
        // CSV imports of a large collection exceed axum's 2 MB default.
        .layer(DefaultBodyLimit::max(16 * 1024 * 1024))
        .layer(middleware::from_fn_with_state(trusted, ingress::guard))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

pub fn openapi() -> utoipa::openapi::OpenApi {
    api::router().into_openapi()
}
