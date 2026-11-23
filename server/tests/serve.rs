//! The limits the server keeps on connections.

use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use axum::extract::ConnectInfo;
use axum::routing::get;
use memory_card::serve::{Limits, serve};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Notify;

const REQUEST: &[u8] = b"GET / HTTP/1.1\r\nHost: app\r\n\r\n";

async fn start(limits: Limits) -> (SocketAddr, Arc<Notify>, tokio::task::JoinHandle<()>) {
    let router = Router::new().route(
        "/",
        get(|ConnectInfo(peer): ConnectInfo<SocketAddr>| async move { peer.ip().to_string() }),
    );
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let stop = Arc::new(Notify::new());
    let server = tokio::spawn({
        let stop = stop.clone();
        async move {
            serve(
                listener,
                router,
                limits,
                async move { stop.notified().await },
            )
            .await
        }
    });
    (address, stop, server)
}

fn limits(connections: usize, header_timeout: Duration) -> Limits {
    Limits {
        connections,
        header_timeout,
        peers: None,
    }
}

/// What the server sends before it closes, or `None` if it is still open after `wait`.
async fn read_until_closed(stream: &mut TcpStream, wait: Duration) -> Option<String> {
    let mut received = Vec::new();
    tokio::time::timeout(wait, async {
        let mut buffer = [0u8; 1024];
        loop {
            match stream.read(&mut buffer).await {
                Ok(0) | Err(_) => break,
                Ok(n) => received.extend_from_slice(&buffer[..n]),
            }
        }
    })
    .await
    .ok()?;
    Some(String::from_utf8_lossy(&received).into_owned())
}

async fn response(stream: &mut TcpStream) -> String {
    let mut buffer = vec![0u8; 1024];
    let n = tokio::time::timeout(Duration::from_secs(2), stream.read(&mut buffer))
        .await
        .expect("an answer in time")
        .unwrap();
    String::from_utf8_lossy(&buffer[..n]).into_owned()
}

#[tokio::test]
async fn requests_reach_the_router_with_the_peer_address() {
    let (address, _stop, _server) = start(limits(4, Duration::from_secs(5))).await;
    let mut stream = TcpStream::connect(address).await.unwrap();
    stream.write_all(REQUEST).await.unwrap();
    let answer = response(&mut stream).await;
    assert!(answer.starts_with("HTTP/1.1 200"), "{answer}");
    assert!(answer.ends_with("127.0.0.1"), "{answer}");
}

#[tokio::test]
async fn untrusted_peers_are_closed_unanswered() {
    let other: IpAddr = "172.30.32.2".parse().unwrap();
    let (address, _stop, _server) = start(Limits {
        peers: Some(Arc::from([other])),
        ..limits(4, Duration::from_secs(5))
    })
    .await;
    let mut stream = TcpStream::connect(address).await.unwrap();
    let _ = stream.write_all(REQUEST).await;
    let sent = read_until_closed(&mut stream, Duration::from_secs(2)).await;
    assert_eq!(sent.as_deref(), Some(""), "closed without an answer");
}

#[tokio::test]
async fn connections_that_never_send_headers_are_closed() {
    let (address, _stop, _server) = start(limits(4, Duration::from_millis(200))).await;
    let mut silent = TcpStream::connect(address).await.unwrap();
    assert!(
        read_until_closed(&mut silent, Duration::from_secs(3))
            .await
            .is_some()
    );

    let mut partial = TcpStream::connect(address).await.unwrap();
    partial.write_all(b"GET / HTTP/1.1\r\nHost:").await.unwrap();
    let sent = read_until_closed(&mut partial, Duration::from_secs(3)).await;
    assert!(sent.is_some(), "a half-sent request is closed too");

    let mut idle = TcpStream::connect(address).await.unwrap();
    idle.write_all(REQUEST).await.unwrap();
    assert!(response(&mut idle).await.starts_with("HTTP/1.1 200"));
    let after = read_until_closed(&mut idle, Duration::from_secs(3)).await;
    assert_eq!(
        after.as_deref(),
        Some(""),
        "an idle kept-alive connection closes"
    );
}

#[tokio::test]
async fn connections_past_the_cap_wait_for_a_place() {
    let (address, _stop, _server) = start(limits(2, Duration::from_secs(10))).await;
    let first = TcpStream::connect(address).await.unwrap();
    let _second = TcpStream::connect(address).await.unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;

    let mut third = TcpStream::connect(address).await.unwrap();
    third.write_all(REQUEST).await.unwrap();
    let mut buffer = [0u8; 64];
    let early = tokio::time::timeout(Duration::from_millis(300), third.read(&mut buffer)).await;
    assert!(early.is_err(), "no answer while both places are taken");

    drop(first);
    assert!(response(&mut third).await.starts_with("HTTP/1.1 200"));
}

#[tokio::test]
async fn stopping_waits_for_open_answers_then_returns() {
    let (address, stop, server) = start(limits(4, Duration::from_secs(10))).await;
    let mut stream = TcpStream::connect(address).await.unwrap();
    stream.write_all(REQUEST).await.unwrap();
    assert!(response(&mut stream).await.starts_with("HTTP/1.1 200"));

    stop.notify_one();
    tokio::time::timeout(Duration::from_secs(2), server)
        .await
        .expect("the server stops once its connections close")
        .unwrap();
    assert!(
        TcpStream::connect(address).await.is_err(),
        "no longer listening"
    );
}
