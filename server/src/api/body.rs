//! Raw request bodies that must say what they are. A browser sends a
//! cross-site form or `text/plain` without asking first; these media types
//! need a CORS preflight, which the app never answers.

use axum::Json;
use axum::body::Bytes;
use axum::extract::{FromRequest, Request};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use http_body_util::{BodyExt, LengthLimitError, Limited};

use crate::error::ErrorBody;

/// Largest import or upload; a large collection's CSV can pass 2 MB. Every
/// other body stops at the app's default limit.
const MAX: usize = 16 * 1024 * 1024;

/// A CSV file, sent as `text/csv`.
pub struct CsvBody(pub Bytes);

/// An image file, sent as `application/octet-stream` or `image/*`.
pub struct ImageBody(pub Bytes);

impl<S: Send + Sync> FromRequest<S> for CsvBody {
    type Rejection = Response;

    async fn from_request(request: Request, _: &S) -> Result<Self, Self::Rejection> {
        if !has_type(&request, |essence| essence == "text/csv") {
            return Err(error(
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                "unsupported_media_type",
            ));
        }
        read(request).await.map(Self).map_err(|err| *err)
    }
}

impl<S: Send + Sync> FromRequest<S> for ImageBody {
    type Rejection = Response;

    async fn from_request(request: Request, _: &S) -> Result<Self, Self::Rejection> {
        let image =
            |essence: &str| essence == "application/octet-stream" || essence.starts_with("image/");
        if !has_type(&request, image) {
            return Err(error(
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                "unsupported_media_type",
            ));
        }
        read(request).await.map(Self).map_err(|err| *err)
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

async fn read(request: Request) -> Result<Bytes, Box<Response>> {
    match Limited::new(request.into_body(), MAX).collect().await {
        Ok(body) => Ok(body.to_bytes()),
        Err(err) if err.is::<LengthLimitError>() => {
            Err(Box::new(error(StatusCode::PAYLOAD_TOO_LARGE, "too_large")))
        }
        Err(_) => Err(Box::new(error(StatusCode::BAD_REQUEST, "invalid_body"))),
    }
}

fn error(status: StatusCode, code: &'static str) -> Response {
    (status, Json(ErrorBody { code, field: None })).into_response()
}
