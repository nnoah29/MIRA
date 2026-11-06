#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Appels du contrat : enveloppe appariée par identifiant, appels concurrents, erreurs, `ping`.

mod support;

use mira_protocol::Frame;
use serde_json::json;
use support::{TestClient, TestDaemon, config};

const LIST_ACCOUNTS: &str = r#"{"domain":"account","request":{"op":"list_accounts"}}"#;

/// Un appel reçoit l'enveloppe du contrat, appariée à son identifiant.
#[tokio::test(flavor = "multi_thread")]
async fn a_call_gets_the_contract_envelope_matched_by_id() {
    let home = tempfile::tempdir().unwrap();
    let daemon = TestDaemon::start(config(home.path())).await;
    let mut client = TestClient::connect(&daemon).await;
    client.hello_ok(daemon.token()).await;
    let envelope = client
        .call(serde_json::from_str(LIST_ACCOUNTS).unwrap())
        .await;
    assert_eq!(envelope["ok"], true, "{envelope}");
    assert_eq!(envelope["contract_version"]["major"], 1);
}

/// Trois appels d'affilée reçoivent chacun leur réponse, quel que soit l'ordre d'arrivée.
#[tokio::test(flavor = "multi_thread")]
async fn concurrent_calls_are_matched_by_id() {
    let home = tempfile::tempdir().unwrap();
    let daemon = TestDaemon::start(config(home.path())).await;
    let mut client = TestClient::connect(&daemon).await;
    client.hello_ok(daemon.token()).await;
    let accounts = client
        .send_call(serde_json::from_str(LIST_ACCOUNTS).unwrap())
        .await;
    let info = client
        .send_call(json!({"domain": "engine", "request": {"op": "get_contract_info"}}))
        .await;
    let broken = client.send_call(json!({"domain": "nowhere"})).await;
    let mut answers = std::collections::HashMap::new();
    for _ in 0..3 {
        let Some(Frame::Result { id, envelope }) = client.recv().await else {
            panic!("result attendu")
        };
        answers.insert(id, envelope);
    }
    assert!(
        answers[&accounts]["data"]["data"].is_array(),
        "{}",
        answers[&accounts]
    );
    assert_eq!(answers[&info]["data"]["data"]["version"]["major"], 1);
    assert_eq!(answers[&broken]["ok"], false);
}

/// Une requête illisible reçoit une enveloppe d'échec, pas une connexion fermée.
#[tokio::test(flavor = "multi_thread")]
async fn a_malformed_request_gets_a_failure_envelope_not_a_closed_connection() {
    let home = tempfile::tempdir().unwrap();
    let daemon = TestDaemon::start(config(home.path())).await;
    let mut client = TestClient::connect(&daemon).await;
    client.hello_ok(daemon.token()).await;
    let envelope = client.call(json!({})).await;
    assert_eq!(envelope["ok"], false);
    let again = client
        .call(serde_json::from_str(LIST_ACCOUNTS).unwrap())
        .await;
    assert_eq!(again["ok"], true);
}

/// Sur un hôte jamais ouvert, l'enveloppe d'échec porte le message fixe.
#[tokio::test(flavor = "multi_thread")]
async fn a_call_on_a_never_opened_host_answers_moteur_non_ouvert() {
    let daemon = TestDaemon::start_closed().await;
    let mut client = TestClient::connect(&daemon).await;
    client.hello_ok(daemon.token()).await;
    let envelope = client
        .call(serde_json::from_str(LIST_ACCOUNTS).unwrap())
        .await;
    assert_eq!(envelope["error"]["message"], "moteur non ouvert");
}

/// `ping` reçoit `pong` avec le même `nonce`.
#[tokio::test(flavor = "multi_thread")]
async fn ping_is_answered_with_pong_and_the_same_nonce() {
    let daemon = TestDaemon::start_closed().await;
    let mut client = TestClient::connect(&daemon).await;
    client.hello_ok(daemon.token()).await;
    client.send(Frame::Ping { nonce: 77 }).await;
    assert_eq!(client.recv().await, Some(Frame::Pong { nonce: 77 }));
}

/// Une réponse plus grande que la taille maximale d'une trame devient une enveloppe d'échec ; la connexion continue.
#[tokio::test(flavor = "multi_thread")]
async fn an_oversize_result_becomes_a_failure_envelope_and_the_connection_survives() {
    let home = tempfile::tempdir().unwrap();
    let daemon =
        TestDaemon::start_with(Some(config(home.path())), |ctx| ctx.max_frame = 1024).await;
    let mut client = TestClient::connect(&daemon).await;
    client.hello_ok(daemon.token()).await;
    let big = client
        .call(json!({"domain": "provider", "request": {"op": "list_providers"}}))
        .await;
    assert_eq!(big["ok"], false, "{big}");
    assert_eq!(big["error"]["message"], "réponse trop grande");
    client.send(Frame::Ping { nonce: 3 }).await;
    assert_eq!(client.recv().await, Some(Frame::Pong { nonce: 3 }));
}
