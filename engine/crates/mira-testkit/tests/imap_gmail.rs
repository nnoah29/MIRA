#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Extensions Gmail du faux serveur IMAP : X-GM-MSGID, X-GM-THRID, X-GM-LABELS.

mod imap_support;

use imap_support::fixtures::{MAIL, logged_in, selected, start};
use imap_support::within;
use mira_testkit::imap::CapabilitySet;

/// Un message présent dans deux boîtes garde le même X-GM-MSGID
#[tokio::test]
async fn same_message_in_two_mailboxes_shares_gm_msgid() {
    within(async {
        let env = start(CapabilitySet::gmail()).await;
        env.store
            .create_mailbox("alice", "[Gmail]/All Mail", Some("\\All"));
        let uids = env
            .store
            .deliver_gmail("alice", &["INBOX", "[Gmail]/All Mail"], MAIL, 77, 88);
        assert_eq!(uids.len(), 2);
        let mut client = selected(&env).await;
        let inbox = client.command("UID FETCH 1 (X-GM-MSGID X-GM-THRID)").await;
        assert!(
            inbox.has("X-GM-MSGID 77") && inbox.has("X-GM-THRID 88"),
            "{inbox:?}"
        );
        assert!(client.command("SELECT \"[Gmail]/All Mail\"").await.is_ok());
        let all = client.command("UID FETCH 1 (X-GM-MSGID)").await;
        assert!(all.has("X-GM-MSGID 77"), "{all:?}");
    })
    .await;
}

/// X-GM-LABELS liste les boîtes du message, INBOX devenant `\Inbox`
#[tokio::test]
async fn fetch_labels_lists_mailboxes() {
    within(async {
        let env = start(CapabilitySet::gmail()).await;
        env.store
            .create_mailbox("alice", "[Gmail]/All Mail", Some("\\All"));
        env.store
            .deliver_gmail("alice", &["INBOX", "Work", "[Gmail]/All Mail"], MAIL, 5, 5);
        let mut client = selected(&env).await;
        let response = client.command("UID FETCH 1 (X-GM-LABELS)").await;
        assert!(response.has("X-GM-LABELS (\\Inbox Work)"), "{response:?}");
    })
    .await;
}

/// +X-GM-LABELS ajoute le message à la boîte du libellé
#[tokio::test]
async fn store_add_label_copies_message_into_mailbox() {
    within(async {
        let env = start(CapabilitySet::gmail()).await;
        env.store.create_mailbox("alice", "Work", None);
        env.store.deliver_gmail("alice", &["INBOX"], MAIL, 9, 9);
        let mut client = selected(&env).await;
        let response = client.command("UID STORE 1 +X-GM-LABELS (Work)").await;
        assert!(response.has("X-GM-LABELS (\\Inbox Work)"), "{response:?}");
        let work = env.store.messages("alice", "Work");
        assert_eq!((work.len(), work[0].gm_msgid), (1, 9));
    })
    .await;
}

/// -X-GM-LABELS retire le message de la boîte du libellé
#[tokio::test]
async fn store_remove_label_removes_message_from_mailbox() {
    within(async {
        let env = start(CapabilitySet::gmail()).await;
        env.store
            .deliver_gmail("alice", &["INBOX", "Work"], MAIL, 4, 4);
        let mut client = selected(&env).await;
        assert!(
            client
                .command("UID STORE 1 -X-GM-LABELS (Work)")
                .await
                .is_ok()
        );
        assert!(env.store.messages("alice", "Work").is_empty());
        assert_eq!(env.store.messages("alice", "INBOX").len(), 1);
    })
    .await;
}

/// Sans l'extension Gmail, les éléments X-GM-* sont refusés
#[tokio::test]
async fn gmail_items_require_the_extension() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        env.store.deliver("alice", "INBOX", MAIL, &[]);
        let mut client = selected(&env).await;
        let response = client.command("UID FETCH 1 (X-GM-MSGID)").await;
        assert!(response.status.contains(" BAD "), "{response:?}");
    })
    .await;
}

/// Le profil Gmail annonce X-GM-EXT-1 et pas QRESYNC
#[tokio::test]
async fn gmail_profile_capabilities() {
    within(async {
        let env = start(CapabilitySet::gmail()).await;
        let mut client = logged_in(&env).await;
        let line = client.command("CAPABILITY").await.line("CAPABILITY");
        assert!(
            line.contains("X-GM-EXT-1") && !line.contains("QRESYNC"),
            "{line}"
        );
    })
    .await;
}
