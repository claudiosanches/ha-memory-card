pub mod access;
pub mod api;
pub mod assets;
pub mod barcodes;
pub mod changes;
pub mod config;
pub mod covers;
pub mod db;
pub mod domain;
pub mod error;
pub mod http;
pub mod igdb;
pub mod images;
pub mod ingress;
pub mod logging;
pub mod money;
pub mod origin;
pub mod regional;
pub mod sensors;
pub mod serve;
pub mod shutdown;
pub mod slug;
pub mod tags;
pub mod transfer;

use std::net::IpAddr;
use std::sync::{Arc, Mutex};

use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::http::{HeaderName, HeaderValue, header};
use axum::middleware;
use tower_http::services::ServeDir;
use tower_http::set_header::{SetResponseHeader, SetResponseHeaderLayer};
use tower_http::trace::TraceLayer;

use crate::changes::Changes;
use crate::config::Config;
use crate::igdb::sync::SyncStatus;
use crate::igdb::{Endpoints, Igdb};
use crate::sensors::SensorsStatus;
use crate::tags::{PendingLink, TagsStatus};

#[derive(Clone)]
pub struct AppState {
    pub db: db::Pool,
    pub config: Arc<Config>,
    /// `None` without credentials.
    pub igdb: Option<Arc<Igdb>>,
    pub sync: Arc<Mutex<SyncStatus>>,
    /// Announced after every change to the collection.
    pub changes: Changes,
    pub sensors: Arc<Mutex<SensorsStatus>>,
    pub tag_status: Arc<Mutex<TagsStatus>>,
    /// The copy waiting for the next tag scan.
    pub tag_link: Arc<Mutex<Option<PendingLink>>>,
    /// For Home Assistant, the Supervisor and the barcode database.
    pub http: reqwest::Client,
    pub regional: Arc<tokio::sync::Mutex<regional::Cache>>,
}

impl AppState {
    pub fn new(db: db::Pool, config: Arc<Config>) -> Self {
        let igdb = config
            .igdb
            .clone()
            .map(|credentials| Arc::new(Igdb::new(credentials, Endpoints::default())));
        let sensors = SensorsStatus::initial(&config);
        let tags = TagsStatus::initial(config.home_assistant.as_ref());
        Self {
            db,
            config,
            igdb,
            sync: Arc::default(),
            changes: Changes::default(),
            sensors: Arc::new(Mutex::new(sensors)),
            tag_status: Arc::new(Mutex::new(tags)),
            tag_link: Arc::default(),
            http: http::shared(),
            regional: Arc::default(),
        }
    }
}

pub fn app(state: AppState) -> Router {
    let trusted: Arc<[IpAddr]> = state.config.trusted_peers.clone().into();
    let policy = Arc::new(access::Policy::from_config(&state.config));
    let (api, _) = api::router().split_for_parts();

    // Uploaded files never run as a page, even when opened directly.
    let covers = SetResponseHeader::overriding(
        ServeDir::new(state.config.covers_dir()),
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static("default-src 'none'; sandbox"),
    );

    api.nest_service("/covers", covers)
        .route_layer(middleware::from_fn_with_state(
            state.changes.clone(),
            sensors::track_changes,
        ))
        .route_layer(middleware::from_fn_with_state(
            state.db.clone(),
            access::remember,
        ))
        // Assets stay open so the UI can explain a missing identity.
        .route_layer(middleware::from_fn_with_state(policy, access::guard))
        .merge(assets::router())
        // Imports and uploads read larger bodies themselves; see `api::body`.
        .layer(DefaultBodyLimit::max(1024 * 1024))
        .layer(middleware::from_fn(origin::same_origin_writes))
        .layer(middleware::from_fn_with_state(trusted, ingress::guard))
        // Under ingress the app shares Home Assistant's origin, so a hole here
        // reaches the Home Assistant session.
        .layer(SetResponseHeaderLayer::if_not_present(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::REFERRER_POLICY,
            HeaderValue::from_static("no-referrer"),
        ))
        // The barcode scanner reads the camera; nothing else may.
        .layer(SetResponseHeaderLayer::if_not_present(
            HeaderName::from_static("permissions-policy"),
            HeaderValue::from_static("camera=(self), geolocation=(), microphone=()"),
        ))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

pub fn openapi() -> utoipa::openapi::OpenApi {
    api::router().into_openapi()
}
