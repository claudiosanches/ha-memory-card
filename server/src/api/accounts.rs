use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use serde::Deserialize;
use utoipa::ToSchema;

use super::input::{NAME_MAX, NOTES_MAX, optional, required};
use crate::AppState;
use crate::access::valid_user_id;
use crate::db::accounts::{self, Account, AccountFields};
use crate::db::users::Person;
use crate::error::{ApiError, ApiJson, ApiPath, ApiResult, ErrorBody};

#[derive(Debug, Deserialize, ToSchema)]
pub struct AccountInput {
    pub name: String,
    /// Names it in library links. Blank makes one from the name for a new
    /// one and keeps the current one on update.
    #[serde(default)]
    pub slug: Option<String>,
    #[serde(default)]
    pub store_id: Option<i64>,
    /// The Home Assistant user whose account it is.
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default = "default_active")]
    pub active: bool,
    #[serde(default, deserialize_with = "super::input::id_list")]
    pub platform_ids: Vec<i64>,
}

fn default_active() -> bool {
    true
}

impl AccountInput {
    fn validate(&self) -> ApiResult<AccountFields> {
        let user_id = self
            .user_id
            .as_deref()
            .map(str::trim)
            .filter(|id| !id.is_empty());
        if user_id.is_some_and(|id| !valid_user_id(id)) {
            return Err(ApiError::invalid("invalid_user", "user_id"));
        }
        Ok(AccountFields {
            name: required(&self.name, "name", NAME_MAX)?,
            slug: crate::slug::input(self.slug.as_deref())?,
            store_id: self.store_id,
            user_id: user_id.map(str::to_owned),
            notes: optional(self.notes.as_deref(), "notes", NOTES_MAX)?,
            active: self.active,
            platform_ids: self.platform_ids.clone(),
        })
    }
}

#[utoipa::path(
    get, path = "/accounts", tag = "accounts", operation_id = "listAccounts",
    responses((status = OK, body = Vec<Account>))
)]
pub async fn list(State(state): State<AppState>) -> ApiResult<Json<Vec<Account>>> {
    Ok(Json(accounts::list(&state.db).await?))
}

#[utoipa::path(
    post, path = "/accounts", tag = "accounts", operation_id = "createAccount",
    request_body = AccountInput,
    responses(
        (status = CREATED, body = Account),
        (status = CONFLICT, description = "`name_taken`, `slug_taken`", body = ErrorBody),
        (status = UNPROCESSABLE_ENTITY, description = "Includes `unknown_platform`, `unknown_store`, `invalid_user` and `unknown_user`", body = ErrorBody),
    )
)]
pub async fn create(
    State(state): State<AppState>,
    ApiJson(input): ApiJson<AccountInput>,
) -> ApiResult<(StatusCode, Json<Account>)> {
    let id = accounts::create(&state.db, &input.validate()?).await?;
    Ok((
        StatusCode::CREATED,
        Json(accounts::get(&state.db, id).await?),
    ))
}

#[utoipa::path(
    put, path = "/accounts/{id}", tag = "accounts", operation_id = "updateAccount",
    params(("id" = i64, Path)),
    request_body = AccountInput,
    responses(
        (status = OK, body = Account),
        (status = NOT_FOUND, body = ErrorBody),
        (status = CONFLICT, description = "`name_taken`, `slug_taken`", body = ErrorBody),
        (status = UNPROCESSABLE_ENTITY, description = "Includes `unknown_platform`, `unknown_store`, `invalid_user` and `unknown_user`", body = ErrorBody),
    )
)]
pub async fn update(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<i64>,
    ApiJson(input): ApiJson<AccountInput>,
) -> ApiResult<Json<Account>> {
    accounts::update(&state.db, id, &input.validate()?).await?;
    Ok(Json(accounts::get(&state.db, id).await?))
}

#[utoipa::path(
    delete, path = "/accounts/{id}", tag = "accounts", operation_id = "deleteAccount",
    params(("id" = i64, Path)),
    responses(
        (status = NO_CONTENT, description = "Deleted; its copies keep existing with no account"),
        (status = NOT_FOUND, body = ErrorBody),
    )
)]
pub async fn delete(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<i64>,
) -> ApiResult<StatusCode> {
    accounts::delete(&state.db, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get, path = "/people", tag = "accounts", operation_id = "listPeople",
    responses((status = OK, description = "Everyone Memory Card knows, by name", body = Vec<Person>))
)]
pub async fn people(State(state): State<AppState>) -> ApiResult<Json<Vec<Person>>> {
    Ok(Json(accounts::people(&state.db).await?))
}
