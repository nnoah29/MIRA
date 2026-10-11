#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Effets locaux sur les libellés (D-17) : ajout, retrait, refus sur un compte à dossiers.

mod support;
mod support_actions;
mod support_mirror;

use mira_core::actions::submit;
use mira_vocabulary::{ActionKind, MailboxId};
use support_actions::inspect::{boxes_of, mailbox, queue_len};
use support_actions::quirks::{folders_quirks, labels_quirks};
use support_actions::requests::{request, submit_folders};
use support_actions::world::World;

/// Soumet sur un service à libellés
fn submit_labels(world: &World, kind: ActionKind, targets: &[&str], key: &str) {
    submit(&world.ctx(), &labels_quirks(), request(kind, targets, key)).unwrap();
}

/// Ajouter un libellé garde les autres apparitions du message
#[test]
fn add_label_keeps_existing_mailboxes() {
    let world = World::seeded();
    let mailbox_id = MailboxId::from_raw("box-proj");
    submit_labels(
        &world,
        ActionKind::AddToMailbox {
            mailbox: mailbox_id,
        },
        &["msg-1"],
        "k1",
    );
    assert_eq!(boxes_of(&world.db, "msg-1"), ["box-in", "box-proj"]);
    assert_eq!(mailbox(&world.db, "box-proj").total, 1);
    assert_eq!(mailbox(&world.db, "box-in").total, 3);
}

/// D-17 : ajouter un second libellé sur un compte à dossiers est refusé, sans action ni effet
#[test]
fn d17_add_label_on_folder_account_is_refused() {
    let world = World::seeded();
    let kind = ActionKind::AddToMailbox {
        mailbox: MailboxId::from_raw("box-proj"),
    };
    let req = request(kind, &["msg-1"], "k1");
    let error = submit(&world.ctx(), &folders_quirks(), req).unwrap_err();
    assert_eq!(error.code, "incompatible_state.multiple_labels_unsupported");
    assert_eq!(queue_len(&world.db), 0);
    assert_eq!(boxes_of(&world.db, "msg-1"), ["box-in"]);
}

/// Retirer un libellé supprime seulement cette apparition
#[test]
fn remove_label_drops_only_that_appearance() {
    let world = World::seeded();
    let kind = ActionKind::RemoveFromMailbox {
        mailbox: MailboxId::from_raw("box-work"),
    };
    submit_labels(&world, kind, &["msg-3"], "k1");
    assert_eq!(boxes_of(&world.db, "msg-3"), ["box-in"]);
    assert_eq!(mailbox(&world.db, "box-work").total, 0);
}

/// Sur un compte à dossiers, retirer un message de sa boîte revient à l'archiver
#[test]
fn remove_from_mailbox_on_folder_account_archives() {
    let world = World::seeded();
    let kind = ActionKind::RemoveFromMailbox {
        mailbox: MailboxId::from_raw("box-in"),
    };
    submit_folders(&world, kind, &["msg-1"], "k1");
    assert_eq!(boxes_of(&world.db, "msg-1"), ["box-arch"]);
}
