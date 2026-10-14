#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Début d'une tentative de rejeu : conditionné à l'état « en attente » (course avec l'annulation).

mod support;
mod support_actions;
mod support_mirror;

use mira_core::actions::cancel;
use mira_vocabulary::{ActionKind, ActionState};
use support_actions::inspect::{action, force_state};
use support_actions::requests::submit_folders;
use support_actions::world::World;

/// Une action annulée entre la lecture de la file et le début de la tentative n'est pas relancée
#[test]
fn regression_cancelled_action_cannot_begin_an_attempt() {
    let world = World::seeded();
    let id = submit_folders(&world, ActionKind::Trash, &["msg-1"], "k1");
    cancel(&world.ctx(), &id).unwrap();
    let began = world.db.write(|tx| tx.begin_action_attempt(&id)).unwrap();
    assert!(!began);
    assert_eq!(action(&world.db, &id).state, ActionState::Cancelled);
}

/// Une action en attente passe à « en cours » ; une action déjà en cours ou terminée n'est pas reprise
#[test]
fn begin_attempt_only_moves_pending_actions_to_in_progress() {
    let world = World::seeded();
    let id = submit_folders(&world, ActionKind::Archive, &["msg-2"], "k1");
    assert!(world.db.write(|tx| tx.begin_action_attempt(&id)).unwrap());
    assert_eq!(action(&world.db, &id).state, ActionState::InProgress);
    for state in [ActionState::InProgress, ActionState::Succeeded] {
        force_state(&world, &id, state);
        assert!(!world.db.write(|tx| tx.begin_action_attempt(&id)).unwrap());
        assert_eq!(action(&world.db, &id).state, state);
    }
}

/// Une action inconnue est signalée comme introuvable
#[test]
fn begin_attempt_on_unknown_action_is_not_found() {
    let world = World::seeded();
    let unknown = mira_vocabulary::ActionId::from_raw("absente");
    let error = world
        .db
        .write(|tx| tx.begin_action_attempt(&unknown))
        .unwrap_err();
    assert_eq!(error.code, "not_found.action");
}
