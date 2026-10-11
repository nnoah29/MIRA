#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Effets locaux sur les drapeaux (D-14) : portés par le message, compteurs à jour.

mod support;
mod support_actions;
mod support_mirror;

use mira_vocabulary::ActionKind;
use support_actions::inspect::{mailbox, message};
use support_actions::requests::submit_folders;
use support_actions::world::World;

/// Marquer lu met le drapeau sur le message et recalcule les non lus de la boîte
#[test]
fn mark_read_sets_the_flag_and_updates_unread_counter() {
    let world = World::seeded();
    assert_eq!(mailbox(&world.db, "box-in").unread, 3);
    submit_folders(
        &world,
        ActionKind::MarkRead { read: true },
        &["msg-1"],
        "k1",
    );
    assert!(message(&world.db, "msg-1").flags.seen);
    assert!(!message(&world.db, "msg-2").flags.seen);
    assert_eq!(mailbox(&world.db, "box-in").unread, 2);
    assert_eq!(mailbox(&world.db, "box-in").total, 3);
}

/// Marquer non lu fait l'inverse sur un message déjà lu
#[test]
fn mark_unread_clears_the_flag_and_restores_unread_counter() {
    let world = World::seeded();
    submit_folders(
        &world,
        ActionKind::MarkRead { read: true },
        &["msg-1"],
        "k1",
    );
    submit_folders(
        &world,
        ActionKind::MarkRead { read: false },
        &["msg-1"],
        "k2",
    );
    assert!(!message(&world.db, "msg-1").flags.seen);
    assert_eq!(mailbox(&world.db, "box-in").unread, 3);
}

/// D-14 : un message présent dans deux boîtes est lu une seule fois, et les deux compteurs suivent
#[test]
fn mark_read_applies_to_the_message_in_all_its_mailboxes() {
    let world = World::seeded();
    assert_eq!(mailbox(&world.db, "box-work").unread, 1);
    submit_folders(
        &world,
        ActionKind::MarkRead { read: true },
        &["msg-3"],
        "k1",
    );
    assert_eq!(mailbox(&world.db, "box-work").unread, 0);
    assert_eq!(mailbox(&world.db, "box-in").unread, 2);
}

/// Marquer suivi ne touche ni « lu » ni les compteurs
#[test]
fn mark_flagged_sets_only_the_flagged_flag() {
    let world = World::seeded();
    submit_folders(
        &world,
        ActionKind::MarkFlagged { flagged: true },
        &["msg-2"],
        "k1",
    );
    let flags = message(&world.db, "msg-2").flags;
    assert!(flags.flagged);
    assert!(!flags.seen);
    assert_eq!(mailbox(&world.db, "box-in").unread, 3);
}
