//! Pendant la création d'un brouillon, l'écran est modal : rien n'agit sur le courrier, Échap annule.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod action_fixtures;
mod compose_fixtures;
mod input_fixtures;
mod message_fixtures;
mod navigation_fixtures;
mod reader_fixtures;
mod search_fixtures;
mod sidebar_fixtures;

use action_fixtures::state_with_mailboxes;
use compose_fixtures::{compose, create_command, created_reply, delete_command, draft};
use crossterm::event::KeyCode;
use input_fixtures::{code, ctrl, key, map};
use mira_assembly::api::DraftMode;
use mira_tui::input::{KeyContext, context_of};
use mira_tui::state::Popup;
use mira_tui::update::{ComposeIntent, Intent, SearchIntent, update};
use navigation_fixtures::press;

/// Aucune intention sur le courrier, la recherche ou l'aide ne part tant que le brouillon n'est pas créé.
#[test]
fn creating_ignores_mail_actions_search_and_help() {
    let mut state = state_with_mailboxes();
    compose(&mut state, ComposeIntent::Reply);
    for intent in [
        Intent::Trash,
        Intent::Archive,
        Intent::DeletePermanently,
        Intent::ToggleFlag,
        Intent::ToggleRead,
        Intent::OpenMovePicker,
        Intent::Undo,
        Intent::Search(SearchIntent::Open),
        Intent::Help,
        Intent::Open,
        Intent::NextPane,
    ] {
        assert!(press(&mut state, intent).is_empty(), "{intent:?}");
    }
    assert_eq!(state.popup, Popup::Closed);
    assert!(!state.search.is_editing());
    assert!(state.compose.is_creating());
}

/// Les touches tapées pendant la création ne font rien (ni `d`, `e`, `D`, `/`) ; Échap annule, Ctrl+c quitte.
#[test]
fn typed_keys_map_to_nothing_while_creating() {
    let mut state = state_with_mailboxes();
    compose(&mut state, ComposeIntent::Reply);
    assert_eq!(context_of(&state), KeyContext::Creating);
    for c in ['d', 'e', 'D', '/', 'q', 'j', 'r', '?'] {
        assert_eq!(map(&state, key(c)), None, "{c}");
    }
    let escape = Some(Intent::Compose(ComposeIntent::Close));
    assert_eq!(map(&state, code(KeyCode::Esc)), escape);
    assert_eq!(map(&state, ctrl('c')), Some(Intent::Quit));
}

/// Échap annule la création ; la réponse tardive est ignorée et son brouillon orphelin supprimé.
#[test]
fn escape_cancels_creation_and_late_reply_deletes_the_orphan() {
    let mut state = state_with_mailboxes();
    compose(&mut state, ComposeIntent::Reply);
    assert!(compose(&mut state, ComposeIntent::Close).is_empty());
    assert!(state.compose.is_closed());
    let late = update(
        &mut state,
        created_reply("tui-1-draft-1", draft("d7", &[], "x", "")),
    );
    assert_eq!(late, vec![delete_command("d7")]);
    assert!(state.compose.is_closed());
    let again = update(
        &mut state,
        created_reply("tui-1-draft-1", draft("d7", &[], "x", "")),
    );
    assert!(again.is_empty());
}

/// Après une annulation une nouvelle demande part, et la réponse de l'ancienne ne l'ouvre pas.
#[test]
fn new_request_after_cancel_is_not_taken_over_by_the_old_reply() {
    let mut state = state_with_mailboxes();
    compose(&mut state, ComposeIntent::Reply);
    compose(&mut state, ComposeIntent::Close);
    let commands = compose(&mut state, ComposeIntent::New);
    let expected = create_command("a1", DraftMode::New, "tui-1-draft-2");
    assert_eq!(commands, vec![expected]);
    let late = update(
        &mut state,
        created_reply("tui-1-draft-1", draft("d7", &[], "x", "")),
    );
    assert_eq!(late, vec![delete_command("d7")]);
    assert!(state.compose.is_creating());
    update(
        &mut state,
        created_reply("tui-1-draft-2", draft("d8", &[], "y", "")),
    );
    assert_eq!(state.compose.open().unwrap().draft().as_str(), "d8");
}

/// Une réponse jamais demandée est ignorée sans supprimer quoi que ce soit.
#[test]
fn reply_for_a_token_never_issued_is_ignored() {
    let mut state = state_with_mailboxes();
    let commands = update(
        &mut state,
        created_reply("tui-1-draft-99", draft("d9", &[], "x", "")),
    );
    assert!(commands.is_empty());
}
