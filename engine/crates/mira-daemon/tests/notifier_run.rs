#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Le notifieur sur un vrai moteur : un message livré après la synchronisation arrive au récepteur.

mod support;

use std::time::Duration;

use mira_host::notifier::{Texts, run_notifier};
use support::notifier_probe::{deliver, synced_daemon, wait_for};
use support::notifier_sink::probes;

const WINDOW: Duration = Duration::from_millis(100);
const ARRIVAL: Duration = Duration::from_secs(15);

/// Un message livré après la synchronisation donne une notification avec son objet.
#[tokio::test(flavor = "multi_thread")]
async fn a_new_message_reaches_the_sink_with_its_subject() {
    let (daemon, mut client, _account) = synced_daemon().await;
    let (sink, clock) = probes(12);
    let host = daemon.daemon.context().host.clone();
    tokio::spawn(run_notifier(
        host,
        sink.clone(),
        clock,
        Texts::for_locale("fr"),
        WINDOW,
    ));
    tokio::time::sleep(Duration::from_millis(300)).await;
    deliver(&daemon, &mut client, "Rendez-vous demain").await;
    assert!(wait_for(&sink, 1, ARRIVAL).await);
    assert_eq!(sink.shown()[0].body, "Rendez-vous demain");
}

/// Quand un client regarde MIRA, le message n'est pas notifié ; il l'est quand le client part.
#[tokio::test(flavor = "multi_thread")]
async fn an_active_client_blocks_the_notification_until_it_leaves() {
    let (daemon, mut client, _account) = synced_daemon().await;
    let (sink, clock) = probes(12);
    let host = daemon.daemon.context().host.clone();
    host.presence().set("fenêtre", true);
    tokio::spawn(run_notifier(
        host.clone(),
        sink.clone(),
        clock,
        Texts::for_locale("fr"),
        WINDOW,
    ));
    tokio::time::sleep(Duration::from_millis(300)).await;
    deliver(&daemon, &mut client, "Pendant que vous regardez").await;
    assert!(!wait_for(&sink, 1, Duration::from_secs(3)).await);
    host.presence().remove("fenêtre");
    deliver(&daemon, &mut client, "Après votre départ").await;
    assert!(wait_for(&sink, 1, ARRIVAL).await);
    assert_eq!(sink.shown()[0].body, "Après votre départ");
}

/// Un compte en sourdine n'est pas notifié.
#[tokio::test(flavor = "multi_thread")]
async fn a_muted_account_is_not_notified() {
    let (daemon, mut client, account) = synced_daemon().await;
    let (sink, clock) = probes(12);
    let set = serde_json::json!({"domain": "settings", "request": {"op": "set_settings",
        "settings": {"poll_secs": null, "muted_accounts": [account], "quiet_hours": null}}});
    assert_eq!(client.call(set).await["ok"], true);
    let host = daemon.daemon.context().host.clone();
    tokio::spawn(run_notifier(
        host,
        sink.clone(),
        clock,
        Texts::for_locale("fr"),
        WINDOW,
    ));
    tokio::time::sleep(Duration::from_millis(300)).await;
    deliver(&daemon, &mut client, "Compte en sourdine").await;
    assert!(!wait_for(&sink, 1, Duration::from_secs(3)).await);
}

/// La fermeture du moteur termine le notifieur et libère le moteur.
#[tokio::test(flavor = "multi_thread")]
async fn closing_the_engine_ends_the_notifier() {
    let (daemon, client, _account) = synced_daemon().await;
    drop(client);
    let (sink, clock) = probes(12);
    let host = daemon.daemon.context().host.clone();
    let task = tokio::spawn(run_notifier(
        host,
        sink,
        clock,
        Texts::for_locale("fr"),
        WINDOW,
    ));
    tokio::time::sleep(Duration::from_millis(300)).await;
    let closed = mira_daemon::shutdown::run(daemon.daemon.context()).await;
    assert!(closed.contains("\"ok\":true"), "{closed}");
    assert!(
        tokio::time::timeout(Duration::from_secs(10), task)
            .await
            .is_ok()
    );
}
