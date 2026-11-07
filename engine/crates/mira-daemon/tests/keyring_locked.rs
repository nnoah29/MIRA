#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Trousseau verrouillé : annoncé, réessayé avec un délai croissant ou sur demande, puis moteur ouvert.

mod support;

use std::sync::Arc;
use std::time::Duration;

use mira_daemon::unlock::{UnlockBackoff, open_when_unlocked};
use mira_protocol::HostState;
use support::engine_config::config;
use support::locked_store::LockedStore;
use support::unlock_harness::{context, wait_state};

const SHORT: UnlockBackoff = UnlockBackoff {
    first: Duration::from_millis(30),
    max: Duration::from_millis(60),
};
const LONG: UnlockBackoff = UnlockBackoff {
    first: Duration::from_secs(60),
    max: Duration::from_secs(60),
};

/// Verrouillé deux fois, le trousseau est annoncé puis le moteur s'ouvre au troisième essai.
#[tokio::test(flavor = "multi_thread")]
async fn a_locked_keyring_is_announced_then_the_engine_opens_after_unlock() {
    let dir = tempfile::tempdir().unwrap();
    let ctx = context(dir.path());
    let store = Arc::new(LockedStore::locked_for(2));
    let mut engine = config(dir.path());
    engine.secrets = store.clone();
    let task = tokio::spawn({
        let ctx = ctx.clone();
        async move { open_when_unlocked(&ctx, engine, SHORT).await }
    });
    wait_state(&ctx, HostState::KeyringLocked).await;
    let opened = task.await.unwrap().unwrap();
    assert!(opened.contains("\"ok\":true"), "{opened}");
    assert_eq!(ctx.host_state(), HostState::Starting);
    assert_eq!(store.probes(), 3);
}

/// `retry_unlock` interrompt l'attente : pas besoin d'attendre le délai.
#[tokio::test(flavor = "multi_thread")]
async fn retry_unlock_skips_the_wait() {
    let dir = tempfile::tempdir().unwrap();
    let ctx = context(dir.path());
    let store = Arc::new(LockedStore::locked_for(1));
    let mut engine = config(dir.path());
    engine.secrets = store.clone();
    let task = tokio::spawn({
        let ctx = ctx.clone();
        async move { open_when_unlocked(&ctx, engine, LONG).await }
    });
    wait_state(&ctx, HostState::KeyringLocked).await;
    ctx.retry_unlock.notify_one();
    let opened = tokio::time::timeout(Duration::from_secs(10), task).await;
    assert!(opened.unwrap().unwrap().unwrap().contains("\"ok\":true"));
}

/// Un trousseau déjà ouvert n'est jamais annoncé verrouillé : une seule sonde, moteur ouvert.
#[tokio::test(flavor = "multi_thread")]
async fn an_unlocked_keyring_opens_at_once() {
    let dir = tempfile::tempdir().unwrap();
    let ctx = context(dir.path());
    let store = Arc::new(LockedStore::locked_for(0));
    let mut engine = config(dir.path());
    engine.secrets = store.clone();
    let opened = open_when_unlocked(&ctx, engine, SHORT).await.unwrap();
    assert!(opened.contains("\"ok\":true"), "{opened}");
    assert_eq!(store.probes(), 1);
    assert_eq!(ctx.host_state(), HostState::Starting);
}
