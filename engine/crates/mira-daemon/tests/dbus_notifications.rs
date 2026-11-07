#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Notifications du bureau par D-Bus : affichage, clic qui relance l'app, service absent.

mod support;

use std::sync::atomic::Ordering;
use std::time::Duration;

use mira_daemon::desktop::DbusNotificationSink;
use mira_host::notifier::NotificationSink;
use support::dbus_probe::{counting, sample, until};
use support::fake_notification_server::{click, serve};
use support::private_bus::PrivateBus;

/// Le texte, l'action « Ouvrir » et le son arrivent au service de notifications.
#[tokio::test(flavor = "multi_thread")]
async fn a_notification_reaches_the_notification_service() {
    let bus = PrivateBus::start();
    let (_connection, seen) = serve(&bus.address).await;
    let (_count, launch) = counting();
    let sink = DbusNotificationSink::connect_to(&bus.address, launch).unwrap();
    sink.show(sample(true));
    assert!(until(|| !seen.lock().unwrap().is_empty()).await);
    let first = seen.lock().unwrap()[0].clone();
    assert_eq!(
        (first.summary.as_str(), first.body.as_str()),
        ("Bob", "Rendez-vous demain")
    );
    assert!(first.actions.contains(&"default".to_string()));
    assert!(
        first.hint_keys.contains(&"sound-name".to_string()),
        "{:?}",
        first.hint_keys
    );
}

/// Sans son demandé, le service reçoit « suppress-sound » et non un nom de son.
#[tokio::test(flavor = "multi_thread")]
async fn a_silent_notification_asks_to_suppress_the_sound() {
    let bus = PrivateBus::start();
    let (_connection, seen) = serve(&bus.address).await;
    let (_count, launch) = counting();
    let sink = DbusNotificationSink::connect_to(&bus.address, launch).unwrap();
    sink.show(sample(false));
    assert!(until(|| !seen.lock().unwrap().is_empty()).await);
    let keys = seen.lock().unwrap()[0].hint_keys.clone();
    assert!(
        keys.contains(&"suppress-sound".to_string()) && !keys.contains(&"sound-name".to_string())
    );
}

/// Un clic sur notre notification relance l'app ; un clic sur une autre notification, non.
#[tokio::test(flavor = "multi_thread")]
async fn a_click_on_our_notification_launches_the_app_but_not_on_another() {
    let bus = PrivateBus::start();
    let (connection, seen) = serve(&bus.address).await;
    let (count, launch) = counting();
    let sink = DbusNotificationSink::connect_to(&bus.address, launch).unwrap();
    sink.show(sample(true));
    assert!(until(|| !seen.lock().unwrap().is_empty()).await);
    let id = seen.lock().unwrap()[0].id;
    click(&connection, id + 100, "default").await;
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert_eq!(count.load(Ordering::SeqCst), 0);
    click(&connection, id, "default").await;
    assert!(until(|| count.load(Ordering::SeqCst) == 1).await);
}

/// Sans service de notifications, afficher ne fait rien et ne plante pas.
#[tokio::test(flavor = "multi_thread")]
async fn a_missing_notification_service_is_silent() {
    let bus = PrivateBus::start();
    let (_count, launch) = counting();
    let sink = DbusNotificationSink::connect_to(&bus.address, launch).unwrap();
    sink.show(sample(true));
    tokio::time::sleep(Duration::from_millis(300)).await;
}
