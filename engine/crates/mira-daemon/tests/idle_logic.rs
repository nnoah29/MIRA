#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Attente d'inactivité : le démon sans client depuis la durée de grâce rend la main, un client qui revient à temps l'en empêche.

use std::time::{Duration, Instant};

use mira_daemon::idle::wait_until_idle_for;
use mira_daemon::server::Clients;
use tokio::time::{sleep, timeout};

const GRACE: Duration = Duration::from_millis(300);

/// Sans aucun client, l'attente se termine après la grâce.
#[tokio::test]
async fn returns_after_the_grace_when_nobody_is_attached() {
    let clients = Clients::default();
    let started = Instant::now();
    timeout(Duration::from_secs(5), wait_until_idle_for(&clients, GRACE))
        .await
        .unwrap();
    assert!(started.elapsed() >= GRACE);
}

/// Tant qu'un client est attaché, l'attente ne se termine pas.
#[tokio::test]
async fn does_not_return_while_a_client_is_attached() {
    let clients = Clients::default();
    let _guard = clients.attach();
    let waiting = timeout(2 * GRACE, wait_until_idle_for(&clients, GRACE)).await;
    assert!(waiting.is_err());
}

/// Après le départ du dernier client, la grâce complète est attendue.
#[tokio::test]
async fn returns_one_grace_after_the_last_client_leaves() {
    let clients = Clients::default();
    let guard = clients.attach();
    let started = Instant::now();
    tokio::spawn(async move {
        sleep(Duration::from_millis(150)).await;
        drop(guard);
    });
    timeout(Duration::from_secs(5), wait_until_idle_for(&clients, GRACE))
        .await
        .unwrap();
    assert!(started.elapsed() >= Duration::from_millis(150) + GRACE - Duration::from_millis(30));
}

/// Un client qui arrive pendant la grâce remet l'attente à zéro.
#[tokio::test]
async fn a_client_arriving_within_the_grace_resets_the_wait() {
    let clients = Clients::default();
    let probe = clients.clone();
    let waiting = tokio::spawn(async move { wait_until_idle_for(&probe, GRACE).await });
    sleep(Duration::from_millis(150)).await;
    let guard = clients.attach();
    sleep(2 * GRACE).await;
    assert!(!waiting.is_finished());
    drop(guard);
    timeout(Duration::from_secs(5), waiting)
        .await
        .unwrap()
        .unwrap();
}
