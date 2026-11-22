pub mod barcodes;
pub mod body;
pub mod covers;
pub mod devices;
pub mod duplicates;
pub mod games;
pub mod igdb;
mod input;
pub mod me;
pub mod missing;
pub mod photos;
pub mod platforms;
pub mod play_statuses;
pub mod sessions;
pub mod stats;
pub mod status;
pub mod stores;
pub mod suggestions;
pub mod tags;
pub mod transfer;

use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::AppState;

#[derive(OpenApi)]
#[openapi(info(
    title = "Memory Card",
    license(name = "GPL-3.0-or-later", identifier = "GPL-3.0-or-later")
))]
struct ApiDoc;

pub fn router() -> OpenApiRouter<AppState> {
    let api = OpenApiRouter::new()
        .routes(routes!(status::get_status))
        .routes(routes!(stats::get))
        .routes(routes!(barcodes::look_up))
        .routes(routes!(me::get_me))
        .routes(routes!(platforms::list, platforms::create))
        .routes(routes!(platforms::reorder))
        .routes(routes!(platforms::update, platforms::delete))
        .routes(routes!(devices::list, devices::create))
        .routes(routes!(devices::update, devices::delete))
        .routes(routes!(play_statuses::list, play_statuses::create))
        .routes(routes!(play_statuses::reorder))
        .routes(routes!(play_statuses::update, play_statuses::delete))
        .routes(routes!(stores::list, stores::create))
        .routes(routes!(stores::update, stores::delete))
        .routes(routes!(games::list, games::create))
        .routes(routes!(games::get, games::update, games::delete))
        .routes(routes!(games::set_play_status))
        .routes(routes!(sessions::add))
        .routes(routes!(sessions::update, sessions::delete))
        .routes(routes!(games::add_copy))
        .routes(routes!(games::list_copies))
        .routes(routes!(games::update_copy, games::delete_copy))
        .routes(routes!(games::add_content))
        .routes(routes!(photos::add))
        .routes(routes!(photos::delete))
        .routes(routes!(tags::start_link, tags::cancel_link))
        .routes(routes!(tags::unlink))
        .routes(routes!(games::update_release, games::delete_release))
        .routes(routes!(covers::upload_game, covers::remove_game))
        .routes(routes!(covers::upload_release, covers::remove_release))
        .routes(routes!(games::game_modes))
        .routes(routes!(games::genres))
        .routes(routes!(duplicates::list))
        .routes(routes!(missing::list))
        .routes(routes!(suggestions::suggest))
        .routes(routes!(igdb::search))
        .routes(routes!(igdb::link))
        .routes(routes!(igdb::bundle_contents))
        .routes(routes!(igdb::matches))
        .routes(routes!(igdb::skip))
        .routes(routes!(igdb::sync_status, igdb::start_sync))
        .routes(routes!(igdb::title_differences, igdb::resolve_titles))
        .routes(routes!(transfer::export_csv))
        .routes(routes!(transfer::template_csv))
        .routes(routes!(transfer::preview))
        .routes(routes!(transfer::run));
    OpenApiRouter::with_openapi(ApiDoc::openapi()).nest("/api", api)
}
