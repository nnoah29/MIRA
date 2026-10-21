#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Recherche de texte côté serveur avec des caractères non ASCII (RFC 3501 §6.4.4, CHARSET UTF-8).

mod support_imap;

use mira_testkit::imap::CapabilitySet;
use support_imap::env::start;
use support_imap::logins::selected;
use support_imap::mail::MAIL;
use support_imap::within;

/// Une requête non ASCII passe par `SEARCH CHARSET UTF-8` avec un littéral, jamais en guillemets
#[tokio::test]
async fn search_text_with_accents_uses_utf8_charset() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        let raw = "Subject: café\r\n\r\nUn café très chaud.\r\n".as_bytes();
        env.store.deliver("alice", "INBOX", raw, &[]);
        env.store.deliver("alice", "INBOX", MAIL, &[]);
        let mut client = selected(&env).await;
        assert_eq!(client.search_text("café").await.unwrap(), vec![1]);
        let log = env.store.command_log().join("\n");
        assert!(log.contains("CHARSET UTF-8"), "{log}");
    })
    .await;
}

/// Une requête ASCII reste une chaîne entre guillemets, sans CHARSET
#[tokio::test]
async fn search_text_in_ascii_stays_quoted() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        env.store.deliver("alice", "INBOX", MAIL, &[]);
        let mut client = selected(&env).await;
        client.search_text("facture").await.unwrap();
        let log = env.store.command_log().join("\n");
        assert!(log.contains("TEXT \"facture\""), "{log}");
        assert!(!log.contains("CHARSET"), "{log}");
    })
    .await;
}
