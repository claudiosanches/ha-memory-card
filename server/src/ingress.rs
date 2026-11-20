use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use axum::extract::{ConnectInfo, Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

/// Rejects every peer except the trusted ones (the Supervisor in production).
/// Home Assistant authenticates ingress users, so this check is the app's only gate.
pub async fn guard(
    State(trusted): State<Arc<[IpAddr]>>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    request: Request,
    next: Next,
) -> Response {
    let ip = peer.ip().to_canonical();
    if trusted.contains(&ip) {
        next.run(request).await
    } else {
        tracing::warn!(%ip, "rejected request from untrusted peer");
        StatusCode::FORBIDDEN.into_response()
    }
}
