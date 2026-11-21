use axum::{Extension, Json};
use serde::Serialize;
use utoipa::ToSchema;

use crate::access::{Access, Role, User};
use crate::error::ErrorBody;

#[derive(Debug, Serialize, ToSchema)]
pub struct Me {
    pub user: User,
    pub role: Role,
    /// False until the app options name a manager; until then nobody can change anything.
    pub managers_configured: bool,
    /// May change play status and last played.
    pub can_track_progress: bool,
}

#[utoipa::path(
    get,
    path = "/me",
    tag = "access",
    operation_id = "getMe",
    responses(
        (status = OK, body = Me),
        (status = UNAUTHORIZED, description = "The request names no Home Assistant user", body = ErrorBody),
    )
)]
pub async fn get_me(Extension(access): Extension<Access>) -> Json<Me> {
    Json(Me {
        user: access.user,
        role: access.role,
        managers_configured: access.managers_configured,
        can_track_progress: access.can_track_progress,
    })
}
