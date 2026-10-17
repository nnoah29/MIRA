//! Échec d'une action : erreur affichée, vue rechargée, annulation sûre.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod action_fixtures;
mod message_fixtures;
mod navigation_fixtures;
mod reader_fixtures;
mod sidebar_fixtures;

use action_fixtures::{
    action_reply, reload_command, reload_reply, state_with_mailboxes, undo_command, undo_reply,
};
use mira_tui::update::{Intent, update};
use navigation_fixtures::press;
use reader_fixtures::offline_error;

/// Archive qui échoue : message d'état, rechargement de la vue, ligne restaurée à la réponse.
#[test]
fn failed_action_reloads_view_and_restores_row() {
    let mut state = state_with_mailboxes();
    press(&mut state, Intent::Archive);
    assert_eq!(state.messages.len(), 1);
    let commands = update(
        &mut state,
        action_reply("tui-1-act-1", Err(offline_error())),
    );
    assert!(state.status.message.is_some());
    assert!(commands.contains(&reload_command()), "{commands:?}");
    update(&mut state, reload_reply(2));
    assert_eq!(state.messages.len(), 2);
}

/// L'échec d'une action plus ancienne ne recharge rien.
#[test]
fn failure_of_superseded_action_does_not_reload() {
    let mut state = state_with_mailboxes();
    press(&mut state, Intent::Archive);
    press(&mut state, Intent::Archive);
    let commands = update(
        &mut state,
        action_reply("tui-1-act-1", Err(offline_error())),
    );
    assert!(!commands.contains(&reload_command()));
}

/// `z` juste après l'envoi d'une action, avant sa réponse, n'annule rien.
#[test]
fn undo_while_action_in_flight_sends_nothing() {
    let mut state = state_with_mailboxes();
    press(&mut state, Intent::Archive);
    update(&mut state, action_reply("tui-1-act-1", Ok("old")));
    press(&mut state, Intent::Archive);
    assert!(press(&mut state, Intent::Undo).is_empty());
    assert!(state.status.message.is_some());
}

/// Après l'échec de la dernière action, `z` n'annule rien d'ancien.
#[test]
fn undo_after_failed_action_does_not_undo_older_one() {
    let mut state = state_with_mailboxes();
    press(&mut state, Intent::Archive);
    update(&mut state, action_reply("tui-1-act-1", Ok("old")));
    press(&mut state, Intent::Archive);
    update(
        &mut state,
        action_reply("tui-1-act-2", Err(offline_error())),
    );
    assert!(press(&mut state, Intent::Undo).is_empty());
}

/// Une annulation qui échoue garde l'action : `z` peut réessayer le même identifiant.
#[test]
fn failed_undo_restores_last_action() {
    let mut state = state_with_mailboxes();
    press(&mut state, Intent::Archive);
    update(&mut state, action_reply("tui-1-act-1", Ok("act-9")));
    press(&mut state, Intent::Undo);
    update(&mut state, undo_reply("act-9", Err(offline_error())));
    assert!(state.actions.last_action.is_some());
    assert_eq!(press(&mut state, Intent::Undo), vec![undo_command("act-9")]);
}
