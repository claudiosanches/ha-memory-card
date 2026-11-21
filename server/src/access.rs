//! Who is using the app, from the Supervisor's ingress headers, and what they may do.

use std::sync::Arc;

use axum::Json;
use axum::extract::{MatchedPath, Request, State};
use axum::http::{HeaderMap, Method, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use utoipa::ToSchema;

use crate::error::ErrorBody;

// The Supervisor drops client copies of these and sets its own, so they can
// be trusted once the peer check passed.
const USER_ID: &str = "x-remote-user-id";
const USER_NAME: &str = "x-remote-user-name";
const USER_DISPLAY_NAME: &str = "x-remote-user-display-name";

/// Reads anyone may make, except these manager tools.
const MANAGER_READS: [&str; 4] = [
    "/api/export.csv",
    "/api/import/template.csv",
    "/api/igdb/search",
    "/api/igdb/titles",
];

/// Changes anyone may make. A suggestion only records what was rolled.
const VIEWER_WRITES: [&str; 1] = ["/api/suggestions"];

/// Play progress, open to viewers when the option allows it.
const PROGRESS_WRITES: [&str; 2] = ["/api/games/{id}/play-status", "/api/games/{id}/last-played"];

/// Who may do what, from the app options.
#[derive(Debug)]
pub struct Policy {
    /// Home Assistant user IDs; empty means everyone manages.
    pub managers: Vec<String>,
    pub viewers_track_progress: bool,
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
    pub user: Option<User>,
    pub role: Role,
    /// False while no managers are configured, when everyone manages.
    pub enforced: bool,
    /// May change play status and last played.
    pub can_track_progress: bool,
}

/// Resolves the user and role, then rejects requests the role does not allow.
/// With no managers configured, everyone is a manager and identity is optional.
pub async fn guard(
    State(policy): State<Arc<Policy>>,
    mut request: Request,
    next: Next,
) -> Response {
    let user = user(request.headers());
    let enforced = !policy.managers.is_empty();
    let role = match &user {
        _ if !enforced => Role::Manager,
        None => return deny(StatusCode::UNAUTHORIZED, "identity_required"),
        Some(user) if policy.managers.contains(&user.id) => Role::Manager,
        Some(_) => Role::Viewer,
    };

    let path = match request.extensions().get::<MatchedPath>() {
        Some(matched) => matched.as_str(),
        None => request.uri().path(),
    };
    if role < required_role(request.method(), path, policy.viewers_track_progress) {
        tracing::debug!(method = %request.method(), path, "viewer denied");
        return deny(StatusCode::FORBIDDEN, "manager_required");
    }

    request.extensions_mut().insert(Access {
        user,
        role,
        enforced,
        can_track_progress: role == Role::Manager || policy.viewers_track_progress,
    });
    next.run(request).await
}

fn required_role(method: &Method, path: &str, viewers_track_progress: bool) -> Role {
    let open = if method == Method::GET || method == Method::HEAD {
        !MANAGER_READS.contains(&path)
    } else if method == Method::PUT && PROGRESS_WRITES.contains(&path) {
        viewers_track_progress
    } else {
        method == Method::POST && VIEWER_WRITES.contains(&path)
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
    let valid = (1..=64).contains(&id.len())
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_');
    if !valid {
        tracing::debug!("ignored malformed Home Assistant user id");
        return None;
    }
    Some(User {
        id: id.to_owned(),
        name: text(headers, USER_NAME),
        display_name: text(headers, USER_DISPLAY_NAME),
    })
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
    fn viewers_read_and_roll_suggestions() {
        assert_eq!(
            required_role(&Method::GET, "/api/games", false),
            Role::Viewer
        );
        assert_eq!(
            required_role(&Method::HEAD, "/api/games/{id}", false),
            Role::Viewer
        );
        assert_eq!(
            required_role(&Method::POST, "/api/suggestions", false),
            Role::Viewer
        );
    }

    #[test]
    fn managers_change_things_and_use_tools() {
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
        for path in PROGRESS_WRITES {
            assert_eq!(required_role(&Method::PUT, path, false), Role::Manager);
            assert_eq!(required_role(&Method::PUT, path, true), Role::Viewer);
        }
    }
}
