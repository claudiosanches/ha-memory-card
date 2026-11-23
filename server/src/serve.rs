//! Serves a router over HTTP/1.1 like `axum::serve`, with the limits it
//! lacks: a cap on open connections, a deadline for request headers, and
//! the peers allowed to connect at all. Other containers on the Supervisor's
//! network can reach the app's ports, so idle sockets must not pile up.

use std::future::Future;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use axum::extract::ConnectInfo;
use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper_util::rt::{TokioIo, TokioTimer};
use hyper_util::service::TowerToHyperService;
use tokio::net::TcpListener;
use tokio::sync::{Semaphore, watch};
use tower::ServiceExt;

/// A pause after a failed accept, such as running out of file descriptors.
const ACCEPT_BACKOFF: Duration = Duration::from_millis(100);

pub struct Limits {
    pub connections: usize,
    /// A connection that has not sent a request's headers in time closes.
    pub header_timeout: Duration,
    /// Connections from anyone else close unanswered; `None` lets everyone in.
    pub peers: Option<Arc<[IpAddr]>>,
}

/// Serves until `shutdown`, then lets open connections finish their
/// responses and returns once they all closed.
pub async fn serve(
    listener: TcpListener,
    router: Router,
    limits: Limits,
    shutdown: impl Future<Output = ()>,
) {
    let places = Arc::new(Semaphore::new(limits.connections));
    let (stop, stopping) = watch::channel(false);
    let mut shutdown = std::pin::pin!(shutdown);
    let mut warned = false;
    let port = listener.local_addr().map_or(0, |address| address.port());

    loop {
        let place = tokio::select! {
            () = &mut shutdown => break,
            place = places.clone().acquire_owned() => place.expect("never closed"),
        };
        let (stream, peer) = tokio::select! {
            () = &mut shutdown => break,
            accepted = listener.accept() => match accepted {
                Ok(accepted) => accepted,
                Err(err) => {
                    tracing::debug!(error = %err, "cannot accept a connection");
                    tokio::time::sleep(ACCEPT_BACKOFF).await;
                    continue;
                }
            },
        };
        let ip = peer.ip().to_canonical();
        if let Some(peers) = &limits.peers
            && !peers.contains(&ip)
        {
            if warned {
                tracing::debug!(%ip, port, "closed a connection from an untrusted peer");
            } else {
                tracing::warn!(%ip, port, "closed a connection from an untrusted peer; only Home Assistant may connect");
                warned = true;
            }
            continue;
        }

        let service = TowerToHyperService::new(router.clone().map_request(
            move |mut request: hyper::Request<Incoming>| {
                request.extensions_mut().insert(ConnectInfo(peer));
                request
            },
        ));
        let mut builder = http1::Builder::new();
        builder
            .timer(TokioTimer::new())
            .header_read_timeout(limits.header_timeout);
        let mut stopping = stopping.clone();
        tokio::spawn(async move {
            let connection = builder.serve_connection(TokioIo::new(stream), service);
            let mut connection = std::pin::pin!(connection);
            let stopped = async move {
                let _ = stopping.wait_for(|stop| *stop).await;
            };
            let result = tokio::select! {
                result = connection.as_mut() => result,
                () = stopped => {
                    connection.as_mut().graceful_shutdown();
                    connection.await
                }
            };
            if let Err(err) = result {
                tracing::debug!(error = %err, "connection ended with an error");
            }
            drop(place);
        });
    }

    stop.send_replace(true);
    // Each open connection holds a place, so having them all back means
    // every connection closed.
    let all = u32::try_from(limits.connections).unwrap_or(u32::MAX);
    let _ = places.acquire_many(all).await;
}
