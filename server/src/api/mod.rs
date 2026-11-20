pub mod devices;
mod input;
pub mod platforms;
pub mod status;
pub mod stores;

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
        .routes(routes!(stores::update, stores::delete));
    OpenApiRouter::with_openapi(ApiDoc::openapi()).nest("/api", api)
}
