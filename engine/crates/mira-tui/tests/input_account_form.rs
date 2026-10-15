//! Clavier du formulaire d'ajout de compte : ouverture par `n`, texte, déplacement, confirmation de fermeture.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod action_fixtures;
mod input_fixtures;
mod message_fixtures;
mod navigation_fixtures;
mod reader_fixtures;
mod sidebar_fixtures;

use action_fixtures::state_with_mailboxes;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use input_fixtures::{code, ctrl, key, map, released};
use mira_tui::input::{KeyContext, context_of};
use mira_tui::state::{AppState, Pane};
use mira_tui::update::{AccountFormIntent, EditKey, Intent};

/// Intention d'une touche du formulaire.
fn form_intent(intent: AccountFormIntent) -> Option<Intent> {
    Some(Intent::AccountForm(intent))
}

/// `n` ouvre le formulaire depuis la barre latérale, et partout tant qu'aucun compte n'existe ; pas depuis la liste.
#[test]
fn n_opens_the_form_from_the_sidebar_or_the_welcome_screen() {
    let mut state = state_with_mailboxes();
    state.focus = Pane::Sidebar;
    assert_eq!(map(&state, key('n')), form_intent(AccountFormIntent::Open));
    state.focus = Pane::List;
    assert_eq!(map(&state, key('n')), None);
    let mut empty = AppState::new(100, 30, 1);
    empty.focus = Pane::Reader;
    assert_eq!(map(&empty, key('n')), form_intent(AccountFormIntent::Open));
}

/// Formulaire ouvert, tout caractère imprimable (même `q`, `n`, `?`, `/`) est du texte, AltGr compris.
#[test]
fn printable_keys_are_text_in_the_form() {
    let mut state = AppState::new(100, 30, 1);
    state.account_form.open();
    assert_eq!(context_of(&state), KeyContext::AccountForm);
    for c in ['q', 'n', '?', '/', ':', 'é', 'D', '@'] {
        let typed = form_intent(AccountFormIntent::Edit(EditKey::Char(c)));
        assert_eq!(map(&state, key(c)), typed, "{c}");
    }
    let alt_gr = KeyEvent::new(
        KeyCode::Char('@'),
        KeyModifiers::CONTROL | KeyModifiers::ALT,
    );
    assert_eq!(
        map(&state, alt_gr),
        form_intent(AccountFormIntent::Edit(EditKey::Char('@')))
    );
    assert_eq!(map(&state, released('a')), None);
}

/// Tab, Maj+Tab, Entrée, Échap, Ctrl+t et l'édition ont leur intention ; Ctrl+c quitte toujours.
#[test]
fn navigation_keys_map_to_form_intents() {
    let mut state = AppState::new(100, 30, 1);
    state.account_form.open();
    let table = [
        (code(KeyCode::Tab), AccountFormIntent::NextField),
        (code(KeyCode::BackTab), AccountFormIntent::PrevField),
        (code(KeyCode::Enter), AccountFormIntent::Enter),
        (code(KeyCode::Esc), AccountFormIntent::Close),
        (ctrl('t'), AccountFormIntent::CycleSecurity),
        (
            code(KeyCode::Backspace),
            AccountFormIntent::Edit(EditKey::Backspace),
        ),
    ];
    for (event, expected) in table {
        assert_eq!(map(&state, event), form_intent(expected), "{event:?}");
    }
    assert_eq!(map(&state, ctrl('c')), Some(Intent::Quit));
    assert_eq!(map(&state, ctrl('r')), None);
}

/// Sous la confirmation de fermeture, seuls Entrée/`y` et Échap/`n` répondent.
#[test]
fn close_confirmation_has_its_own_keys() {
    let mut state = AppState::new(100, 30, 1);
    state.account_form.open();
    state.account_form.form_mut().unwrap().confirm_close = true;
    assert_eq!(context_of(&state), KeyContext::AccountFormConfirm);
    assert_eq!(
        map(&state, key('y')),
        form_intent(AccountFormIntent::ConfirmClose)
    );
    assert_eq!(
        map(&state, code(KeyCode::Enter)),
        form_intent(AccountFormIntent::ConfirmClose)
    );
    assert_eq!(
        map(&state, key('n')),
        form_intent(AccountFormIntent::KeepEditing)
    );
    assert_eq!(
        map(&state, code(KeyCode::Esc)),
        form_intent(AccountFormIntent::KeepEditing)
    );
    assert_eq!(map(&state, key('a')), None);
}
