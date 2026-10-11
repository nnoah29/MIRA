#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Effet local d'« archiver » : déplacement ou retrait de la réception, selon le service.

mod support;
mod support_actions;
mod support_mirror;

use mira_core::actions::submit;
use mira_vocabulary::{ActionKind, MailboxId};
use support_actions::inspect::{boxes_of, queue_len};
use support_actions::quirks::{folders_quirks, labels_quirks};
use support_actions::requests::{request, submit_folders};
use support_actions::world::World;

/// Service à dossiers : le message passe dans la boîte d'archive
#[test]
fn archive_on_folder_service_moves_to_the_archive_mailbox() {
    let world = World::seeded();
    submit_folders(&world, ActionKind::Archive, &["msg-1"], "k1");
    assert_eq!(boxes_of(&world.db, "msg-1"), ["box-arch"]);
}

/// Service à libellés : seul le libellé de la réception est retiré, les autres restent
#[test]
fn archive_on_label_service_only_leaves_the_inbox() {
    let world = World::seeded();
    let req = request(ActionKind::Archive, &["msg-3"], "k1");
    submit(&world.ctx(), &labels_quirks(), req).unwrap();
    assert_eq!(boxes_of(&world.db, "msg-3"), ["box-work"]);
}

/// Sans boîte d'archive connue, l'archivage est refusé avant la mise en file (`server.mailbox_missing`)
#[test]
fn archive_without_an_archive_mailbox_is_refused() {
    let world = World::seeded();
    world
        .db
        .write(|tx| tx.delete_mailbox(&MailboxId::from_raw("box-arch")))
        .unwrap();
    let req = request(ActionKind::Archive, &["msg-1"], "k1");
    let error = submit(&world.ctx(), &folders_quirks(), req).unwrap_err();
    assert_eq!(error.code, "server.mailbox_missing");
    assert_eq!(queue_len(&world.db), 0);
}
