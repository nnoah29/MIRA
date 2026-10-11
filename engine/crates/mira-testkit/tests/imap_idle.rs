#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! IDLE, NOOP et coupures du faux serveur IMAP.

mod imap_support;

use imap_support::client::RawClient;
use imap_support::fixtures::{MAIL, logged_in, selected, start};
use imap_support::within;
use mira_testkit::imap::CapabilitySet;

/// Une livraison pendant IDLE est annoncée par EXISTS, puis DONE rend la main
#[tokio::test]
async fn idle_announces_exists_after_deliver() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        let mut client = selected(&env).await;
        client.send_raw("i1 IDLE\r\n").await;
        assert_eq!(client.read_line().await.unwrap(), "+ idling");
        env.store.deliver("alice", "INBOX", MAIL, &[]);
        assert_eq!(client.read_until_contains("EXISTS").await, "* 1 EXISTS");
        client.send_raw("DONE\r\n").await;
        assert!(client.read_response("i1").await.is_ok());
        assert!(client.command("NOOP").await.is_ok());
    })
    .await;
}

/// Un changement de drapeaux pendant IDLE est annoncé par FETCH
#[tokio::test]
async fn idle_announces_flag_changes() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        env.store.deliver("alice", "INBOX", MAIL, &[]);
        let mut client = selected(&env).await;
        client.send_raw("i1 IDLE\r\n").await;
        client.read_line().await;
        env.store.set_flags("alice", "INBOX", 1, &["\\Seen"]);
        let line = client.read_until_contains("FETCH").await;
        assert_eq!(line, "* 1 FETCH (UID 1 FLAGS (\\Seen))");
    })
    .await;
}

/// Un expunge pendant IDLE est annoncé par EXPUNGE
#[tokio::test]
async fn idle_announces_expunge() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        env.store.deliver("alice", "INBOX", MAIL, &[]);
        env.store.deliver("alice", "INBOX", MAIL, &[]);
        let mut client = selected(&env).await;
        client.send_raw("i1 IDLE\r\n").await;
        client.read_line().await;
        env.store.expunge("alice", "INBOX", 1);
        assert_eq!(client.read_until_contains("EXPUNGE").await, "* 1 EXPUNGE");
    })
    .await;
}

/// IDLE est refusé quand l'extension n'est pas annoncée
#[tokio::test]
async fn idle_without_extension_is_bad() {
    within(async {
        let env = start(CapabilitySet::minimal()).await;
        let mut client = selected(&env).await;
        assert!(client.command("IDLE").await.status.contains(" BAD "));
    })
    .await;
}

/// NOOP annonce les messages arrivés depuis la dernière commande
#[tokio::test]
async fn noop_reports_new_messages() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        let mut client = selected(&env).await;
        env.store.deliver("alice", "INBOX", MAIL, &[]);
        let response = client.command("NOOP").await;
        assert!(response.has("1 EXISTS"), "{response:?}");
        assert!(client.command("NOOP").await.untagged.is_empty());
    })
    .await;
}

/// drop_all_connections coupe les sockets et les connexions sont comptées
#[tokio::test]
async fn drop_all_connections_cuts_sockets() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        let mut client = logged_in(&env).await;
        assert_eq!(env.server.connection_count(), 1);
        env.server.drop_all_connections();
        assert!(client.read_line().await.is_none());
        let again = RawClient::connect(&env.server.address()).await;
        assert!(again.greeting.contains("IMAP4rev1"));
        assert_eq!(env.server.connection_count(), 2);
    })
    .await;
}
