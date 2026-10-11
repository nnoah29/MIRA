#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Drapeaux, MOVE, COPY, EXPUNGE et APPEND contre le faux serveur.

mod support_imap;

use mira_core::imap::ImapFlag;
use mira_testkit::imap::CapabilitySet;
use support_imap::env::start;
use support_imap::logins::{connected, selected};
use support_imap::mail::MAIL;
use support_imap::within;

/// Ajout puis retrait de drapeaux, y compris `$Forwarded`
#[tokio::test]
async fn add_and_remove_flags() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        env.store.deliver("alice", "INBOX", MAIL, &[]);
        let mut client = selected(&env).await;
        let flags = [ImapFlag::Seen, ImapFlag::Flagged, ImapFlag::Forwarded];
        client.add_flags(&[1], &flags).await.unwrap();
        let mut now = env.store.messages("alice", "INBOX")[0].flags.clone();
        now.sort();
        assert_eq!(now, vec!["$Forwarded", "\\Flagged", "\\Seen"]);
        client
            .remove_flags(&[1], &[ImapFlag::Flagged, ImapFlag::Forwarded])
            .await
            .unwrap();
        assert_eq!(
            env.store.messages("alice", "INBOX")[0].flags,
            vec!["\\Seen"]
        );
        client.add_flags(&[], &flags).await.unwrap();
    })
    .await;
}

/// MOVE déplace les messages vers l'autre boîte
#[tokio::test]
async fn move_messages_between_mailboxes() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        env.store.create_mailbox("alice", "Archive", None);
        env.store.deliver("alice", "INBOX", MAIL, &[]);
        env.store.deliver("alice", "INBOX", MAIL, &[]);
        let mut client = selected(&env).await;
        client.move_messages(&[1, 2], "Archive").await.unwrap();
        assert!(env.store.messages("alice", "INBOX").is_empty());
        assert_eq!(env.store.messages("alice", "Archive").len(), 2);
    })
    .await;
}

/// Sans MOVE, l'extension est signalée absente et rien n'est envoyé au serveur
#[tokio::test]
async fn move_without_extension_is_reported() {
    within(async {
        let env = start(CapabilitySet::without_move()).await;
        env.store.create_mailbox("alice", "Archive", None);
        env.store.deliver("alice", "INBOX", MAIL, &[]);
        let mut client = selected(&env).await;
        let error = client.move_messages(&[1], "Archive").await.unwrap_err();
        assert_eq!(error.code, "server.extension_missing");
        assert!(!env.store.command_log().iter().any(|l| l.contains("MOVE")));
        assert_eq!(env.store.messages("alice", "INBOX").len(), 1);
    })
    .await;
}

/// COPY garde l'original ; `\Deleted` puis UID EXPUNGE le supprime ; UIDPLUS est requis
#[tokio::test]
async fn copy_then_expunge_selected_uids() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        env.store.create_mailbox("alice", "Archive", None);
        for _ in 0..2 {
            env.store.deliver("alice", "INBOX", MAIL, &[]);
        }
        let mut client = selected(&env).await;
        client.copy(&[1], "Archive").await.unwrap();
        assert_eq!(env.store.messages("alice", "Archive").len(), 1);
        client
            .add_flags(&[1, 2], &[ImapFlag::Deleted])
            .await
            .unwrap();
        client.expunge_uids(&[1]).await.unwrap();
        let left = env.store.messages("alice", "INBOX");
        assert_eq!(left.iter().map(|m| m.uid).collect::<Vec<_>>(), vec![2]);
        let bare = start(CapabilitySet::minimal()).await;
        let error = selected(&bare).await.expunge_uids(&[1]).await.unwrap_err();
        assert_eq!(error.code, "server.extension_missing");
    })
    .await;
}

/// APPEND avec LITERAL+ et UIDPLUS rend l'UID attribué et conserve drapeaux et contenu
#[tokio::test]
async fn append_returns_the_assigned_uid() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        let mut client = connected(&env).await;
        let first = client
            .append("INBOX", &[ImapFlag::Seen], MAIL)
            .await
            .unwrap();
        let second = client
            .append("INBOX", &[], b"Subject: deux\r\n\r\nx\r\n")
            .await
            .unwrap();
        assert_eq!((first, second), (Some(1), Some(2)));
        let stored = env.store.messages("alice", "INBOX");
        assert_eq!(stored[0].raw, MAIL);
        assert_eq!(stored[0].flags, vec!["\\Seen"]);
    })
    .await;
}

/// Sans LITERAL+ ni UIDPLUS, APPEND attend l'invite du serveur et ne rend pas d'UID
#[tokio::test]
async fn append_without_literal_plus_or_uidplus() {
    within(async {
        let env = start(CapabilitySet::minimal()).await;
        let mut client = connected(&env).await;
        let uid = client
            .append("INBOX", &[ImapFlag::Draft], MAIL)
            .await
            .unwrap();
        assert_eq!(uid, None);
        let stored = env.store.messages("alice", "INBOX");
        assert_eq!((stored.len(), stored[0].raw.as_slice()), (1, MAIL));
        assert_eq!(stored[0].flags, vec!["\\Draft"]);
    })
    .await;
}
