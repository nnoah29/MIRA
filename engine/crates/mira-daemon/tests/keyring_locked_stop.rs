#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Trousseau verrouillé : l'arrêt demandé interrompt l'attente, et `retry_unlock` arrive par la socket.

mod support;

use std::sync::Arc;
use std::time::Duration;

use mira_daemon::unlock::{UnlockBackoff, open_when_unlocked};
use mira_protocol::{Frame, HostOp, HostState};
use support::engine_config::config;
use support::locked_store::LockedStore;
use support::unlock_harness::{context, wait_state};
use support::{TestClient, TestDaemon};

/// Un arrêt demandé pendant l'attente la termine sans ouvrir le moteur.
#[tokio::test(flavor = "multi_thread")]
async fn a_shutdown_during_the_wait_ends_it_without_opening_the_engine() {
    let dir = tempfile::tempdir().unwrap();
    let ctx = context(dir.path());
    let mut engine = config(dir.path());
    engine.secrets = Arc::new(LockedStore::locked_for(usize::MAX));
    let long = UnlockBackoff {
        first: Duration::from_secs(60),
        max: Duration::from_secs(60),
    };
    let task = tokio::spawn({
        let ctx = ctx.clone();
        async move { open_when_unlocked(&ctx, engine, long).await }
    });
    wait_state(&ctx, HostState::KeyringLocked).await;
    ctx.begin_shutdown();
    let ended = tokio::time::timeout(Duration::from_secs(10), task).await;
    assert_eq!(ended.unwrap().unwrap(), None);
    assert!(ctx.host.engine().is_none());
}

/// `retry_unlock` reçu par la socket répond ok et réveille l'attente.
#[tokio::test(flavor = "multi_thread")]
async fn retry_unlock_over_the_socket_answers_ok_and_wakes_the_wait() {
    let daemon = TestDaemon::start_closed().await;
    let mut client = TestClient::connect(&daemon).await;
    client.hello_ok(daemon.token()).await;
    let op = HostOp::RetryUnlock;
    client.send(Frame::HostOp { id: 3, op }).await;
    let Some(Frame::Result { id: 3, envelope }) = client.recv().await else {
        panic!("result attendu")
    };
    assert_eq!(envelope["ok"], true);
    let ctx = daemon.context();
    let woken = ctx.retry_unlock.notified();
    let waited = tokio::time::timeout(Duration::from_secs(1), woken).await;
    assert!(waited.is_ok());
}
