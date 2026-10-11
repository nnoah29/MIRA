#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! UID SEARCH et UID STORE du faux serveur IMAP.

mod imap_support;

use imap_support::fixtures::{MAIL, mail, selected, start};
use imap_support::within;
use mira_testkit::imap::CapabilitySet;

/// SINCE, BEFORE, UID et ALL filtrent selon la date interne et l'UID
#[tokio::test]
async fn search_by_date_and_uid() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        env.store
            .deliver_dated("alice", "INBOX", MAIL, &[], 1_577_836_800);
        env.store
            .deliver_dated("alice", "INBOX", MAIL, &[], 1_704_067_200);
        let mut client = selected(&env).await;
        assert_eq!(client.search_line("ALL").await, "* SEARCH 1 2");
        assert_eq!(client.search_line("SINCE 01-Jun-2022").await, "* SEARCH 2");
        assert_eq!(client.search_line("BEFORE 1-Jun-2022").await, "* SEARCH 1");
        assert_eq!(client.search_line("UID 2:*").await, "* SEARCH 2");
    })
    .await;
}

/// SUBJECT, FROM, TEXT, NOT, OR et MODSEQ se combinent
#[tokio::test]
async fn search_by_text_and_modseq() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        env.store
            .deliver("alice", "INBOX", &mail("Invoice March", "a@b"), &[]);
        env.store
            .deliver("alice", "INBOX", &mail("Holiday", "c@d"), &[]);
        let mut client = selected(&env).await;
        assert_eq!(client.search_line("SUBJECT invoice").await, "* SEARCH 1");
        assert_eq!(client.search_line("FROM alice").await, "* SEARCH 1 2");
        assert_eq!(
            client.search_line("TEXT \"body of holiday\"").await,
            "* SEARCH 2"
        );
        assert_eq!(
            client.search_line("NOT SUBJECT invoice").await,
            "* SEARCH 2"
        );
        let either = client
            .search_line("OR SUBJECT invoice SUBJECT holiday")
            .await;
        assert_eq!(either, "* SEARCH 1 2");
        assert_eq!(
            client.search_line("MODSEQ 3").await,
            "* SEARCH 2 (MODSEQ 3)"
        );
    })
    .await;
}

/// HEADER compare la valeur d'un champ sans tenir compte de la casse
#[tokio::test]
async fn search_header_matches_message_id_case_insensitively() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        env.store.deliver("alice", "INBOX", MAIL, &[]);
        env.store
            .deliver("alice", "INBOX", &mail("Other", "other@mira"), &[]);
        let mut client = selected(&env).await;
        let hit = client
            .command("UID SEARCH HEADER Message-ID \"<ABC@mira>\"")
            .await;
        assert_eq!(hit.line("SEARCH"), "* SEARCH 1");
        let miss = client
            .command("UID SEARCH HEADER Message-ID \"<nope@mira>\"")
            .await;
        assert_eq!(miss.line("SEARCH"), "* SEARCH");
        let subject = client.command("UID SEARCH HEADER subject OTH").await;
        assert_eq!(subject.line("SEARCH"), "* SEARCH 2");
    })
    .await;
}

/// STORE ajoute des drapeaux et incrémente MODSEQ
#[tokio::test]
async fn store_adds_flags_and_bumps_modseq() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        env.store.deliver("alice", "INBOX", MAIL, &[]);
        let mut client = selected(&env).await;
        assert!(client.command("ENABLE CONDSTORE").await.is_ok());
        let before = env.store.messages("alice", "INBOX")[0].modseq;
        let response = client.command("UID STORE 1 +FLAGS (\\Flagged)").await;
        let line = response.line("FETCH");
        assert!(
            line.contains("FLAGS (\\Flagged)") && line.contains("MODSEQ ("),
            "{line}"
        );
        let after = env.store.messages("alice", "INBOX")[0].modseq;
        assert!(after > before);
    })
    .await;
}

/// UNCHANGEDSINCE refuse la modification d'un message plus récent
#[tokio::test]
async fn store_unchangedsince_reports_modified() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        env.store.deliver("alice", "INBOX", MAIL, &[]);
        let mut client = selected(&env).await;
        let response = client
            .command("UID STORE 1 (UNCHANGEDSINCE 0) +FLAGS (\\Seen)")
            .await;
        assert!(response.status.contains("[MODIFIED 1]"), "{response:?}");
        assert!(env.store.messages("alice", "INBOX")[0].flags.is_empty());
    })
    .await;
}

/// -FLAGS.SILENT retire un drapeau sans réponse FETCH
#[tokio::test]
async fn store_silent_removes_flags_quietly() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        env.store
            .deliver("alice", "INBOX", MAIL, &["\\Seen", "\\Flagged"]);
        let mut client = selected(&env).await;
        let response = client.command("UID STORE 1 -FLAGS.SILENT (\\Seen)").await;
        assert!(
            response.is_ok() && response.untagged.is_empty(),
            "{response:?}"
        );
        assert_eq!(
            env.store.messages("alice", "INBOX")[0].flags,
            vec!["\\Flagged"]
        );
    })
    .await;
}
