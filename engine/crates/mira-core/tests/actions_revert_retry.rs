#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Échec définitif (invariant 22) et relance d'une action.

mod support;
mod support_actions;
mod support_mirror;

use mira_core::actions::{retry, revert_local_effect};
use mira_vocabulary::{ActionId, ActionKind, ActionState, MailboxId, MessageId};
use support_actions::inspect::{action, boxes_of, mailbox, message};
use support_actions::requests::submit_folders;
use support_actions::world::World;

/// Fait échouer l'action comme le rejeu : effet local annulé puis état « échouée »
fn fail_definitively(world: &World, id: &ActionId) {
    world
        .db
        .write(|tx| {
            let pending = tx.get_action(id)?.unwrap();
            revert_local_effect(tx, &pending)?;
            tx.set_action_state(id, ActionState::Failed)
        })
        .unwrap();
}

/// Invariant 22 : après un échec définitif, l'effet local est annulé et l'interface revient à l'état réel
#[test]
fn invariant_22_definitive_failure_reverts_local_effect() {
    let world = World::seeded();
    let read = submit_folders(
        &world,
        ActionKind::MarkRead { read: true },
        &["msg-1"],
        "k1",
    );
    let moved = submit_folders(&world, ActionKind::Trash, &["msg-2"], "k2");
    fail_definitively(&world, &read);
    fail_definitively(&world, &moved);
    assert!(!message(&world.db, "msg-1").flags.seen);
    assert_eq!(mailbox(&world.db, "box-in").unread, 3);
    assert_eq!(boxes_of(&world.db, "msg-2"), ["box-in"]);
    assert_eq!(mailbox(&world.db, "box-trash").total, 0);
}

/// Défaire un déplacement rend à l'apparition son UID serveur d'origine
#[test]
fn reverting_a_move_restores_the_original_server_uid() {
    let world = World::seeded();
    let to = MailboxId::from_raw("box-proj");
    let id = submit_folders(&world, ActionKind::Move { to }, &["msg-2"], "k1");
    fail_definitively(&world, &id);
    let restored = world
        .db
        .read(|tx| tx.appearance_by_uid(&MailboxId::from_raw("box-in"), 2))
        .unwrap()
        .unwrap();
    assert_eq!(restored.message, MessageId::from_raw("msg-2"));
    assert!(!restored.needs_revalidation);
    let gone = world
        .db
        .read(|tx| {
            tx.get_appearance(
                &MessageId::from_raw("msg-2"),
                &MailboxId::from_raw("box-proj"),
            )
        })
        .unwrap();
    assert!(gone.is_none());
}

/// Défaire deux fois ne change rien de plus
#[test]
fn reverting_twice_is_harmless() {
    let world = World::seeded();
    let id = submit_folders(
        &world,
        ActionKind::MarkFlagged { flagged: true },
        &["msg-1"],
        "k1",
    );
    fail_definitively(&world, &id);
    world
        .db
        .write(|tx| {
            tx.update_message_flags(
                &MessageId::from_raw("msg-1"),
                mira_vocabulary::Flags {
                    flagged: true,
                    ..Default::default()
                },
            )
        })
        .unwrap();
    fail_definitively(&world, &id);
    assert!(
        message(&world.db, "msg-1").flags.flagged,
        "pas de second défaire"
    );
}

/// Relancer une action échouée la remet en attente, immédiatement rejouable, avec son effet local
#[test]
fn retry_requeues_a_failed_action_and_reapplies_its_effect() {
    let world = World::seeded();
    let id = submit_folders(&world, ActionKind::Trash, &["msg-1"], "k1");
    world
        .db
        .write(|tx| tx.record_action_failure(&id, "réseau", world.now()))
        .unwrap();
    fail_definitively(&world, &id);
    assert_eq!(boxes_of(&world.db, "msg-1"), ["box-in"]);
    world.clock.advance_millis(30_000);
    retry(&world.ctx(), &id).unwrap();
    let stored = action(&world.db, &id);
    assert_eq!(stored.state, ActionState::Pending);
    assert_eq!(stored.attempts, 0);
    assert_eq!(stored.last_error, None);
    assert_eq!(stored.not_before, world.now());
    assert_eq!(boxes_of(&world.db, "msg-1"), ["box-trash"]);
}

/// Seule une action échouée se relance ; l'effet d'une relance peut à son tour être défait
#[test]
fn retry_refuses_non_failed_actions() {
    let world = World::seeded();
    let id = submit_folders(
        &world,
        ActionKind::MarkRead { read: true },
        &["msg-1"],
        "k1",
    );
    let error = retry(&world.ctx(), &id).unwrap_err();
    assert_eq!(error.code, "incompatible_state.action_not_retryable");
    fail_definitively(&world, &id);
    retry(&world.ctx(), &id).unwrap();
    fail_definitively(&world, &id);
    assert!(!message(&world.db, "msg-1").flags.seen);
}
