#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! UID FETCH du faux serveur IMAP : littéraux, drapeaux, taille, date, enveloppe.

mod imap_support;

use imap_support::fixtures::{MAIL, mail, selected, start};
use imap_support::within;
use mira_testkit::imap::CapabilitySet;

/// Les en-têtes sont envoyés en littéral terminé par la ligne vide
#[tokio::test]
async fn fetch_header_returns_a_literal() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        env.store.deliver("alice", "INBOX", MAIL, &[]);
        let mut client = selected(&env).await;
        let response = client.command("UID FETCH 1 (UID BODY.PEEK[HEADER])").await;
        let line = response.line("FETCH");
        assert!(line.contains("BODY[HEADER] {"), "{line}");
        assert!(line.contains("Subject: Hello"));
        assert!(
            line.ends_with("Message-ID: <abc@mira>\r\n\r\n)"),
            "{line:?}"
        );
    })
    .await;
}

/// Le corps complet est envoyé avec la taille exacte du message
#[tokio::test]
async fn fetch_full_body_has_exact_literal_size() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        env.store.deliver("alice", "INBOX", MAIL, &[]);
        let mut client = selected(&env).await;
        let response = client.command("UID FETCH 1 (BODY.PEEK[])").await;
        let line = response.line("FETCH");
        assert!(
            line.contains(&format!("BODY[] {{{}}}", MAIL.len())),
            "{line}"
        );
        assert!(line.contains("Body text"));
    })
    .await;
}

/// Drapeaux, taille et date interne sont renvoyés
#[tokio::test]
async fn fetch_flags_size_and_internal_date() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        env.store
            .deliver_dated("alice", "INBOX", MAIL, &["\\Seen"], 1_700_000_000);
        let mut client = selected(&env).await;
        let response = client
            .command("UID FETCH 1:* (FLAGS RFC822.SIZE INTERNALDATE)")
            .await;
        let line = response.line("FETCH");
        assert!(line.contains("FLAGS (\\Seen)"), "{line}");
        assert!(line.contains(&format!("RFC822.SIZE {}", MAIL.len())));
        assert!(line.contains("INTERNALDATE \"14-Nov-2023 22:13:20 +0000\""));
        assert!(line.contains("UID 1"));
    })
    .await;
}

/// BODY[] sans PEEK pose le drapeau `\Seen`
#[tokio::test]
async fn fetch_without_peek_sets_seen() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        env.store.deliver("alice", "INBOX", MAIL, &[]);
        let mut client = selected(&env).await;
        let response = client.command("UID FETCH 1 (BODY[])").await;
        assert!(
            response.line("FETCH").contains("FLAGS (\\Seen)"),
            "{response:?}"
        );
        assert_eq!(
            env.store.messages("alice", "INBOX")[0].flags,
            vec!["\\Seen"]
        );
    })
    .await;
}

/// HEADER.FIELDS filtre les champs et les ensembles désignent les bons messages
#[tokio::test]
async fn fetch_header_fields_over_a_range() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        for index in 0..3 {
            env.store
                .deliver("alice", "INBOX", &mail(&format!("S{index}"), "x@y"), &[]);
        }
        let mut client = selected(&env).await;
        let response = client
            .command("UID FETCH 2:* (BODY.PEEK[HEADER.FIELDS (Subject)])")
            .await;
        assert_eq!(response.count("FETCH"), 2, "{response:?}");
        assert!(response.has("BODY[HEADER.FIELDS (SUBJECT)]"));
        assert!(response.has("Subject: S1") && response.has("Subject: S2"));
        assert!(!response.has("From:"));
        assert_eq!(client.command("UID FETCH * (UID)").await.count("FETCH"), 1);
    })
    .await;
}

/// ENVELOPE décrit expéditeur, destinataire et sujet
#[tokio::test]
async fn fetch_envelope_describes_addresses() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        env.store.deliver("alice", "INBOX", MAIL, &[]);
        let mut client = selected(&env).await;
        let response = client.command("UID FETCH 1 (ENVELOPE)").await;
        let line = response.line("ENVELOPE");
        assert!(line.contains("\"Hello\""), "{line}");
        assert!(line.contains("((\"Alice\" NIL \"alice\" \"example.org\"))"));
        assert!(line.contains("((NIL NIL \"bob\" \"example.org\"))"));
        assert!(line.contains("\"<abc@mira>\""));
    })
    .await;
}
