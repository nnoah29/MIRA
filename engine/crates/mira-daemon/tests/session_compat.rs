#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Compatibilité : demi-fermeture du client, trames inconnues ou superflues, `hello` mal formé.

mod support;

use std::time::Duration;

use mira_protocol::{Frame, HelloRefusal};
use serde_json::json;
use support::{Recv, TestClient, TestDaemon, config};

/// Trame brute : longueur sur quatre octets puis ce JSON.
fn raw(json: &str) -> Vec<u8> {
    let mut bytes = (json.len() as u32).to_be_bytes().to_vec();
    bytes.extend_from_slice(json.as_bytes());
    bytes
}

/// Un client qui ferme son côté écriture après un appel reçoit quand même la réponse.
#[tokio::test(flavor = "multi_thread")]
async fn a_half_closed_client_still_receives_its_results() {
    let home = tempfile::tempdir().unwrap();
    let daemon = TestDaemon::start(config(home.path())).await;
    let mut client = TestClient::connect(&daemon).await;
    client.hello_ok(daemon.token()).await;
    let request = json!({"domain": "account", "request": {"op": "list_accounts"}});
    let id = client.send_call(request).await;
    client.half_close().await;
    let Some(Frame::Result { id: got, envelope }) = client.recv().await else {
        panic!("result attendu")
    };
    assert_eq!(got, id);
    assert_eq!(envelope["ok"], true);
}

/// Une trame de type inconnu (client plus récent) est ignorée : la connexion continue.
#[tokio::test(flavor = "multi_thread")]
async fn an_unknown_frame_type_after_hello_is_ignored() {
    let daemon = TestDaemon::start_closed().await;
    let mut client = TestClient::connect(&daemon).await;
    client.hello_ok(daemon.token()).await;
    client.send_raw(&raw(r#"{"type":"teleport","id":1}"#)).await;
    client.send(Frame::Ping { nonce: 5 }).await;
    assert_eq!(client.recv().await, Some(Frame::Pong { nonce: 5 }));
}

/// Un `pong` du client (réponse à un `ping` qui ne viendra jamais) est ignoré, pas une violation.
#[tokio::test(flavor = "multi_thread")]
async fn a_pong_from_the_client_is_ignored() {
    let daemon = TestDaemon::start_closed().await;
    let mut client = TestClient::connect(&daemon).await;
    client.hello_ok(daemon.token()).await;
    client.send(Frame::Pong { nonce: 1 }).await;
    client.send(Frame::Ping { nonce: 2 }).await;
    assert_eq!(client.recv().await, Some(Frame::Pong { nonce: 2 }));
}

/// Un moyen d'authentification inconnu reçoit `hello_err malformed`, pas une fermeture muette.
#[tokio::test(flavor = "multi_thread")]
async fn an_unknown_auth_type_gets_hello_err_malformed() {
    let daemon = TestDaemon::start_closed().await;
    let mut client = TestClient::connect(&daemon).await;
    let hello = r#"{"type":"hello","protocol_version":{"major":1,"minor":0},"contract_version":{"major":1,"minor":6},"client_id":"c","client_name":"n","auth":{"type":"device_key","key":"k"}}"#;
    client.send_raw(&raw(hello)).await;
    let reply = client.recv().await.unwrap();
    assert_eq!(
        reply,
        Frame::HelloErr {
            reason: HelloRefusal::Malformed
        }
    );
    assert!(matches!(
        client.recv_for(Duration::from_secs(1)).await,
        Recv::Closed
    ));
}

/// Un `hello` sans `type` reçoit `hello_err malformed`.
#[tokio::test(flavor = "multi_thread")]
async fn a_hello_that_is_not_a_frame_gets_hello_err_malformed() {
    let daemon = TestDaemon::start_closed().await;
    let mut client = TestClient::connect(&daemon).await;
    client.send_raw(&raw(r#"{"foo":1}"#)).await;
    let reply = client.recv().await.unwrap();
    assert_eq!(
        reply,
        Frame::HelloErr {
            reason: HelloRefusal::Malformed
        }
    );
}
