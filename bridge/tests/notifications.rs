#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Notifications du moteur embarqué : forme JSON remise à Dart, fin du flux quand le moteur n'est pas ouvert.

use mira_bridge::notifications::{notification_json, pump};
use mira_host::notifier::Notification;

/// La notification part en JSON plat, sans règle à appliquer côté Dart.
#[test]
fn a_notification_is_sent_to_dart_as_flat_json() {
    let json = notification_json(&Notification {
        id: 7,
        title: "Bob".into(),
        body: "Rendez-vous".into(),
        sound: false,
        message: Some("a1/m1".into()),
    });
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(
        value,
        serde_json::json!({"id":7,"title":"Bob","body":"Rendez-vous","sound":false,"message":"a1/m1"})
    );
}

/// Sans moteur ouvert, le flux se termine aussitôt et ne livre rien.
#[tokio::test]
async fn the_stream_ends_at_once_without_an_open_engine() {
    let mut delivered = 0;
    pump("fr", |_| {
        delivered += 1;
        true
    })
    .await;
    assert_eq!(delivered, 0);
}
