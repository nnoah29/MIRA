#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Création, renommage et suppression de boîtes ; refus du serveur convertis en erreurs.

mod support_imap;

use mira_core::imap::ImapFlag;
use mira_testkit::imap::CapabilitySet;
use support_imap::env::start;
use support_imap::logins::{connected, selected};
use support_imap::mail::MAIL;
use support_imap::within;

/// Créer, renommer puis supprimer une boîte se voit dans la liste
#[tokio::test]
async fn create_rename_delete_mailbox() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        let mut client = connected(&env).await;
        client.create_mailbox("Projets").await.unwrap();
        let names = |boxes: Vec<mira_vocabulary::RawMailbox>| -> Vec<String> {
            boxes.into_iter().map(|b| b.name).collect()
        };
        assert!(names(client.list_mailboxes().await.unwrap()).contains(&"Projets".to_string()));
        client
            .rename_mailbox("Projets", "Projets 2024")
            .await
            .unwrap();
        let renamed = names(client.list_mailboxes().await.unwrap());
        assert!(
            renamed.contains(&"Projets 2024".to_string())
                && !renamed.contains(&"Projets".to_string())
        );
        client.delete_mailbox("Projets 2024").await.unwrap();
        assert!(
            !names(client.list_mailboxes().await.unwrap()).contains(&"Projets 2024".to_string())
        );
    })
    .await;
}

/// Créer une boîte qui existe déjà est refusé (server.rejected)
#[tokio::test]
async fn creating_an_existing_mailbox_is_rejected() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        let mut client = connected(&env).await;
        let error = client.create_mailbox("INBOX").await.unwrap_err();
        assert_eq!(error.code, "server.rejected");
        assert!(!error.retryable);
    })
    .await;
}

/// COPY vers une boîte absente donne server.mailbox_missing
#[tokio::test]
async fn copy_to_a_missing_mailbox_is_reported() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        env.store.deliver("alice", "INBOX", MAIL, &[]);
        let mut client = selected(&env).await;
        let error = client.copy(&[1], "Absente").await.unwrap_err();
        assert_eq!(error.code, "server.mailbox_missing");
    })
    .await;
}

/// Un dépassement de quota annoncé par le serveur donne server.quota_exceeded
#[tokio::test]
async fn over_quota_is_reported() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        env.store
            .fail_next("APPEND", "NO [OVERQUOTA] Quota exceeded (mailbox full)");
        let mut client = connected(&env).await;
        let error = client
            .append("INBOX", &[ImapFlag::Seen], MAIL)
            .await
            .unwrap_err();
        assert_eq!(error.code, "server.quota_exceeded");
        assert!(client.append("INBOX", &[], MAIL).await.unwrap().is_some());
    })
    .await;
}

/// Une commande rejetée par le serveur reste utilisable ensuite (la session n'est pas cassée)
#[tokio::test]
async fn session_survives_a_rejected_command() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        env.store.deliver("alice", "INBOX", MAIL, &[]);
        env.store
            .fail_next("UID STORE", "NO [LIMIT] trop de requêtes");
        let mut client = selected(&env).await;
        let error = client.add_flags(&[1], &[ImapFlag::Seen]).await.unwrap_err();
        assert_eq!(error.code, "server.rejected");
        client.add_flags(&[1], &[ImapFlag::Seen]).await.unwrap();
        assert_eq!(
            env.store.messages("alice", "INBOX")[0].flags,
            vec!["\\Seen"]
        );
    })
    .await;
}
