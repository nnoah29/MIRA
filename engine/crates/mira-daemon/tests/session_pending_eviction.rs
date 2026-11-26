#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Place manquante pour un `hello` : la plus ancienne connexion en attente est évincée, la nouvelle est servie.

mod support;

use std::sync::Arc;
use std::time::Duration;

use support::{Recv, TestDaemon};
use tokio::sync::Semaphore;

/// Démarre un démon qui n'accepte que deux connexions en attente de `hello`.
async fn daemon_with_two_places() -> TestDaemon {
    TestDaemon::start_with(None, |ctx| ctx.pending_hello = Arc::new(Semaphore::new(2))).await
}

/// Quand la place manque, la plus ancienne connexion muette est fermée et la nouvelle passe son `hello`
#[tokio::test(flavor = "multi_thread")]
async fn the_oldest_silent_connection_is_evicted_for_a_newcomer() {
    let daemon = daemon_with_two_places().await;
    let mut oldest = daemon.connect_registered().await;
    let mut second = daemon.connect_registered().await;
    let mut newcomer = daemon.connect_registered().await;
    newcomer.hello_ok(daemon.token()).await;
    let outcome = oldest.recv_for(Duration::from_secs(1)).await;
    assert!(matches!(outcome, Recv::Closed), "{outcome:?}");
    let still_waiting = second.recv_for(Duration::from_millis(200)).await;
    assert!(!matches!(still_waiting, Recv::Closed), "{still_waiting:?}");
}

/// Un programme qui rouvre sans cesse des connexions muettes n'empêche pas un client légitime de se connecter
#[tokio::test(flavor = "multi_thread")]
async fn a_flood_of_silent_connections_cannot_lock_out_a_real_client() {
    let daemon = daemon_with_two_places().await;
    let mut flood = Vec::new();
    for _ in 0..6 {
        flood.push(daemon.connect_registered().await);
    }
    let mut real = daemon.connect_registered().await;
    real.hello_ok(daemon.token()).await;
}

/// Les places des connexions évincées sont rendues : sur cinq connexions pour deux places, les trois plus anciennes sont fermées
#[tokio::test(flavor = "multi_thread")]
async fn evicted_connections_give_their_places_back() {
    let daemon = daemon_with_two_places().await;
    let mut all = Vec::new();
    for _ in 0..5 {
        all.push(daemon.connect_registered().await);
    }
    let mut closed = Vec::new();
    for client in all.iter_mut() {
        let outcome = client.recv_for(Duration::from_millis(300)).await;
        closed.push(matches!(outcome, Recv::Closed));
    }
    assert_eq!(closed, [true, true, true, false, false]);
}
