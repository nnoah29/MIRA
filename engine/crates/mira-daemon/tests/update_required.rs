#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Moteur périmé : un client plus récent reçoit `update_required` ; le jeton passe toujours avant les versions.

mod support;

use mira_assembly::mira_vocabulary::{CONTRACT_VERSION, ContractVersion};
use mira_protocol::{Frame, HelloRefusal, HostState, PROTOCOL_VERSION};
use support::{TestClient, TestDaemon, hello_with};

/// Connexion fraîche qui envoie un `hello` au contrat donné avec ce jeton.
async fn hello_at(daemon: &TestDaemon, token: &str, major: u32, minor: u32) -> Frame {
    let mut client = TestClient::connect(daemon).await;
    let contract = ContractVersion { major, minor };
    client
        .send(hello_with(token, PROTOCOL_VERSION, contract))
        .await;
    client.recv().await.unwrap()
}

/// Mineure supérieure : `hello_ok` avec l'état `update_required`.
#[tokio::test(flavor = "multi_thread")]
async fn a_newer_minor_gets_hello_ok_update_required() {
    let daemon = TestDaemon::start_closed().await;
    let (major, minor) = (CONTRACT_VERSION.major, CONTRACT_VERSION.minor + 1);
    let reply = hello_at(&daemon, daemon.token(), major, minor).await;
    let Frame::HelloOk { host_state, .. } = reply else {
        panic!("hello_ok attendu : {reply:?}")
    };
    assert_eq!(host_state, HostState::UpdateRequired);
}

/// Majeure supérieure : même réponse.
#[tokio::test(flavor = "multi_thread")]
async fn a_newer_major_is_also_update_required() {
    let daemon = TestDaemon::start_closed().await;
    let reply = hello_at(&daemon, daemon.token(), CONTRACT_VERSION.major + 1, 0).await;
    assert!(matches!(
        reply,
        Frame::HelloOk {
            host_state: HostState::UpdateRequired,
            ..
        }
    ));
}

/// Mauvais jeton avec un contrat plus récent : refus d'authentification, rien sur l'état de l'hôte.
#[tokio::test(flavor = "multi_thread")]
async fn a_bad_token_is_refused_before_the_versions_are_looked_at() {
    let daemon = TestDaemon::start_closed().await;
    let (major, minor) = (CONTRACT_VERSION.major, CONTRACT_VERSION.minor + 1);
    let reply = hello_at(&daemon, "faux", major, minor).await;
    let refusal = Frame::HelloErr {
        reason: HelloRefusal::Auth,
    };
    assert_eq!(reply, refusal);
}
