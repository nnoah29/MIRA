#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Boîtes du faux serveur IMAP : LIST, STATUS, SELECT, EXAMINE, CREATE, RENAME, DELETE.

mod imap_support;

use imap_support::fixtures::{MAIL, logged_in, start};
use imap_support::within;
use mira_testkit::imap::CapabilitySet;

/// LIST annonce SPECIAL-USE et la hiérarchie avec le séparateur `/`
#[tokio::test]
async fn list_reports_special_use_and_children() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        env.store.create_mailbox("alice", "Sent", Some("\\Sent"));
        env.store.create_mailbox("alice", "Work", None);
        env.store.create_mailbox("alice", "Work/Projects", None);
        let mut client = logged_in(&env).await;
        let response = client.command("LIST \"\" \"*\"").await;
        assert!(
            response.has("* LIST (\\HasNoChildren \\Sent) \"/\" \"Sent\""),
            "{response:?}"
        );
        assert!(response.has("(\\HasChildren) \"/\" \"Work\""));
        assert!(response.has("\"Work/Projects\""));
        let shallow = client.command("LIST \"\" \"%\"").await;
        assert!(!shallow.has("Work/Projects"));
    })
    .await;
}

/// SELECT annonce EXISTS, UIDVALIDITY, UIDNEXT et HIGHESTMODSEQ
#[tokio::test]
async fn select_reports_uidvalidity_and_highestmodseq() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        env.store.deliver("alice", "INBOX", MAIL, &[]);
        env.store.deliver("alice", "INBOX", MAIL, &[]);
        let mut client = logged_in(&env).await;
        let response = client.command("SELECT INBOX").await;
        let validity = env.store.uidvalidity("alice", "INBOX").unwrap();
        let modseq = env.store.highest_modseq("alice", "INBOX").unwrap();
        assert!(response.has("2 EXISTS"));
        assert!(response.has(&format!("[UIDVALIDITY {validity}]")));
        assert!(response.has("[UIDNEXT 3]"));
        assert!(response.has(&format!("[HIGHESTMODSEQ {modseq}]")));
        assert!(response.status.contains("[READ-WRITE]"));
    })
    .await;
}

/// EXAMINE ouvre en lecture seule et STORE y est refusé
#[tokio::test]
async fn examine_is_read_only() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        env.store.deliver("alice", "INBOX", MAIL, &[]);
        let mut client = logged_in(&env).await;
        let response = client.command("EXAMINE INBOX").await;
        assert!(response.status.contains("[READ-ONLY]"));
        let store = client.command("UID STORE 1 +FLAGS (\\Seen)").await;
        assert!(store.status.contains("NO [READ-ONLY]"), "{store:?}");
    })
    .await;
}

/// Une boîte inconnue est refusée avec NONEXISTENT
#[tokio::test]
async fn select_unknown_mailbox_is_refused() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        let mut client = logged_in(&env).await;
        let response = client.command("SELECT Missing").await;
        assert!(response.status.contains("NO [NONEXISTENT]"), "{response:?}");
        assert!(
            client
                .command("FETCH 1 (FLAGS)")
                .await
                .status
                .contains("BAD")
        );
    })
    .await;
}

/// STATUS donne les compteurs sans sélectionner
#[tokio::test]
async fn status_reports_counters() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        env.store.deliver("alice", "INBOX", MAIL, &["\\Seen"]);
        env.store.deliver("alice", "INBOX", MAIL, &[]);
        let mut client = logged_in(&env).await;
        let response = client
            .command("STATUS INBOX (MESSAGES UIDNEXT UNSEEN)")
            .await;
        assert!(
            response.has("STATUS \"INBOX\" (MESSAGES 2 UIDNEXT 3 UNSEEN 1)"),
            "{response:?}"
        );
    })
    .await;
}

/// CREATE, RENAME et DELETE modifient la liste des boîtes
#[tokio::test]
async fn create_rename_delete_roundtrip() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        let mut client = logged_in(&env).await;
        assert!(client.command("CREATE Archive").await.is_ok());
        assert!(
            client
                .command("CREATE Archive")
                .await
                .status
                .contains("[ALREADYEXISTS]")
        );
        assert!(client.command("RENAME Archive Old").await.is_ok());
        assert!(client.command("STATUS Old (MESSAGES)").await.is_ok());
        assert!(client.command("DELETE Old").await.is_ok());
        assert!(!client.command("STATUS Old (MESSAGES)").await.is_ok());
    })
    .await;
}
