#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Au plus 3 clients identifiés en même temps ; la place libérée est réutilisable (audit SEC, constat 6).

mod support;

use std::time::Duration;

use support::{Recv, TestClient, TestDaemon};

/// Le quatrième client identifié est fermé ; trois restent servis
#[tokio::test(flavor = "multi_thread")]
async fn a_fourth_authenticated_client_is_closed() {
    let daemon = TestDaemon::start_closed().await;
    let mut kept = Vec::new();
    for _ in 0..3 {
        let mut client = TestClient::connect(&daemon).await;
        client.hello_ok(daemon.token()).await;
        kept.push(client);
    }
    let mut fourth = TestClient::connect(&daemon).await;
    fourth.hello_ok(daemon.token()).await;
    let outcome = fourth.recv_for(Duration::from_secs(1)).await;
    assert!(matches!(outcome, Recv::Closed), "{outcome:?}");
}

/// Une place libérée par un départ est de nouveau disponible
#[tokio::test(flavor = "multi_thread")]
async fn a_freed_slot_can_be_reused() {
    let daemon = TestDaemon::start_closed().await;
    let mut kept = Vec::new();
    for _ in 0..3 {
        let mut client = TestClient::connect(&daemon).await;
        client.hello_ok(daemon.token()).await;
        kept.push(client);
    }
    kept.pop();
    tokio::time::sleep(Duration::from_millis(300)).await;
    let mut again = TestClient::connect(&daemon).await;
    again.hello_ok(daemon.token()).await;
    let outcome = again.recv_for(Duration::from_millis(500)).await;
    assert!(matches!(outcome, Recv::Quiet), "{outcome:?}");
}
