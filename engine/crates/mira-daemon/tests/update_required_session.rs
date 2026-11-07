#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Session d'un client plus récent que le moteur : seuls `shutdown`, `ping` et `pong` sont permis.

mod support;

use mira_assembly::mira_vocabulary::{CONTRACT_VERSION, ContractVersion};
use mira_protocol::{Frame, HostOp, PROTOCOL_VERSION};
use serde_json::json;
use support::{TestClient, TestDaemon, hello_with};

/// Client attaché avec un contrat plus récent que celui du moteur.
async fn newer_client(daemon: &TestDaemon) -> TestClient {
    let mut client = TestClient::connect(daemon).await;
    let contract = ContractVersion {
        major: CONTRACT_VERSION.major,
        minor: CONTRACT_VERSION.minor + 1,
    };
    client
        .send(hello_with(daemon.token(), PROTOCOL_VERSION, contract))
        .await;
    assert!(matches!(client.recv().await, Some(Frame::HelloOk { .. })));
    client
}

/// Un appel du contrat ferme la connexion : le moteur ne sert pas un client qu'il ne comprend pas.
#[tokio::test(flavor = "multi_thread")]
async fn a_call_closes_the_restricted_session() {
    let daemon = TestDaemon::start_closed().await;
    let mut client = newer_client(&daemon).await;
    let request = json!({"domain": "engine", "request": {"op": "get_contract_info"}});
    client.send_call(request).await;
    assert!(client.recv().await.is_none());
}

/// `shutdown` répond ok et arrête le démon : la voie de sortie de la mise à jour.
#[tokio::test(flavor = "multi_thread")]
async fn shutdown_is_allowed_and_stops_the_daemon() {
    let mut daemon = TestDaemon::start_closed().await;
    let mut client = newer_client(&daemon).await;
    let op = HostOp::Shutdown;
    client.send(Frame::HostOp { id: 1, op }).await;
    let Some(Frame::Result { id: 1, envelope }) = client.recv().await else {
        panic!("result attendu")
    };
    assert_eq!(envelope["ok"], true);
    let stopped = tokio::time::timeout(std::time::Duration::from_secs(10), daemon.task()).await;
    assert!(stopped.is_ok());
}

/// `ping` reste permis, et `retry_unlock` ne l'est pas.
#[tokio::test(flavor = "multi_thread")]
async fn ping_is_allowed_but_retry_unlock_is_not() {
    let daemon = TestDaemon::start_closed().await;
    let mut client = newer_client(&daemon).await;
    client.send(Frame::Ping { nonce: 9 }).await;
    assert_eq!(client.recv().await, Some(Frame::Pong { nonce: 9 }));
    let op = HostOp::RetryUnlock;
    client.send(Frame::HostOp { id: 2, op }).await;
    assert!(client.recv().await.is_none());
}
