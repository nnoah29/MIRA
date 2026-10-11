#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Annulation d'une action : possible tant qu'elle est en attente, refusée ensuite.

mod support;
mod support_actions;
mod support_mirror;

use mira_core::actions::cancel;
use mira_vocabulary::{AccountId, ActionId, ActionKind, ActionState, EventKind, MailboxId};
use support_actions::inspect::{action, boxes_of, force_state, mailbox, message};
use support_actions::requests::submit_folders;
use support_actions::world::World;

/// Annuler dans le délai défait l'effet local, passe l'action à « annulée » et émet l'événement
#[test]
fn cancel_within_delay_reverts_the_local_effect() {
    let world = World::seeded();
    let id = submit_folders(&world, ActionKind::Trash, &["msg-1"], "k1");
    assert_eq!(boxes_of(&world.db, "msg-1"), ["box-trash"]);
    world.clock.advance_millis(2_000);
    cancel(&world.ctx(), &id).unwrap();
    assert_eq!(boxes_of(&world.db, "msg-1"), ["box-in"]);
    assert_eq!(mailbox(&world.db, "box-in").total, 3);
    assert_eq!(mailbox(&world.db, "box-trash").total, 0);
    assert_eq!(action(&world.db, &id).state, ActionState::Cancelled);
    let last = world.sink.kinds().pop().unwrap();
    assert_eq!(
        last,
        EventKind::ActionCancelled {
            account: AccountId::from_raw("acc-1"),
            action: id
        }
    );
}

/// Une action encore en attente après son délai (le rejeu ne l'a pas prise) reste annulable
#[test]
fn cancel_after_delay_still_works_while_pending() {
    let world = World::seeded();
    let id = submit_folders(
        &world,
        ActionKind::MarkRead { read: true },
        &["msg-1"],
        "k1",
    );
    world.clock.advance_millis(60_000);
    cancel(&world.ctx(), &id).unwrap();
    assert!(!message(&world.db, "msg-1").flags.seen);
}

/// Une action en cours, réussie, échouée ou déjà annulée n'est plus annulable
#[test]
fn cancel_refuses_actions_that_are_no_longer_pending() {
    let world = World::seeded();
    let id = submit_folders(
        &world,
        ActionKind::MarkRead { read: true },
        &["msg-1"],
        "k1",
    );
    for state in [
        ActionState::InProgress,
        ActionState::Succeeded,
        ActionState::Failed,
        ActionState::Cancelled,
        ActionState::Uncertain,
    ] {
        force_state(&world, &id, state);
        let error = cancel(&world.ctx(), &id).unwrap_err();
        assert_eq!(error.code, "incompatible_state.action_not_cancellable");
    }
    assert!(message(&world.db, "msg-1").flags.seen, "effet conservé");
}

/// Annuler deux fois : la seconde est refusée
#[test]
fn cancelling_twice_is_refused_the_second_time() {
    let world = World::seeded();
    let id = submit_folders(&world, ActionKind::Archive, &["msg-2"], "k1");
    cancel(&world.ctx(), &id).unwrap();
    let error = cancel(&world.ctx(), &id).unwrap_err();
    assert_eq!(error.code, "incompatible_state.action_not_cancellable");
    assert_eq!(boxes_of(&world.db, "msg-2"), ["box-in"]);
}

/// Une action inconnue donne `not_found.action`
#[test]
fn cancel_unknown_action_is_not_found() {
    let world = World::seeded();
    let error = cancel(&world.ctx(), &ActionId::from_raw("nope")).unwrap_err();
    assert_eq!(error.code, "not_found.action");
}

/// Annuler la création d'une boîte la supprime ; annuler une suppression la restaure avec son contenu
#[test]
fn cancel_restores_deleted_mailbox_and_removes_created_one() {
    let world = World::seeded();
    let delete = ActionKind::DeleteMailbox {
        mailbox: MailboxId::from_raw("box-work"),
    };
    let id = submit_folders(&world, delete, &[], "k1");
    assert!(boxes_of(&world.db, "msg-3") == ["box-in"]);
    cancel(&world.ctx(), &id).unwrap();
    assert_eq!(boxes_of(&world.db, "msg-3"), ["box-in", "box-work"]);
    assert_eq!(mailbox(&world.db, "box-work").total, 1);
    let create = ActionKind::CreateMailbox {
        name: "Temp".to_string(),
        parent: None,
    };
    let created = submit_folders(&world, create, &[], "k2");
    cancel(&world.ctx(), &created).unwrap();
    let found = world
        .db
        .read(|tx| tx.find_mailbox_by_name(&AccountId::from_raw("acc-1"), "Temp"))
        .unwrap();
    assert!(found.is_none());
}
