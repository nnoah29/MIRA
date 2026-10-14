//! Quitter avec un brouillon modifié passe par la confirmation de fermeture.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod action_fixtures;
mod compose_fixtures;
mod message_fixtures;
mod navigation_fixtures;
mod reader_fixtures;
mod sidebar_fixtures;

use compose_fixtures::{
    compose, delete_command, draft, open_modified, open_reply, saved_reply, subject_only,
    update_command,
};
use mira_tui::commands::Command;
use mira_tui::state::Popup;
use mira_tui::update::{ComposeIntent, Intent, update};
use navigation_fixtures::press;
use reader_fixtures::offline_error;

/// Quitter avec des modifications non enregistrées demande confirmation au lieu de quitter.
#[test]
fn quit_with_unsaved_changes_asks_confirmation() {
    let mut state = open_modified();
    let commands = press(&mut state, Intent::Quit);
    assert!(commands.is_empty());
    assert_eq!(state.popup, Popup::ConfirmCloseDraft);
    assert!(!state.should_quit);
    assert!(press(&mut state, Intent::Quit).is_empty());
    assert!(!state.should_quit);
}

/// Sans modification, la sortie est directe comme avant.
#[test]
fn quit_without_changes_quits_directly() {
    let mut state = open_reply();
    assert_eq!(press(&mut state, Intent::Quit), vec![Command::Quit]);
    assert!(state.should_quit);
}

/// Abandonner depuis la sortie supprime le brouillon jamais enregistré puis quitte.
#[test]
fn discard_after_quit_deletes_the_draft_then_quits() {
    let mut state = open_modified();
    press(&mut state, Intent::Quit);
    let commands = compose(&mut state, ComposeIntent::Discard);
    assert_eq!(commands, vec![delete_command("d1"), Command::Quit]);
    assert!(state.should_quit && state.compose.is_closed());
}

/// Enregistrer depuis la sortie envoie les changements, puis quitte à la confirmation du moteur.
#[test]
fn save_after_quit_saves_then_quits() {
    let mut state = open_modified();
    press(&mut state, Intent::Quit);
    let commands = compose(&mut state, ComposeIntent::SaveAndClose);
    assert_eq!(
        commands,
        vec![update_command("d1", subject_only("Re: Bonjour!"))]
    );
    assert!(!state.should_quit);
    let saved = draft("d1", &["bob@example.org"], "Re: Bonjour!", "Corps");
    let commands = update(&mut state, saved_reply("d1", Ok(saved)));
    assert_eq!(commands, vec![Command::Quit]);
    assert!(state.should_quit);
}

/// Continuer reste dans l'éditeur, et une fermeture ultérieure ne quitte pas l'application.
#[test]
fn continue_after_quit_stays_and_later_close_does_not_quit() {
    let mut state = open_modified();
    press(&mut state, Intent::Quit);
    assert!(press(&mut state, Intent::Cancel).is_empty());
    assert!(state.compose.open().is_some() && !state.should_quit);
    compose(&mut state, ComposeIntent::Close);
    let commands = compose(&mut state, ComposeIntent::Discard);
    assert_eq!(commands, vec![delete_command("d1")]);
    assert!(!state.should_quit);
}

/// Un enregistrement refusé annule la sortie : l'éditeur reste ouvert.
#[test]
fn failed_save_after_quit_cancels_the_exit() {
    let mut state = open_modified();
    press(&mut state, Intent::Quit);
    compose(&mut state, ComposeIntent::SaveAndClose);
    let commands = update(&mut state, saved_reply("d1", Err(offline_error())));
    assert!(commands.is_empty());
    assert!(state.compose.open().is_some() && !state.should_quit);
}
