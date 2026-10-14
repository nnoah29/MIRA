//! Fermeture du brouillon : confirmation si des modifications ne sont pas enregistrées.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod action_fixtures;
mod compose_fixtures;
mod message_fixtures;
mod navigation_fixtures;
mod reader_fixtures;
mod sidebar_fixtures;

use compose_fixtures::{
    compose, delete_command, draft, open_modified, open_reply, saved_reply, subject_only,
    type_in_draft, update_command,
};
use mira_tui::state::Popup;
use mira_tui::state::compose::Field;
use mira_tui::update::{ComposeIntent, Intent, update};
use navigation_fixtures::press;

/// Échap avec des modifications non enregistrées demande confirmation et ne ferme rien.
#[test]
fn escape_with_unsaved_changes_asks_confirmation() {
    let mut state = open_modified();
    let commands = compose(&mut state, ComposeIntent::Close);
    assert!(commands.is_empty());
    assert_eq!(state.popup, Popup::ConfirmCloseDraft);
    assert!(state.compose.open().is_some());
    assert!(press(&mut state, Intent::Cancel).is_empty());
    assert_eq!(state.popup, Popup::Closed);
    assert!(state.compose.open().unwrap().is_dirty(Field::Subject));
}

/// Abandonner ferme l'éditeur et supprime le brouillon jamais enregistré par l'utilisateur.
#[test]
fn discard_closes_and_deletes_never_saved_draft() {
    let mut state = open_modified();
    compose(&mut state, ComposeIntent::Close);
    let commands = compose(&mut state, ComposeIntent::Discard);
    assert_eq!(commands, vec![delete_command("d1")]);
    assert!(state.compose.is_closed());
    assert_eq!(state.popup, Popup::Closed);
}

/// Sans modification, Échap ferme tout de suite et supprime le brouillon intact créé pour rien.
#[test]
fn escape_without_changes_closes_and_deletes_pristine_draft() {
    let mut state = open_reply();
    let commands = compose(&mut state, ComposeIntent::Close);
    assert_eq!(commands, vec![delete_command("d1")]);
    assert!(state.compose.is_closed());
    assert_eq!(state.popup, Popup::Closed);
}

/// Un brouillon déjà enregistré est gardé à la fermeture, même quand on abandonne la suite.
#[test]
fn saved_draft_is_kept_on_close_and_discard() {
    let mut state = open_modified();
    compose(&mut state, ComposeIntent::Save);
    let saved = draft("d1", &["bob@example.org"], "Re: Bonjour!", "Corps");
    update(&mut state, saved_reply("d1", Ok(saved)));
    assert!(compose(&mut state, ComposeIntent::Close).is_empty());
    assert!(state.compose.is_closed());
    let mut again = open_modified();
    compose(&mut again, ComposeIntent::Save);
    let saved = draft("d1", &["bob@example.org"], "Re: Bonjour!", "Corps");
    update(&mut again, saved_reply("d1", Ok(saved)));
    type_in_draft(&mut again, "?");
    compose(&mut again, ComposeIntent::Close);
    assert!(compose(&mut again, ComposeIntent::Discard).is_empty());
    assert!(again.compose.is_closed());
}

/// Enregistrer et fermer envoie les changements, puis ferme à la réponse du moteur.
#[test]
fn save_and_close_closes_when_the_engine_confirms() {
    let mut state = open_modified();
    compose(&mut state, ComposeIntent::Close);
    let commands = compose(&mut state, ComposeIntent::SaveAndClose);
    assert_eq!(
        commands,
        vec![update_command("d1", subject_only("Re: Bonjour!"))]
    );
    assert_eq!(state.popup, Popup::Closed);
    assert!(state.compose.open().is_some());
    let saved = draft("d1", &["bob@example.org"], "Re: Bonjour!", "Corps");
    let commands = update(&mut state, saved_reply("d1", Ok(saved)));
    assert!(commands.is_empty());
    assert!(state.compose.is_closed());
}

/// Entrée dans la confirmation enregistre ; un échec d'enregistrement garde l'éditeur ouvert.
#[test]
fn enter_saves_and_a_failed_save_keeps_the_editor_open() {
    let mut state = open_modified();
    compose(&mut state, ComposeIntent::Close);
    let commands = press(&mut state, Intent::Confirm);
    assert_eq!(commands.len(), 1);
    update(
        &mut state,
        saved_reply("d1", Err(reader_fixtures::offline_error())),
    );
    let open = state.compose.open().unwrap();
    assert!(open.is_dirty(Field::Subject));
    assert!(state.status.message.is_some());
}
