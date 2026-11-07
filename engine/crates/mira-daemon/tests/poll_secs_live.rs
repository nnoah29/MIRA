#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Le relevé choisi par l'app passe par les réglages du moteur : enregistré et annoncé, sans opération d'hôte.

mod support;

use mira_protocol::Frame;
use serde_json::json;
use support::event_helpers::attached;

/// Requête `set_settings` fixant le relevé périodique.
fn set_poll(secs: u32) -> serde_json::Value {
    json!({"domain": "settings", "request": {"op": "set_settings",
        "settings": {"poll_secs": secs, "muted_accounts": [], "quiet_hours": null}}})
}

/// Un relevé fixé par la socket est relu tel quel par `get_settings`.
#[tokio::test(flavor = "multi_thread")]
async fn poll_secs_set_through_the_socket_is_stored() {
    let (_daemon, mut client, _home) = attached(|_| {}).await;
    assert_eq!(client.call(set_poll(60)).await["ok"], true);
    let read = client
        .call(json!({"domain": "settings", "request": {"op": "get_settings"}}))
        .await;
    assert_eq!(read["data"]["data"]["data"]["poll_secs"], 60, "{read}");
}

/// Le changement est annoncé aux abonnés, sans redémarrage du démon.
#[tokio::test(flavor = "multi_thread")]
async fn poll_secs_change_is_announced_to_event_subscribers() {
    let (_daemon, mut client, _home) = attached(|_| {}).await;
    client.open_events(5, None, 8).await;
    assert_eq!(client.call(set_poll(120)).await["ok"], true);
    let frame = client.recv_stream(5).await.unwrap();
    let Frame::Event { event, .. } = frame else {
        panic!("event attendu : {frame:?}")
    };
    assert!(event.to_string().contains("poll_secs"), "{event}");
}
