use axum::Json;
use axum::extract::rejection::{JsonRejection, PathRejection};
use axum::extract::{FromRequest, FromRequestParts};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use utoipa::ToSchema;

/// Error body. `code` is a stable key the UI translates; it never carries
/// display text. `field` names the input field at fault, when there is one.
#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorBody {
    pub code: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<&'static str>,
}

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("not found")]
    NotFound,
    /// A search with no result, as a 404 with its own code.
    #[error("nothing matched: {0}")]
    NoMatch(&'static str),
    #[error("conflict: {0}")]
    Conflict(&'static str),
    #[error("forbidden: {0}")]
    Forbidden(&'static str),
    #[error("request body too large")]
    TooLarge,
    #[error("invalid {field:?}: {code}")]
    Invalid {
        code: &'static str,
        field: Option<&'static str>,
    },
    /// A feature that needs setup, such as IGDB credentials.
    #[error("unavailable: {0}")]
    Unavailable(&'static str),
    /// Too much of something at once; worth trying again later.
    #[error("busy: {0}")]
    Busy(&'static str),
    #[error("IGDB request failed: {0}")]
    Upstream(String),
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error("{0}")]
    Internal(String),
}

impl ApiError {
    pub fn invalid(code: &'static str, field: &'static str) -> Self {
        Self::Invalid {
            code,
            field: Some(field),
        }
    }

    /// Maps a unique-constraint violation to `Conflict(code)`; other errors stay database errors.
    pub fn unique(err: sqlx::Error, code: &'static str) -> Self {
        match err.as_database_error() {
            Some(db) if db.is_unique_violation() => Self::Conflict(code),
            _ => Self::Database(err),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, field) = match self {
            Self::NotFound => (StatusCode::NOT_FOUND, "not_found", None),
            Self::NoMatch(code) => (StatusCode::NOT_FOUND, code, None),
            Self::Conflict(code) => (StatusCode::CONFLICT, code, None),
            Self::Forbidden(code) => (StatusCode::FORBIDDEN, code, None),
            Self::TooLarge => (StatusCode::PAYLOAD_TOO_LARGE, "too_large", None),
            Self::Invalid { code, field } => (StatusCode::UNPROCESSABLE_ENTITY, code, field),
            Self::Unavailable(code) => (StatusCode::SERVICE_UNAVAILABLE, code, None),
            Self::Busy(code) => (StatusCode::TOO_MANY_REQUESTS, code, None),
            Self::Upstream(err) => {
                tracing::warn!(error = %err, "IGDB request failed");
                (StatusCode::BAD_GATEWAY, "igdb_failed", None)
            }
            Self::Database(err) => {
                tracing::error!(error = %err, "database error");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal", None)
            }
            Self::Internal(err) => {
                tracing::error!(error = %err, "internal error");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal", None)
            }
        };
        (status, Json(ErrorBody { code, field })).into_response()
    }
}

impl From<crate::igdb::IgdbError> for ApiError {
    fn from(err: crate::igdb::IgdbError) -> Self {
        Self::Upstream(err.to_string())
    }
}

impl From<JsonRejection> for ApiError {
    fn from(rejection: JsonRejection) -> Self {
        tracing::debug!(%rejection, "rejected request body");
        if rejection.status() == StatusCode::PAYLOAD_TOO_LARGE {
            return Self::TooLarge;
        }
        Self::Invalid {
            code: "invalid_body",
            field: None,
        }
    }
}

impl From<PathRejection> for ApiError {
    fn from(_: PathRejection) -> Self {
        Self::NotFound
    }
}

/// `Json` whose rejections use the API error body.
#[derive(FromRequest)]
#[from_request(via(axum::Json), rejection(ApiError))]
pub struct ApiJson<T>(pub T);

/// `Path` whose rejections use the API error body.
#[derive(FromRequestParts)]
#[from_request(via(axum::extract::Path), rejection(ApiError))]
pub struct ApiPath<T>(pub T);

pub type ApiResult<T> = Result<T, ApiError>;
