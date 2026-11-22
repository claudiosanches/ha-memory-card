//! Raw request bodies that must say what they are. A browser sends a
//! cross-site form or `text/plain` without asking first; these media types
//! need a CORS preflight, which the app never answers.

use axum::Json;
use axum::body::Bytes;
use axum::extract::{FromRequest, Request};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};

use crate::error::ErrorBody;

/// A CSV file, sent as `text/csv`.
pub struct CsvBody(pub Bytes);

/// An image file, sent as `application/octet-stream` or `image/*`.
pub struct ImageBody(pub Bytes);

impl<S: Send + Sync> FromRequest<S> for CsvBody {
    type Rejection = Response;

    async fn from_request(request: Request, state: &S) -> Result<Self, Self::Rejection> {
        if !has_type(&request, |essence| essence == "text/csv") {
            return Err(unsupported());
        }
        let body = Bytes::from_request(request, state).await;
        body.map(Self).map_err(IntoResponse::into_response)
    }
}

impl<S: Send + Sync> FromRequest<S> for ImageBody {
    type Rejection = Response;

    async fn from_request(request: Request, state: &S) -> Result<Self, Self::Rejection> {
        let image =
            |essence: &str| essence == "application/octet-stream" || essence.starts_with("image/");
        if !has_type(&request, image) {
            return Err(unsupported());
        }
        let body = Bytes::from_request(request, state).await;
        body.map(Self).map_err(IntoResponse::into_response)
    }
}

fn has_type(request: &Request, allowed: impl Fn(&str) -> bool) -> bool {
    request
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(';').next())
        .is_some_and(|essence| allowed(&essence.trim().to_ascii_lowercase()))
}

fn unsupported() -> Response {
    (
        StatusCode::UNSUPPORTED_MEDIA_TYPE,
        Json(ErrorBody {
            code: "unsupported_media_type",
            field: None,
        }),
    )
        .into_response()
}
