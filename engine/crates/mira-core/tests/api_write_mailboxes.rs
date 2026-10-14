#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Écritures sur les boîtes par la façade : D-17, boîtes système, création, renommage, suppression, vidage.

mod support;
mod support_api;
mod support_imap;
mod support_mirror;
mod support_write;

use mira_vocabulary::{ActionKind, ActionState, DraftId, MailboxId};
use support_imap::mail::mail;
use support_imap::within;
use support_write::USER;
use support_write::builders::apply_request;
use support_write::fixture::Fixture;
use support_write::mailbox_builders::{
    create_mailbox, delete_mailbox, empty_mailbox, rename_mailbox,
};

/// D-17 : un second libellé sur un compte à dossiers est refusé, rien n'est mis en file
#[tokio::test]
async fn d17_second_label_is_refused_on_a_folder_account() {
    within(async {
        let f = Fixture::start().await;
        let mailbox = MailboxId::from_raw("box-work");
        let request = apply_request(ActionKind::AddToMailbox { mailbox }, &["msg-1"], "k1");
        let code = f.refused_code(request).await;
        assert_eq!(code, "incompatible_state.multiple_labels_unsupported");
        assert!(f.actions(&[]).await.is_empty());
        assert_eq!(f.boxes_of("msg-1"), ["box-in"]);
    })
    .await;
}

/// Une boîte système ne se supprime ni ne se renomme ; une boîte inconnue est introuvable
#[tokio::test]
async fn system_mailboxes_cannot_be_deleted_or_renamed() {
    within(async {
        let f = Fixture::start().await;
        let system = "incompatible_state.system_mailbox";
        let cases = [
            (delete_mailbox("box-in", "k1"), system),
            (rename_mailbox("box-sent", "Autre", "k2"), system),
            (empty_mailbox("box-nope", "k3"), "not_found.mailbox"),
        ];
        for (request, code) in cases {
            assert_eq!(f.refused_code(request).await, code);
        }
        assert!(f.actions(&[]).await.is_empty());
    })
    .await;
}

/// Créer une boîte : visible tout de suite, créée sur le serveur au rejeu, idempotent par clé
#[tokio::test]
async fn create_mailbox_is_local_first_then_created_on_the_server() {
    within(async {
        let f = Fixture::start().await;
        let create = || create_mailbox(&f.account, "Factures", "k1");
        let action = f.submit_mailbox(create()).await;
        assert_eq!(f.submit_mailbox(create()).await, action);
        let local =
            f.db.read(|tx| tx.find_mailbox_by_name(&f.account, "Factures"));
        assert!(local.unwrap().is_some());
        assert!(f.store.uidvalidity(USER, "Factures").is_none());
        f.advance(5_001);
        f.wait_for_state(&action, ActionState::Succeeded).await;
        assert!(f.store.uidvalidity(USER, "Factures").is_some());
    })
    .await;
}

/// Renommer puis supprimer une boîte ordinaire : le serveur suit, le miroir aussi
#[tokio::test]
async fn rename_then_delete_mailbox_reaches_the_server() {
    within(async {
        let f = Fixture::start().await;
        let rename = rename_mailbox("box-work", "Travail", "k1");
        let renamed = f.submit_mailbox(rename).await;
        f.advance(5_001);
        f.wait_for_state(&renamed, ActionState::Succeeded).await;
        assert!(f.store.uidvalidity(USER, "Travail").is_some());
        assert!(f.store.uidvalidity(USER, "Work").is_none());
        let deleted = f.submit_mailbox(delete_mailbox("box-work", "k2")).await;
        let work = MailboxId::from_raw("box-work");
        assert!(f.db.read(|tx| tx.get_mailbox(&work)).unwrap().is_none());
        f.advance(5_001);
        f.wait_for_state(&deleted, ActionState::Succeeded).await;
        assert!(f.store.uidvalidity(USER, "Travail").is_none());
    })
    .await;
}

/// Vider la corbeille supprime ses messages sur le serveur
#[tokio::test]
async fn empty_trash_removes_its_messages_on_the_server() {
    within(async {
        let f = Fixture::start().await;
        let old = mail("Vieux", "old@example.org");
        f.store.deliver(USER, "Trash", &old, &[]);
        assert_eq!(f.server_uids("Trash").len(), 1);
        let action = f.submit_mailbox(empty_mailbox("box-trash", "k1")).await;
        f.advance(5_001);
        f.wait_for_state(&action, ActionState::Succeeded).await;
        assert!(f.server_uids("Trash").is_empty());
    })
    .await;
}

/// ApplyAction ne sert qu'aux actions sur des messages : boîtes et envoi ont leurs requêtes
#[tokio::test]
async fn apply_action_refuses_mailbox_and_send_kinds() {
    within(async {
        let f = Fixture::start().await;
        let mailbox = MailboxId::from_raw("box-work");
        let draft = DraftId::from_raw("d-1");
        let kinds = [
            ActionKind::DeleteMailbox { mailbox },
            ActionKind::Send { draft },
        ];
        for kind in kinds {
            let request = apply_request(kind, &["msg-1"], "k1");
            assert_eq!(f.refused_code(request).await, "invalid_input.parameter");
        }
        assert!(f.actions(&[]).await.is_empty());
    })
    .await;
}
