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
    #[error("conflict: {0}")]
    Conflict(&'static str),
    #[error("invalid {field:?}: {code}")]
    Invalid {
        code: &'static str,
        field: Option<&'static str>,
    },
    #[error(transparent)]
    Database(#[from] sqlx::Error),
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
            Self::Conflict(code) => (StatusCode::CONFLICT, code, None),
            Self::Invalid { code, field } => (StatusCode::UNPROCESSABLE_ENTITY, code, field),
            Self::Database(err) => {
                tracing::error!(error = %err, "database error");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal", None)
            }
        };
        (status, Json(ErrorBody { code, field })).into_response()
    }
}

impl From<JsonRejection> for ApiError {
    fn from(rejection: JsonRejection) -> Self {
        tracing::debug!(%rejection, "rejected request body");
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
