#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Traduction des actions en commandes IMAP : archive, corbeille, spam, suppression, libellés, gestion des boîtes.

mod support;
mod support_actions;
mod support_imap;
mod support_mirror;
mod support_replay;
mod support_smtp;

use mira_core::actions::{UNDO_DELAY_MILLIS, submit};
use mira_testkit::imap::CapabilitySet;
use mira_vocabulary::{ActionKind, MailboxId, ProviderQuirks};
use support_actions::quirks::labels_quirks;
use support_actions::requests::{request, submit_folders};
use support_imap::within;
use support_replay::fixture::Fixture;
use support_replay::observe::{command_count, server_message_ids};
use support_replay::presets::replay_on_msg_1;

/// Archiver sur un service à dossiers déplace le message vers la boîte Archive
#[tokio::test]
async fn archive_moves_the_message_to_the_archive_mailbox() {
    within(async {
        let fx = replay_on_msg_1(ActionKind::Archive).await;
        assert_eq!(server_message_ids(&fx, "Archive"), ["msg-1@example.org"]);
        assert!(!server_message_ids(&fx, "INBOX").contains(&"msg-1@example.org".to_string()));
    })
    .await;
}

/// Archiver sur un service à libellés retire seulement le libellé de la réception
#[tokio::test]
async fn archive_on_a_labels_service_removes_the_inbox_label() {
    within(async {
        let caps = CapabilitySet::full().with_label_extensions();
        let fx = Fixture::start(caps, labels_quirks()).await;
        let archive = request(ActionKind::Archive, &["msg-1"], "k-arch");
        submit(&fx.world.ctx(), &labels_quirks(), archive).unwrap();
        fx.advance(UNDO_DELAY_MILLIS + 1);
        let mut client = fx.client().await;
        let report = fx.replay(&mut client).await;
        assert_eq!((report.succeeded, report.failed), (1, 0));
        assert_eq!(command_count(&fx, "-X-GM-LABELS.SILENT (\"\\\\Inbox\")"), 1);
        assert_eq!(command_count(&fx, "MOVE"), 0);
    })
    .await;
}

/// Corbeille et spam déplacent vers les boîtes de rôle Trash et Junk
#[tokio::test]
async fn trash_and_spam_move_to_their_role_mailboxes() {
    within(async {
        let fx = replay_on_msg_1(ActionKind::Trash).await;
        assert_eq!(server_message_ids(&fx, "Trash"), ["msg-1@example.org"]);
        let spam = ActionKind::MarkSpam { spam: true };
        submit_folders(&fx.world, spam, &["msg-2"], "k-2");
        fx.advance(UNDO_DELAY_MILLIS + 1);
        let mut client = fx.client().await;
        fx.replay(&mut client).await;
        assert_eq!(server_message_ids(&fx, "Junk"), ["msg-2@example.org"]);
    })
    .await;
}

/// Suppression définitive : `\Deleted` puis UID EXPUNGE, le message disparaît du serveur
#[tokio::test]
async fn delete_permanently_purges_the_message_from_the_server() {
    within(async {
        let fx = replay_on_msg_1(ActionKind::DeletePermanently).await;
        assert_eq!(command_count(&fx, "UID EXPUNGE"), 1);
        assert_eq!(
            server_message_ids(&fx, "INBOX"),
            ["msg-2@example.org", "msg-3@example.org"]
        );
    })
    .await;
}

/// Créer puis renommer une boîte
#[tokio::test]
async fn create_then_rename_a_mailbox() {
    within(async {
        let fx = Fixture::start(CapabilitySet::full(), ProviderQuirks::default()).await;
        let create = ActionKind::CreateMailbox {
            name: "Reports".to_string(),
            parent: None,
        };
        submit_folders(&fx.world, create, &[], "k-create");
        let rename = ActionKind::RenameMailbox {
            mailbox: MailboxId::from_raw("box-proj"),
            name: "Projets".to_string(),
        };
        submit_folders(&fx.world, rename, &[], "k-rename");
        fx.advance(UNDO_DELAY_MILLIS + 1);
        let mut client = fx.client().await;
        assert_eq!(fx.replay(&mut client).await.succeeded, 2);
        let names: Vec<String> = client
            .list_mailboxes()
            .await
            .unwrap()
            .into_iter()
            .map(|mailbox| mailbox.name)
            .collect();
        assert!(names.contains(&"Reports".to_string()));
        assert!(names.contains(&"Projets".to_string()));
        assert!(!names.contains(&"Projects".to_string()));
    })
    .await;
}

/// Vider une boîte purge ses messages ; la supprimer retire la boîte du serveur
#[tokio::test]
async fn empty_then_delete_a_mailbox() {
    within(async {
        let fx = Fixture::start(CapabilitySet::full(), ProviderQuirks::default()).await;
        let work = MailboxId::from_raw("box-work");
        let empty = ActionKind::EmptyMailbox {
            mailbox: work.clone(),
        };
        submit_folders(&fx.world, empty, &[], "k-empty");
        fx.advance(UNDO_DELAY_MILLIS + 1);
        let mut client = fx.client().await;
        fx.replay(&mut client).await;
        assert!(server_message_ids(&fx, "Work").is_empty());
        let delete = ActionKind::DeleteMailbox { mailbox: work };
        submit_folders(&fx.world, delete, &[], "k-delete");
        fx.advance(UNDO_DELAY_MILLIS + 1);
        assert_eq!(fx.replay(&mut client).await.succeeded, 1);
        let boxes = client.list_mailboxes().await.unwrap();
        assert!(boxes.iter().all(|mailbox| mailbox.name != "Work"));
    })
    .await;
}
