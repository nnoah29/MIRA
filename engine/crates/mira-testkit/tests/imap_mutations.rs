#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! APPEND, EXPUNGE, COPY et MOVE du faux serveur IMAP.

mod imap_support;

use imap_support::fixtures::{MAIL, logged_in, selected, start};
use imap_support::within;
use mira_testkit::imap::CapabilitySet;

/// APPEND avec littéral synchronisant attend `+ Ready` puis renvoie APPENDUID
#[tokio::test]
async fn append_with_synchronizing_literal() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        let mut client = logged_in(&env).await;
        client
            .send_raw(&format!("p1 APPEND INBOX (\\Seen) {{{}}}\r\n", MAIL.len()))
            .await;
        assert_eq!(client.read_line().await.unwrap(), "+ Ready");
        client.send_bytes(MAIL).await;
        client.send_raw("\r\n").await;
        let response = client.read_response("p1").await;
        assert!(response.status.contains("[APPENDUID"), "{response:?}");
        let stored = env.store.messages("alice", "INBOX");
        assert_eq!((stored.len(), stored[0].raw.as_slice()), (1, MAIL));
        assert_eq!(stored[0].flags, vec!["\\Seen"]);
    })
    .await;
}

/// LITERAL+ envoie le message sans attendre de continuation et la date est conservée
#[tokio::test]
async fn append_with_literal_plus_keeps_the_date() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        let mut client = logged_in(&env).await;
        let head = format!(
            "p1 APPEND INBOX \"14-Nov-2023 22:13:20 +0000\" {{{}+}}\r\n",
            MAIL.len()
        );
        client.send_raw(&head).await;
        client.send_bytes(MAIL).await;
        client.send_raw("\r\n").await;
        let response = client.read_response("p1").await;
        assert!(
            response.is_ok() && response.untagged.is_empty(),
            "{response:?}"
        );
        assert!(client.command("SELECT INBOX").await.is_ok());
        let fetch = client.command("UID FETCH 1 (INTERNALDATE)").await;
        assert!(fetch.has("14-Nov-2023 22:13:20 +0000"), "{fetch:?}");
    })
    .await;
}

/// EXPUNGE supprime les messages `\Deleted` et annonce leur numéro
#[tokio::test]
async fn expunge_removes_deleted_messages() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        for _ in 0..3 {
            env.store.deliver("alice", "INBOX", MAIL, &[]);
        }
        env.store.set_flags("alice", "INBOX", 2, &["\\Deleted"]);
        let mut client = selected(&env).await;
        let response = client.command("EXPUNGE").await;
        assert!(
            response.is_ok() && response.has("* 2 EXPUNGE"),
            "{response:?}"
        );
        let uids: Vec<u32> = env
            .store
            .messages("alice", "INBOX")
            .iter()
            .map(|m| m.uid)
            .collect();
        assert_eq!(uids, vec![1, 3]);
    })
    .await;
}

/// UID COPY renvoie COPYUID et conserve les drapeaux
#[tokio::test]
async fn copy_returns_copyuid_and_keeps_flags() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        env.store.create_mailbox("alice", "Archive", None);
        env.store.deliver("alice", "INBOX", MAIL, &["\\Seen"]);
        env.store.deliver("alice", "INBOX", MAIL, &[]);
        let mut client = selected(&env).await;
        let response = client.command("UID COPY 1:2 Archive").await;
        assert!(
            response.status.contains("[COPYUID") && response.status.contains("1:2 1:2]"),
            "{response:?}"
        );
        let copies = env.store.messages("alice", "Archive");
        assert_eq!(
            (copies.len(), copies[0].flags.clone()),
            (2, vec!["\\Seen".to_string()])
        );
    })
    .await;
}

/// UID MOVE transfère le message puis annonce son EXPUNGE
#[tokio::test]
async fn move_transfers_and_expunges_the_source() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        env.store.create_mailbox("alice", "Archive", None);
        env.store.deliver("alice", "INBOX", MAIL, &[]);
        env.store.deliver("alice", "INBOX", MAIL, &[]);
        let mut client = selected(&env).await;
        let response = client.command("UID MOVE 1 Archive").await;
        assert!(
            response.has("OK [COPYUID") && response.has("1 EXPUNGE"),
            "{response:?}"
        );
        assert_eq!(env.store.messages("alice", "INBOX").len(), 1);
        assert_eq!(env.store.messages("alice", "Archive").len(), 1);
    })
    .await;
}

/// Sans l'extension MOVE, la commande est refusée par BAD
#[tokio::test]
async fn move_without_extension_is_bad() {
    within(async {
        let env = start(CapabilitySet::without_move()).await;
        env.store.create_mailbox("alice", "Archive", None);
        env.store.deliver("alice", "INBOX", MAIL, &[]);
        let mut client = selected(&env).await;
        let response = client.command("UID MOVE 1 Archive").await;
        assert!(response.status.contains(" BAD "), "{response:?}");
        assert_eq!(env.store.messages("alice", "INBOX").len(), 1);
    })
    .await;
}
