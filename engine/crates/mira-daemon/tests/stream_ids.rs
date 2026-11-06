#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! HTML d'origine et identifiants de flux : réponse en `result`, réutilisation fermée, inconnus ignorés.

mod support;

use mira_protocol::Frame;
use support::mail_daemon::{MailDaemon, add_account, start_sync, wait_inbox};
use support::{TestClient, TestDaemon, config};

/// L'HTML d'origine d'un message riche garde tout ; un message texte seul n'en a pas.
#[tokio::test(flavor = "multi_thread")]
async fn get_original_html_answers_with_a_result() {
    let mail = MailDaemon::start().await;
    let mut client = mail.attach().await;
    let account = add_account(&mut client).await;
    start_sync(&mut client).await;
    let messages = wait_inbox(&mut client, &account).await;
    let id_of = |subject: &str| messages.iter().find(|m| m.0 == subject).unwrap().1.clone();
    client
        .send(Frame::GetOriginalHtml {
            id: 90,
            message: id_of("Riche"),
        })
        .await;
    let Some(Frame::Result { id: 90, envelope }) = client.recv().await else {
        panic!("result attendu")
    };
    let html = envelope["data"]["html"].as_str().unwrap();
    assert!(html.contains("<table>") && html.contains("https://tracker.example/p.png"));
    client
        .send(Frame::GetOriginalHtml {
            id: 91,
            message: id_of("Bonjour"),
        })
        .await;
    let Some(Frame::Result { id: 91, envelope }) = client.recv().await else {
        panic!("result attendu")
    };
    assert!(envelope["data"]["html"].is_null());
}

/// Rouvrir un flux dont l'identifiant est encore ouvert ferme la connexion.
#[tokio::test(flavor = "multi_thread")]
async fn a_stream_id_reused_while_open_closes_the_connection() {
    let home = tempfile::tempdir().unwrap();
    let daemon = TestDaemon::start(config(home.path())).await;
    let mut client = TestClient::connect(&daemon).await;
    client.hello_ok(daemon.token()).await;
    client.open_events(3, None, 1).await;
    client.send(Frame::Ping { nonce: 1 }).await;
    assert_eq!(client.recv().await, Some(Frame::Pong { nonce: 1 }));
    client.open_events(3, None, 1).await;
    assert!(client.recv().await.is_none());
}

/// Crédit et annulation sur un identifiant inconnu sont ignorés : la connexion continue.
#[tokio::test(flavor = "multi_thread")]
async fn credit_and_cancel_on_an_unknown_id_are_ignored() {
    let daemon = TestDaemon::start_closed().await;
    let mut client = TestClient::connect(&daemon).await;
    client.hello_ok(daemon.token()).await;
    client.send(Frame::Credit { id: 99, n: 5 }).await;
    client.send(Frame::Cancel { id: 99 }).await;
    client.send(Frame::Ping { nonce: 1 }).await;
    assert_eq!(client.recv().await, Some(Frame::Pong { nonce: 1 }));
}
