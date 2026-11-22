//! Refuses changes a browser sends on behalf of another site.

use axum::Json;
use axum::extract::Request;
use axum::http::{Method, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

use crate::error::ErrorBody;

/// Browsers say where a request comes from in `Sec-Fetch-Site`. Writes from
/// another site are refused. `Origin` is not compared with `Host`, since the
/// Supervisor rewrites `Host`; requests without the header, from scripts or
/// old browsers, pass.
pub async fn same_origin_writes(request: Request, next: Next) -> Response {
    let read = matches!(
        *request.method(),
        Method::GET | Method::HEAD | Method::OPTIONS
    );
    let site = request
        .headers()
        .get("sec-fetch-site")
        .and_then(|value| value.to_str().ok());
    if !read && site.is_some_and(|site| site != "same-origin" && site != "none") {
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
