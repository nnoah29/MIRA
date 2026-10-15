//! Fermeture d'un brouillon : aucune lettre isolée ne détruit le texte ; « enregistrer et fermer » après un enregistrement fini ferme.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod action_fixtures;
mod compose_fixtures;
mod input_fixtures;
mod message_fixtures;
mod navigation_fixtures;
mod reader_fixtures;
mod sidebar_fixtures;

use compose_fixtures::{compose, delete_command, draft, open_modified, saved_reply, screen};
use crossterm::event::{KeyCode, KeyEvent};
use input_fixtures::{code, key, type_keys};
use mira_tui::commands::Command;
use mira_tui::state::Popup;
use mira_tui::state::compose::Field;
use mira_tui::update::{ComposeIntent, Intent, update};
use navigation_fixtures::press;

/// Échap par réflexe puis « ok d » tapé : rien n'est abandonné, le texte et la fenêtre restent.
#[test]
fn lowercase_d_in_close_confirmation_destroys_nothing() {
    let mut state = open_modified();
    let mut keys = vec![code(KeyCode::Esc)];
    keys.extend("ok d".chars().map(key));
    let commands = type_keys(&mut state, &keys);
    assert!(commands.is_empty(), "{commands:?}");
    assert_eq!(state.popup, Popup::ConfirmCloseDraft);
    let open = state.compose.open().unwrap();
    assert_eq!(open.text(Field::Subject), "Re: Bonjour!");
}

/// `D` ouvre une seconde confirmation ; `D` encore abandonne ; Échap y revient à l'éditeur sans rien perdre.
#[test]
fn capital_d_twice_is_the_deliberate_discard() {
    let mut state = open_modified();
    let esc_then_d = [code(KeyCode::Esc), key('D')];
    assert!(type_keys(&mut state, &esc_then_d).is_empty());
    assert_eq!(state.popup, Popup::ConfirmDiscardDraft);
    assert!(type_keys(&mut state, &[key('d'), code(KeyCode::Enter)]).is_empty());
    assert!(type_keys(&mut state, &[code(KeyCode::Esc)]).is_empty());
    assert_eq!(state.popup, Popup::Closed);
    assert!(state.compose.open().unwrap().is_dirty(Field::Subject));
    let keys: [KeyEvent; 3] = [code(KeyCode::Esc), key('D'), key('D')];
    assert_eq!(type_keys(&mut state, &keys), vec![delete_command("d1")]);
    assert!(state.compose.is_closed());
}

/// Ctrl+s en vol, Échap, réponse du moteur, puis `s` : tout est enregistré, donc l'éditeur ferme.
#[test]
fn save_and_close_after_the_save_landed_closes() {
    let mut state = open_modified();
    compose(&mut state, ComposeIntent::Save);
    compose(&mut state, ComposeIntent::Close);
    assert_eq!(state.popup, Popup::ConfirmCloseDraft);
    let saved = draft("d1", &["bob@example.org"], "Re: Bonjour!", "Corps");
    update(&mut state, saved_reply("d1", Ok(saved)));
    assert!(compose(&mut state, ComposeIntent::SaveAndClose).is_empty());
    assert!(state.compose.is_closed());
    assert_eq!(state.popup, Popup::Closed);
}

/// Même chose depuis Ctrl+c : la sortie demandée a lieu.
#[test]
fn save_and_close_after_the_save_landed_still_quits() {
    let mut state = open_modified();
    compose(&mut state, ComposeIntent::Save);
    press(&mut state, Intent::Quit);
    let saved = draft("d1", &["bob@example.org"], "Re: Bonjour!", "Corps");
    update(&mut state, saved_reply("d1", Ok(saved)));
    let commands = compose(&mut state, ComposeIntent::SaveAndClose);
    assert_eq!(commands, vec![Command::Quit]);
    assert!(state.should_quit);
}

/// Le texte de la fenêtre dit vrai : modifications non enregistrées, ou seulement un enregistrement en cours.
#[test]
fn close_confirmation_text_is_exact() {
    let mut state = open_modified();
    compose(&mut state, ComposeIntent::Close);
    let text = screen(&state, 100, 30).0;
    assert!(text.contains("ne sont pas enregistrées"), "{text}");
    assert!(text.contains("D : abandonner"), "{text}");
    let mut state = open_modified();
    compose(&mut state, ComposeIntent::Save);
    compose(&mut state, ComposeIntent::Close);
    let text = screen(&state, 100, 30).0;
    assert!(text.contains("Enregistrement en cours"), "{text}");
    assert!(!text.contains("ne sont pas enregistrées"), "{text}");
}
