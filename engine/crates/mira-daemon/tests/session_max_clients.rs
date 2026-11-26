#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Au plus 3 clients identifiés en même temps ; le suivant est refusé par `hello_err busy` ; la place libérée est réutilisable (audit SEC, constat 6, SEC-06).

mod support;

use std::time::Duration;

use mira_protocol::{Frame, HelloRefusal};
use support::{Recv, TestClient, TestDaemon, valid_hello};

/// Le quatrième client identifié reçoit `hello_err busy` puis est fermé ; il ne se croit jamais connecté
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
    fourth.send(valid_hello(daemon.token())).await;
    let reply = fourth.recv().await.unwrap();
    assert_eq!(
        reply,
        Frame::HelloErr {
            reason: HelloRefusal::Busy
        }
    );
    assert!(fourth.recv().await.is_none());
}

/// Un quatrième client au mauvais jeton reçoit `auth`, jamais `busy` : un inconnu n'apprend pas que le démon est plein
#[tokio::test(flavor = "multi_thread")]
async fn a_wrong_token_never_learns_the_daemon_is_full() {
    let daemon = TestDaemon::start_closed().await;
    let mut kept = Vec::new();
    for _ in 0..3 {
        let mut client = TestClient::connect(&daemon).await;
        client.hello_ok(daemon.token()).await;
        kept.push(client);
    }
    let mut stranger = TestClient::connect(&daemon).await;
    stranger.send(valid_hello(&"0".repeat(64))).await;
    let reply = stranger.recv().await.unwrap();
    assert_eq!(
        reply,
        Frame::HelloErr {
            reason: HelloRefusal::Auth
        }
    );
}

/// Les trois premiers clients restent servis quand le quatrième est refusé
#[tokio::test(flavor = "multi_thread")]
async fn the_three_first_clients_stay_served() {
    let daemon = TestDaemon::start_closed().await;
    let mut kept = Vec::new();
    for _ in 0..3 {
        let mut client = TestClient::connect(&daemon).await;
        client.hello_ok(daemon.token()).await;
        kept.push(client);
    }
    let mut fourth = TestClient::connect(&daemon).await;
    fourth.send(valid_hello(daemon.token())).await;
    fourth.recv().await.unwrap();
    for client in &mut kept {
        let outcome = client.recv_for(Duration::from_millis(200)).await;
        assert!(matches!(outcome, Recv::Quiet), "{outcome:?}");
    }
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
