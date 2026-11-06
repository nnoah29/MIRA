#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Poignée de main : versions, jeton et état annoncés, refus à raison fixe.

mod support;

use mira_assembly::mira_vocabulary::{CONTRACT_VERSION, ContractVersion};
use mira_protocol::{Frame, HelloRefusal, HostState, PROTOCOL_VERSION, ProtocolVersion};
use support::{TestClient, TestDaemon, config, hello_with, valid_hello};

/// Un `hello` valide reçoit `hello_ok` avec les versions du démon et l'état `ready`.
#[tokio::test(flavor = "multi_thread")]
async fn a_valid_hello_gets_hello_ok_with_versions_and_ready_state() {
    let home = tempfile::tempdir().unwrap();
    let daemon = TestDaemon::start(config(home.path())).await;
    let mut client = TestClient::connect(&daemon).await;
    let reply = client.hello_ok(daemon.token()).await;
    let Frame::HelloOk {
        protocol_version,
        contract_version,
        host_state,
    } = reply
    else {
        panic!("hello_ok attendu");
    };
    assert_eq!(protocol_version, PROTOCOL_VERSION);
    assert_eq!(contract_version, CONTRACT_VERSION);
    assert_eq!(host_state, HostState::Ready);
}

/// Un mauvais jeton reçoit `hello_err auth`, puis la connexion se ferme ; aucun jeton ne circule.
#[tokio::test(flavor = "multi_thread")]
async fn a_wrong_token_gets_hello_err_auth_and_the_connection_closes() {
    let daemon = TestDaemon::start_closed().await;
    let mut client = TestClient::connect(&daemon).await;
    let wrong = "0".repeat(64);
    client.send(valid_hello(&wrong)).await;
    let reply = client.recv().await.unwrap();
    assert_eq!(
        reply,
        Frame::HelloErr {
            reason: HelloRefusal::Auth
        }
    );
    assert!(client.recv().await.is_none());
    let seen = String::from_utf8_lossy(client.received_bytes()).to_string();
    assert!(!seen.contains(daemon.token()) && !seen.contains(&wrong));
}

/// Une autre majeure du protocole reçoit `hello_err protocol_version`.
#[tokio::test(flavor = "multi_thread")]
async fn a_different_protocol_major_gets_hello_err_protocol_version() {
    let daemon = TestDaemon::start_closed().await;
    let mut client = TestClient::connect(&daemon).await;
    let protocol = ProtocolVersion {
        major: PROTOCOL_VERSION.major + 1,
        minor: 0,
    };
    client
        .send(hello_with(daemon.token(), protocol, CONTRACT_VERSION))
        .await;
    let reply = client.recv().await.unwrap();
    let refusal = Frame::HelloErr {
        reason: HelloRefusal::ProtocolVersion,
    };
    assert_eq!(reply, refusal);
}

/// Un contrat client plus récent que le moteur reçoit `hello_err contract_version`.
#[tokio::test(flavor = "multi_thread")]
async fn a_newer_contract_minor_gets_hello_err_contract_version() {
    let daemon = TestDaemon::start_closed().await;
    let mut client = TestClient::connect(&daemon).await;
    let contract = ContractVersion {
        major: CONTRACT_VERSION.major,
        minor: CONTRACT_VERSION.minor + 1,
    };
    client
        .send(hello_with(daemon.token(), PROTOCOL_VERSION, contract))
        .await;
    let reply = client.recv().await.unwrap();
    let refusal = Frame::HelloErr {
        reason: HelloRefusal::ContractVersion,
    };
    assert_eq!(reply, refusal);
}
