//! Barre d'état : synchro, file d'actions, hors ligne et dernière erreur.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod action_fixtures;
mod engine_fixtures;
mod message_fixtures;
mod navigation_fixtures;
mod reader_fixtures;
mod search_fixtures;
mod sidebar_fixtures;

use action_fixtures::{action_reply, state_with_mailboxes};
use engine_fixtures::{status_reply, sync_status_reply};
use mira_assembly::mira_vocabulary::AccountState;
use mira_tui::update::{Intent, SearchIntent, update};
use navigation_fixtures::press;
use reader_fixtures::offline_error;
use search_fixtures::type_text;
use sidebar_fixtures::rendered_text;

/// Le nombre d'actions en file, somme des comptes, s'affiche.
#[test]
fn status_bar_shows_pending_action_count() {
    let mut state = state_with_mailboxes();
    update(&mut state, status_reply(&[2, 1]));
    assert!(rendered_text(&state).contains("3 actions en file"));
    update(&mut state, status_reply(&[1]));
    assert!(rendered_text(&state).contains("1 action en file"));
    update(&mut state, status_reply(&[0]));
    assert!(!rendered_text(&state).contains("en file"));
}

/// La synchro en cours et l'état hors ligne s'affichent.
#[test]
fn status_bar_shows_sync_and_offline() {
    let mut state = state_with_mailboxes();
    update(
        &mut state,
        sync_status_reply(true, AccountState::Offline, None),
    );
    let text = rendered_text(&state);
    assert!(text.contains("Synchro en cours"));
    assert!(text.contains("Hors ligne"));
    update(
        &mut state,
        sync_status_reply(false, AccountState::Connected, None),
    );
    assert!(!rendered_text(&state).contains("Synchro en cours"));
}

/// L'erreur reste affichée jusqu'à la prochaine action réussie.
#[test]
fn status_bar_shows_last_error_until_next_action() {
    let mut state = state_with_mailboxes();
    press(&mut state, Intent::Archive);
    update(
        &mut state,
        action_reply("tui-1-act-1", Err(offline_error())),
    );
    assert!(rendered_text(&state).contains("corps absent"));
    press(&mut state, Intent::ToggleFlag);
    assert!(rendered_text(&state).contains("corps absent"));
    update(&mut state, action_reply("tui-1-act-2", Ok("act-9")));
    assert!(!rendered_text(&state).contains("corps absent"));
}

/// Pendant la saisie, la barre montre le texte tapé, neutralisé.
#[test]
fn search_input_is_shown_safely_in_bar() {
    let mut state = state_with_mailboxes();
    press(&mut state, Intent::Search(SearchIntent::Open));
    type_text(&mut state, "ab\u{1b}[31mc");
    let text = rendered_text(&state);
    assert!(text.contains("ab[31mc"));
    assert!(!text.contains('\u{1b}'));
}

/// Le titre de la liste pendant une recherche reprend la requête, neutralisée.
#[test]
fn list_title_shows_safe_search_query() {
    let mut state = state_with_mailboxes();
    press(&mut state, Intent::Search(SearchIntent::Open));
    type_text(&mut state, "a\u{7}b");
    press(&mut state, Intent::Search(SearchIntent::Submit));
    assert!(rendered_text(&state).contains("Recherche : ab"));
}
