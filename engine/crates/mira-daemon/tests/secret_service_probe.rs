#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Sonde du trousseau par le Secret Service : lit l'état « verrouillé » de la collection par défaut.

mod support;

use std::sync::atomic::Ordering;

use mira_daemon::secret_service::SecretServiceProbe;
use mira_daemon::unlock::KeyringProbe;
use support::fake_secret_service::serve_secrets;
use support::private_bus::PrivateBus;

/// Sonde sur le bus privé, lancée hors du fil asynchrone (l'appel D-Bus est bloquant).
async fn locked_on(bus: &PrivateBus) -> bool {
    let address = bus.address.clone();
    tokio::task::spawn_blocking(move || SecretServiceProbe::connect_to(&address).locked())
        .await
        .unwrap()
}

/// Une collection verrouillée est vue verrouillée, puis ouverte une fois déverrouillée.
#[tokio::test(flavor = "multi_thread")]
async fn a_locked_collection_is_seen_locked_then_unlocked() {
    let bus = PrivateBus::start();
    let (_connection, flag) = serve_secrets(&bus.address, true, true).await;
    assert!(locked_on(&bus).await);
    flag.store(false, Ordering::SeqCst);
    assert!(!locked_on(&bus).await);
}

/// Sans Secret Service sur le bus, rien n'est à déverrouiller.
#[tokio::test(flavor = "multi_thread")]
async fn without_a_secret_service_nothing_is_locked() {
    let bus = PrivateBus::start();
    assert!(!locked_on(&bus).await);
}

/// Sans collection par défaut (alias vide), rien n'est à déverrouiller.
#[tokio::test(flavor = "multi_thread")]
async fn without_a_default_collection_nothing_is_locked() {
    let bus = PrivateBus::start();
    let (_connection, _flag) = serve_secrets(&bus.address, true, false).await;
    assert!(!locked_on(&bus).await);
}
