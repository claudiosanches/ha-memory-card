pub mod status;

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
    let api = OpenApiRouter::new().routes(routes!(status::get_status));
    OpenApiRouter::with_openapi(ApiDoc::openapi()).nest("/api", api)
}
