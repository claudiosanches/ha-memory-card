pub mod devices;
pub mod games;
pub mod igdb;
mod input;
pub mod platforms;
pub mod status;
pub mod stores;
pub mod transfer;

use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::AppState;

#[derive(OpenApi)]
#[openapi(info(
    title = "Memory Card",
    license(name = "GPL-3.0-only", identifier = "GPL-3.0-only")
))]
struct ApiDoc;

pub fn router() -> OpenApiRouter<AppState> {
    let api = OpenApiRouter::new()
        .routes(routes!(status::get_status))
        .routes(routes!(platforms::list, platforms::create))
        .routes(routes!(platforms::reorder))
        .routes(routes!(platforms::update, platforms::delete))
        .routes(routes!(devices::list, devices::create))
        .routes(routes!(devices::update, devices::delete))
        .routes(routes!(stores::list, stores::create))
        .routes(routes!(stores::update, stores::delete))
        .routes(routes!(games::list, games::create))
        .routes(routes!(games::get, games::update, games::delete))
        .routes(routes!(games::add_copy))
        .routes(routes!(games::update_copy, games::delete_copy))
        .routes(routes!(games::update_release, games::delete_release))
        .routes(routes!(games::game_modes))
        .routes(routes!(games::genres))
        .routes(routes!(igdb::search))
        .routes(routes!(igdb::link))
        .routes(routes!(igdb::sync_status, igdb::start_sync))
        .routes(routes!(transfer::export_csv))
        .routes(routes!(transfer::template_csv))
        .routes(routes!(transfer::preview))
        .routes(routes!(transfer::run));
    OpenApiRouter::with_openapi(ApiDoc::openapi()).nest("/api", api)
}
