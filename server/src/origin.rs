//! Refuses changes a browser sends on behalf of another site.

use axum::Json;
use axum::extract::Request;
use axum::http::{Method, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

use crate::error::ErrorBody;

/// Every write carries this header. A browser adds a header of its own to a
/// request for another origin only after a CORS preflight, which the app
/// never answers, so another site's form or script cannot send it.
pub const WRITE_HEADER: &str = "x-requested-with";

/// Writes without [`WRITE_HEADER`] are refused, and so are writes a browser
/// says in `Sec-Fetch-Site` come from another site. Browsers send that one
/// only over HTTPS. `Origin` is not compared with `Host`, since the
/// Supervisor rewrites `Host`.
pub async fn same_origin_writes(request: Request, next: Next) -> Response {
    let read = matches!(
        *request.method(),
        Method::GET | Method::HEAD | Method::OPTIONS
    );
    let headers = request.headers();
    let site = headers
        .get("sec-fetch-site")
        .and_then(|value| value.to_str().ok());
    let cross_site = site.is_some_and(|site| site != "same-origin" && site != "none");
    if !read && (cross_site || !headers.contains_key(WRITE_HEADER)) {
        tracing::debug!(method = %request.method(), path = request.uri().path(), "refused a cross-site write");
        return (
            StatusCode::FORBIDDEN,
            Json(ErrorBody {
                code: "cross_site",
                field: None,
            }),
        )
            .into_response();
    }
    next.run(request).await
}
