#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Pannes injectées, journal, ESEARCH et arrêt du faux serveur IMAP.

mod imap_support;

use std::time::Duration;

use imap_support::client::RawClient;
use imap_support::fixtures::{MAIL, logged_in, selected, start};
use imap_support::within;
use mira_testkit::imap::CapabilitySet;
use tokio::net::TcpStream;

/// fail_next fait répondre l'erreur injectée une seule fois
#[tokio::test]
async fn fail_next_returns_the_injected_response_once() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        env.store.create_mailbox("alice", "Archive", None);
        env.store.deliver("alice", "INBOX", MAIL, &[]);
        env.store.fail_next("UID MOVE", "NO [OVERQUOTA] quota");
        let mut client = selected(&env).await;
        let first = client.command("UID MOVE 1 Archive").await;
        assert!(first.status.ends_with("NO [OVERQUOTA] quota"), "{first:?}");
        assert_eq!(env.store.messages("alice", "INBOX").len(), 1);
        assert!(client.command("UID MOVE 1 Archive").await.is_ok());
        assert_eq!(env.store.messages("alice", "Archive").len(), 1);
    })
    .await;
}

/// Le journal garde les commandes dans l'ordre et masque le mot de passe
#[tokio::test]
async fn command_log_records_commands_without_secrets() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        let mut client = selected(&env).await;
        client.command("UID FETCH 1:* (FLAGS)").await;
        let expected = ["LOGIN alice ***", "SELECT INBOX", "UID FETCH 1:* (FLAGS)"];
        assert_eq!(env.store.command_log(), expected);
    })
    .await;
}

/// Avant LOGIN les commandes de boîte sont refusées, une commande inconnue aussi
#[tokio::test]
async fn commands_before_login_and_unknown_commands_are_bad() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        let mut client = RawClient::connect(&env.server.address()).await;
        assert!(
            client
                .command("SELECT INBOX")
                .await
                .status
                .contains(" BAD ")
        );
        assert!(client.command("FROBNICATE").await.status.contains(" BAD "));
        assert!(client.command("NOOP").await.is_ok());
    })
    .await;
}

/// RETURN (MIN MAX COUNT ALL) produit une réponse ESEARCH
#[tokio::test]
async fn esearch_returns_requested_options() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        for _ in 0..3 {
            env.store.deliver("alice", "INBOX", MAIL, &[]);
        }
        let mut client = selected(&env).await;
        let response = client
            .command("UID SEARCH RETURN (MIN MAX COUNT ALL) ALL")
            .await;
        let expected = "* ESEARCH (TAG \"a3\") UID MIN 1 MAX 3 COUNT 3 ALL 1:3";
        assert_eq!(response.line("ESEARCH"), expected);
    })
    .await;
}

/// ID suit l'extension annoncée
#[tokio::test]
async fn id_follows_the_capability() {
    within(async {
        let full = start(CapabilitySet::full()).await;
        let mut client = logged_in(&full).await;
        assert!(client.command("ID NIL").await.has("mira-fake-imap"));
        let minimal = start(CapabilitySet::minimal()).await;
        let mut client = logged_in(&minimal).await;
        assert!(client.command("ID NIL").await.status.contains(" BAD "));
    })
    .await;
}

/// Après shutdown, le port n'accepte plus de connexion
#[tokio::test]
async fn shutdown_stops_listening() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        let address = env.server.address();
        env.server.shutdown();
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert!(TcpStream::connect(&address).await.is_err());
    })
    .await;
}
