//! Who is using the app, from the Supervisor's ingress headers, and what they may do.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError};

use axum::Json;
use axum::extract::{FromRequestParts, MatchedPath, Request, State};
use axum::http::request::Parts;
use axum::http::{HeaderMap, Method, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use sqlx::SqlitePool;
use utoipa::ToSchema;

use crate::error::ErrorBody;

// The Supervisor drops client copies of these and sets its own, so they can
// be trusted once the peer check passed.
const USER_ID: &str = "x-remote-user-id";
const USER_NAME: &str = "x-remote-user-name";
const USER_DISPLAY_NAME: &str = "x-remote-user-display-name";

/// Reads anyone may make, except these manager tools.
const MANAGER_READS: [&str; 8] = [
    "/api/export.csv",
    "/api/import/template.csv",
    "/api/igdb/search",
    "/api/igdb/titles",
    "/api/games/{id}/igdb-contents",
    "/api/igdb/matches",
    "/api/igdb/sync",
    "/api/missing-details",
];

/// Play progress, open to viewers when the option allows it.
const PROGRESS_WRITES: [(Method, &str); 4] = [
    (Method::PUT, "/api/games/{id}/play-status"),
    (Method::POST, "/api/games/{id}/sessions"),
    (Method::PUT, "/api/sessions/{id}"),
    (Method::DELETE, "/api/sessions/{id}"),
];

/// Changes one person may have under way at once. Home Assistant passes a
/// body on as slowly as the person sends it, so without a limit one person
/// could hold every connection.
const WRITES_PER_PERSON: usize = 16;

/// Who may do what, from the app options.
#[derive(Debug)]
pub struct Policy {
    /// Home Assistant user IDs; while empty, nobody can change anything.
    pub managers: Vec<String>,
    pub viewers_track_progress: bool,
    writing: Arc<PerPerson>,
}

impl Policy {
    pub fn from_config(config: &crate::config::Config) -> Self {
        Self {
            managers: config.managers.clone(),
            viewers_track_progress: config.viewers_track_progress,
            writing: PerPerson::new(WRITES_PER_PERSON),
        }
    }

    /// Whether `user` may log play. Without managers nobody may; a scan
    /// from a reader with no Home Assistant user counts as a viewer.
    pub fn can_track_progress(&self, user: Option<&str>) -> bool {
        if self.managers.is_empty() {
            return false;
        }
        user.is_some_and(|id| self.managers.iter().any(|m| m == id)) || self.viewers_track_progress
    }

    /// Whether a roll from Home Assistant may record a pick, as Pick for me
    /// does for managers. A roll with no user comes from an automation,
    /// which only Home Assistant administrators can write.
    pub fn can_roll(&self, user: Option<&str>) -> bool {
        !self.managers.is_empty() && user.is_none_or(|id| self.managers.iter().any(|m| m == id))
    }
}

/// The Home Assistant user behind a request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct User {
    /// Stable ID, used for permissions.
    pub id: String,
    /// Display only.
    pub name: Option<String>,
    /// Display only.
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Viewer,
    Manager,
}

/// Added to every API request that passes [`guard`].
#[derive(Debug, Clone)]
pub struct Access {
    pub user: User,
    pub role: Role,
    pub managers_configured: bool,
    /// May change play status and last played.
    pub can_track_progress: bool,
}

/// Resolves the user and role, then rejects requests the role does not allow.
pub async fn guard(
    State(policy): State<Arc<Policy>>,
    mut request: Request,
    next: Next,
) -> Response {
    let Some(user) = user(request.headers()) else {
        return deny(StatusCode::UNAUTHORIZED, "identity_required");
    };
    let role = if policy.managers.contains(&user.id) {
        Role::Manager
    } else {
        Role::Viewer
    };

    let path = match request.extensions().get::<MatchedPath>() {
        Some(matched) => matched.as_str(),
        None => request.uri().path(),
    };
    let viewers_track_progress = policy.can_track_progress(None);
    if role < required_role(request.method(), path, viewers_track_progress) {
        tracing::debug!(method = %request.method(), path, "viewer denied");
        return deny(StatusCode::FORBIDDEN, "manager_required");
    }
    let _writing = if is_read(request.method()) {
        None
    } else {
        let Some(writing) = policy.writing.enter(&user.id) else {
            return deny(StatusCode::TOO_MANY_REQUESTS, "too_many_writes");
        };
        Some(writing)
    };

    request.extensions_mut().insert(Access {
        role,
        managers_configured: !policy.managers.is_empty(),
        can_track_progress: policy.can_track_progress(Some(&user.id)),
        user,
    });
    next.run(request).await
}

/// What each person has open, up to a limit, so one person cannot take
/// what everyone shares.
#[derive(Debug)]
pub struct PerPerson {
    limit: usize,
    open: Mutex<HashMap<String, usize>>,
}

impl PerPerson {
    pub fn new(limit: usize) -> Arc<Self> {
        Arc::new(Self {
            limit,
            open: Mutex::default(),
        })
    }

    /// A place for `user`, or `None` while they hold `limit` already.
    pub fn enter(self: &Arc<Self>, user: &str) -> Option<Place> {
        let mut open = self.open.lock().unwrap_or_else(PoisonError::into_inner);
        let count = open.entry(user.to_owned()).or_default();
        if *count == self.limit {
            return None;
        }
        *count += 1;
        Some(Place {
            counter: self.clone(),
            user: user.to_owned(),
        })
    }
}

/// A person's place, given back when dropped.
pub struct Place {
    counter: Arc<PerPerson>,
    user: String,
}

impl Drop for Place {
    fn drop(&mut self) {
        let mut open = self
            .counter
            .open
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        if let Some(count) = open.get_mut(&self.user) {
            *count -= 1;
            if *count == 0 {
                open.remove(&self.user);
            }
        }
    }
}

/// Keeps the names of users who change things. Runs after [`guard`].
pub async fn remember(State(db): State<SqlitePool>, request: Request, next: Next) -> Response {
    if !is_read(request.method())
        && let Some(access) = request.extensions().get::<Access>()
        && let Err(err) = crate::db::users::record(&db, &access.user).await
    {
        tracing::warn!(error = %err, "cannot record the user");
    }
    next.run(request).await
}

/// The ID of the user making a change, for `created_by` and `updated_by`.
pub struct Actor(pub String);

impl<S: Send + Sync> FromRequestParts<S> for Actor {
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<Access>()
            .map(|access| Self(access.user.id.clone()))
            .ok_or_else(|| deny(StatusCode::UNAUTHORIZED, "identity_required"))
    }
}

fn is_read(method: &Method) -> bool {
    method == Method::GET || method == Method::HEAD
}

fn required_role(method: &Method, path: &str, viewers_track_progress: bool) -> Role {
    let open = if is_read(method) {
        !MANAGER_READS.contains(&path)
    } else {
        viewers_track_progress
            && PROGRESS_WRITES
                .iter()
                .any(|(m, p)| m == method && *p == path)
    };
    if open { Role::Viewer } else { Role::Manager }
}

fn deny(status: StatusCode, code: &'static str) -> Response {
    (status, Json(ErrorBody { code, field: None })).into_response()
}

/// `None` when the Supervisor named no user or the ID is malformed.
fn user(headers: &HeaderMap) -> Option<User> {
    let mut ids = headers.get_all(USER_ID).iter();
    let (Some(id), None) = (ids.next(), ids.next()) else {
        return None;
    };
    let id = id.to_str().ok()?.trim();
    if !valid_user_id(id) {
        tracing::debug!("ignored malformed Home Assistant user id");
        return None;
    }
    Some(User {
        id: id.to_owned(),
        name: text(headers, USER_NAME),
        display_name: text(headers, USER_DISPLAY_NAME),
    })
}

/// The shape of a Home Assistant user ID.
pub fn valid_user_id(id: &str) -> bool {
    (1..=64).contains(&id.len())
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

fn text(headers: &HeaderMap, name: &str) -> Option<String> {
    let value = std::str::from_utf8(headers.get(name)?.as_bytes())
        .ok()?
        .trim();
    (!value.is_empty()).then(|| value.chars().take(100).collect())
}

#[cfg(test)]
mod tests {
    use axum::http::HeaderValue;

    use super::*;

    fn headers(pairs: &[(&'static str, &[u8])]) -> HeaderMap {
        let mut map = HeaderMap::new();
        for (name, value) in pairs {
            map.append(*name, HeaderValue::from_bytes(value).unwrap());
        }
        map
    }

    #[test]
    fn reads_the_supervisor_headers() {
        let user = user(&headers(&[
            (USER_ID, b"0a1b2c3d4e5f"),
            (USER_NAME, b"sam"),
            (USER_DISPLAY_NAME, "Sam Souz\u{e1}".as_bytes()),
        ]))
        .unwrap();
        assert_eq!(user.id, "0a1b2c3d4e5f");
        assert_eq!(user.name.as_deref(), Some("sam"));
        assert_eq!(user.display_name.as_deref(), Some("Sam Souz\u{e1}"));
    }

    #[test]
    fn names_are_optional() {
        let user = user(&headers(&[(USER_ID, b"abc"), (USER_NAME, b"  ")])).unwrap();
        assert_eq!(user.name, None);
        assert_eq!(user.display_name, None);
    }

    #[test]
    fn rejects_missing_or_malformed_ids() {
        assert_eq!(user(&headers(&[])), None);
        assert_eq!(user(&headers(&[(USER_ID, b"")])), None);
        assert_eq!(user(&headers(&[(USER_ID, b"a b")])), None);
        assert_eq!(user(&headers(&[(USER_ID, &[b'a'; 65])])), None);
        assert_eq!(user(&headers(&[(USER_ID, b"a"), (USER_ID, b"b")])), None);
    }

    #[test]
    fn viewers_read() {
        assert_eq!(
            required_role(&Method::GET, "/api/games", false),
            Role::Viewer
        );
        assert_eq!(
            required_role(&Method::HEAD, "/api/games/{id}", false),
            Role::Viewer
        );
    }

    #[test]
    fn managers_change_things_and_use_tools() {
        assert_eq!(
            required_role(&Method::POST, "/api/suggestions", true),
            Role::Manager
        );
        assert_eq!(
            required_role(&Method::POST, "/api/games", true),
            Role::Manager
        );
        assert_eq!(
            required_role(&Method::PUT, "/api/suggestions", true),
            Role::Manager
        );
        assert_eq!(
            required_role(&Method::PUT, "/api/games/{id}", true),
            Role::Manager
        );
        assert_eq!(
            required_role(&Method::DELETE, "/api/copies/{id}", true),
            Role::Manager
        );
        assert_eq!(
            required_role(&Method::GET, "/api/export.csv", true),
            Role::Manager
        );
        assert_eq!(
            required_role(&Method::GET, "/api/igdb/search", true),
            Role::Manager
        );
    }

    #[test]
    fn progress_follows_the_option() {
        for (method, path) in &PROGRESS_WRITES {
            assert_eq!(required_role(method, path, false), Role::Manager);
            assert_eq!(required_role(method, path, true), Role::Viewer);
        }
        assert_eq!(
            required_role(&Method::DELETE, "/api/games/{id}/sessions", true),
            Role::Manager
        );
    }
}
