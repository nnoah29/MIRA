#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Trames hors limites : trop grande ou coupée ; le démon continue de servir les autres clients.

mod support;

use std::time::Duration;

use support::{TestClient, TestDaemon};

/// Une trame annoncée au-delà du maximum ferme la connexion ; un autre client est servi ensuite.
#[tokio::test(flavor = "multi_thread")]
async fn an_oversize_frame_closes_the_connection_and_the_daemon_keeps_serving() {
    let daemon = TestDaemon::start_with(None, |ctx| ctx.max_frame = 1024).await;
    let mut client = TestClient::connect(&daemon).await;
    client.hello_ok(daemon.token()).await;
    client.send_raw(&(1024u32 * 1024).to_be_bytes()).await;
    assert!(client.recv().await.is_none());
    let mut other = TestClient::connect(&daemon).await;
    other.hello_ok(daemon.token()).await;
}

/// Une trame coupée ferme seulement sa connexion.
#[tokio::test(flavor = "multi_thread")]
async fn a_truncated_frame_closes_only_that_connection() {
    let daemon = TestDaemon::start_closed().await;
    let mut client = TestClient::connect(&daemon).await;
    client.hello_ok(daemon.token()).await;
    client.send_raw(&[0, 0, 0, 40]).await;
    client.send_raw(b"{\"type\":\"ca").await;
    drop(client);
    tokio::time::sleep(Duration::from_millis(100)).await;
    let mut other = TestClient::connect(&daemon).await;
    other.hello_ok(daemon.token()).await;
}
