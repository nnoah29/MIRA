#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Actions sur les boîtes refusées avant la mise en file : boîtes système, noms, comptes.

mod support;
mod support_actions;
mod support_mirror;

use mira_core::actions::submit;
use mira_vocabulary::{ActionKind, MailboxId};
use support_actions::inspect::{mailbox, queue_len};
use support_actions::quirks::folders_quirks;
use support_actions::requests::request;
use support_actions::world::World;

/// Soumet et rend le code de l'erreur attendue, en vérifiant que la file est restée vide
fn refused_code(world: &World, kind: ActionKind, targets: &[&str]) -> String {
    let error = submit(
        &world.ctx(),
        &folders_quirks(),
        request(kind, targets, "k1"),
    )
    .unwrap_err();
    assert_eq!(queue_len(&world.db), 0);
    error.code
}

/// Une boîte système (réception, envoyés, brouillons, corbeille, spam) ne se renomme pas
#[test]
fn renaming_a_system_mailbox_is_refused() {
    let world = World::seeded();
    for id in ["box-in", "box-sent", "box-trash", "box-junk"] {
        let kind = ActionKind::RenameMailbox {
            mailbox: MailboxId::from_raw(id),
            name: "Autre".to_string(),
        };
        assert_eq!(
            refused_code(&world, kind, &[]),
            "incompatible_state.system_mailbox"
        );
    }
    assert_eq!(mailbox(&world.db, "box-in").name, "INBOX");
}

/// Une boîte système ne se supprime pas
#[test]
fn deleting_a_system_mailbox_is_refused() {
    let world = World::seeded();
    let kind = ActionKind::DeleteMailbox {
        mailbox: MailboxId::from_raw("box-trash"),
    };
    assert_eq!(
        refused_code(&world, kind, &[]),
        "incompatible_state.system_mailbox"
    );
}

/// Un nom vide ou déjà pris est refusé
#[test]
fn empty_or_duplicate_mailbox_names_are_refused() {
    let world = World::seeded();
    let empty = ActionKind::CreateMailbox {
        name: "  ".to_string(),
        parent: None,
    };
    assert_eq!(refused_code(&world, empty, &[]), "invalid_input.parameter");
    let duplicate = ActionKind::CreateMailbox {
        name: "Projects".to_string(),
        parent: None,
    };
    assert_eq!(
        refused_code(&world, duplicate, &[]),
        "invalid_input.parameter"
    );
    let rename = ActionKind::RenameMailbox {
        mailbox: MailboxId::from_raw("box-work"),
        name: "Projects".to_string(),
    };
    assert_eq!(refused_code(&world, rename, &[]), "invalid_input.parameter");
}

/// Une boîte d'un autre compte ou inconnue donne `not_found.mailbox`, aussi comme destination d'un déplacement
#[test]
fn foreign_or_unknown_mailbox_is_refused() {
    let world = World::seeded();
    for id in ["box-2-in", "box-nope"] {
        let delete = ActionKind::DeleteMailbox {
            mailbox: MailboxId::from_raw(id),
        };
        assert_eq!(refused_code(&world, delete, &[]), "not_found.mailbox");
        let to = MailboxId::from_raw(id);
        let moved = ActionKind::Move { to };
        assert_eq!(refused_code(&world, moved, &["msg-1"]), "not_found.mailbox");
    }
}

/// Une action sur une boîte ne prend pas de messages
#[test]
fn mailbox_actions_reject_message_targets() {
    let world = World::seeded();
    let kind = ActionKind::EmptyMailbox {
        mailbox: MailboxId::from_raw("box-work"),
    };
    assert_eq!(
        refused_code(&world, kind, &["msg-1"]),
        "invalid_input.parameter"
    );
}
