use axum::{Extension, Json};
use serde::Serialize;
use utoipa::ToSchema;

use crate::access::{Access, Role, User};
use crate::error::ErrorBody;

#[derive(Debug, Serialize, ToSchema)]
pub struct Me {
    /// `None` when the Supervisor named no user.
    pub user: Option<User>,
    pub role: Role,
    /// False while no managers are configured, when everyone manages.
    pub enforced: bool,
}

#[utoipa::path(
    get,
    path = "/me",
    tag = "access",
    operation_id = "getMe",
    responses(
        (status = OK, body = Me),
        (status = UNAUTHORIZED, description = "Managers are configured and the request names no user", body = ErrorBody),
    )
)]
pub async fn get_me(Extension(access): Extension<Access>) -> Json<Me> {
    Json(Me {
        user: access.user,
        role: access.role,
        enforced: access.enforced,
    })
}
