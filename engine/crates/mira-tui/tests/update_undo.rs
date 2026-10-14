//! Annulation de la dernière action et gestion des réponses tardives.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod action_fixtures;
mod message_fixtures;
mod navigation_fixtures;
mod reader_fixtures;
mod sidebar_fixtures;

use action_fixtures::{action_reply, state_with_mailboxes, undo_command, undo_reply};
use mira_tui::update::{Intent, update};
use navigation_fixtures::press;

/// Après la réponse d'une action, `Undo` annule cette action, une seule fois.
#[test]
fn undo_sends_undo_for_last_action() {
    let mut state = state_with_mailboxes();
    press(&mut state, Intent::Archive);
    update(&mut state, action_reply("tui-1-act-1", Ok("act-9")));
    let commands = press(&mut state, Intent::Undo);
    assert_eq!(commands, vec![undo_command("act-9")]);
    assert!(press(&mut state, Intent::Undo).is_empty());
}

/// Sans action mémorisée, `Undo` informe dans la barre d'état sans erreur ni commande.
#[test]
fn undo_without_action_shows_status_not_error() {
    let mut state = state_with_mailboxes();
    let commands = press(&mut state, Intent::Undo);
    assert!(commands.is_empty());
    assert!(state.status.message.is_some());
    assert!(!state.should_quit);
}

/// La réponse tardive d'une action plus ancienne ne remplace pas l'action à annuler.
#[test]
fn late_action_reply_does_not_replace_last_action() {
    let mut state = state_with_mailboxes();
    press(&mut state, Intent::Archive);
    press(&mut state, Intent::Archive);
    update(&mut state, action_reply("tui-1-act-2", Ok("act-2")));
    update(&mut state, action_reply("tui-1-act-1", Ok("act-1")));
    assert_eq!(press(&mut state, Intent::Undo), vec![undo_command("act-2")]);
}

/// Une réponse d'annulation réussie est confirmée dans la barre d'état.
#[test]
fn undo_reply_is_reported_in_status() {
    let mut state = state_with_mailboxes();
    update(&mut state, undo_reply("act-1", Ok(())));
    assert!(state.status.message.is_some());
}
