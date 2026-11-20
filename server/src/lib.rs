pub mod api;
pub mod assets;
pub mod config;
pub mod db;
pub mod domain;
pub mod error;
pub mod ingress;
pub mod logging;
pub mod shutdown;
pub mod transfer;

use std::net::IpAddr;
use std::sync::Arc;

use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::middleware;
use sqlx::SqlitePool;
use tower_http::trace::TraceLayer;

use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub config: Arc<Config>,
}

pub fn app(state: AppState) -> Router {
    let trusted: Arc<[IpAddr]> = state.config.trusted_peers.clone().into();
    let (api, _) = api::router().split_for_parts();

    api.merge(assets::router())
        // CSV imports of a large collection exceed axum's 2 MB default.
        .layer(DefaultBodyLimit::max(16 * 1024 * 1024))
        .layer(middleware::from_fn_with_state(trusted, ingress::guard))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

pub fn openapi() -> utoipa::openapi::OpenApi {
    api::router().into_openapi()
}
