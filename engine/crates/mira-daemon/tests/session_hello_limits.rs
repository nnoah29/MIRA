#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Avant `hello` : une trame d'un autre type, un second `hello`, le silence et la lenteur ferment la connexion.

mod support;

use std::time::Duration;

use mira_protocol::{Frame, MAX_FRAME_BYTES, encode};
use serde_json::json;
use support::{Recv, TestClient, TestDaemon, valid_hello};

/// Une trame autre que `hello` en premier ferme la connexion sans réponse.
#[tokio::test(flavor = "multi_thread")]
async fn a_frame_before_hello_closes_the_connection() {
    let daemon = TestDaemon::start_closed().await;
    let mut client = TestClient::connect(&daemon).await;
    client
        .send(Frame::Call {
            id: 1,
            request: json!({}),
        })
        .await;
    assert!(client.recv().await.is_none());
}

/// Un second `hello` ferme la connexion.
#[tokio::test(flavor = "multi_thread")]
async fn a_second_hello_closes_the_connection() {
    let daemon = TestDaemon::start_closed().await;
    let mut client = TestClient::connect(&daemon).await;
    client.hello_ok(daemon.token()).await;
    client.send(valid_hello(daemon.token())).await;
    assert!(client.recv().await.is_none());
}

/// Une connexion muette est fermée au délai de `hello`.
#[tokio::test(flavor = "multi_thread")]
async fn a_silent_connection_is_closed_after_the_hello_timeout() {
    let daemon =
        TestDaemon::start_with(None, |ctx| ctx.hello_timeout = Duration::from_millis(200)).await;
    let mut client = TestClient::connect(&daemon).await;
    let outcome = client.recv_for(Duration::from_secs(2)).await;
    assert!(matches!(outcome, Recv::Closed), "{outcome:?}");
}

/// Des octets goutte à goutte ne repoussent pas le délai : la lecture de `hello` est bornée en bloc.
#[tokio::test(flavor = "multi_thread")]
async fn bytes_trickled_slower_than_the_timeout_are_closed() {
    let daemon =
        TestDaemon::start_with(None, |ctx| ctx.hello_timeout = Duration::from_millis(300)).await;
    let mut client = TestClient::connect(&daemon).await;
    let bytes = encode(&valid_hello(daemon.token()), MAX_FRAME_BYTES).unwrap();
    let mut closed = false;
    for byte in bytes.iter().take(30) {
        client.send_raw(&[*byte]).await;
        if matches!(
            client.recv_for(Duration::from_millis(100)).await,
            Recv::Closed
        ) {
            closed = true;
            break;
        }
    }
    assert!(
        closed,
        "la connexion lente doit être fermée avant la fin du hello"
    );
}
