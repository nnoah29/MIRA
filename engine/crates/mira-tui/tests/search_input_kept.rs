//! Une réponse qui arrive pendant la saisie de recherche ne la ferme pas ; un échec pendant une recherche la relance.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod action_fixtures;
mod loading_fixtures;
mod message_fixtures;
mod navigation_fixtures;
mod reader_fixtures;
mod search_fixtures;
mod sidebar_fixtures;

use action_fixtures::{action_reply, state_with_mailboxes};
use loading_fixtures::mailboxes_arrive;
use mira_tui::state::AppState;
use mira_tui::update::{Intent, SearchIntent, update};
use navigation_fixtures::press;
use reader_fixtures::offline_error;
use search_fixtures::{search_command, search_reply, type_text};
use sidebar_fixtures::refresh;

/// Les boîtes arrivent pendant qu'on tape une requête : la saisie reste ouverte avec son texte.
#[test]
fn mailboxes_reply_keeps_search_input_open() {
    let mut state = AppState::new(100, 30, 1);
    refresh(&mut state, &["a1"]);
    press(&mut state, Intent::Search(SearchIntent::Open));
    type_text(&mut state, "quit");
    mailboxes_arrive(&mut state);
    assert!(state.search.is_editing());
    assert_eq!(state.search.input(), "quit");
}

/// L'échec d'une action arrive pendant la saisie : la saisie reste ouverte avec son texte.
#[test]
fn action_failure_keeps_search_input_open() {
    let mut state = state_with_mailboxes();
    press(&mut state, Intent::Archive);
    press(&mut state, Intent::Search(SearchIntent::Open));
    type_text(&mut state, "ab");
    update(
        &mut state,
        action_reply("tui-1-act-1", Err(offline_error())),
    );
    assert!(state.search.is_editing());
    assert_eq!(state.search.input(), "ab");
}

/// Une action échouée pendant l'affichage de résultats relance la recherche pour défaire le retrait optimiste.
#[test]
fn action_failure_during_search_reruns_search() {
    let mut state = state_with_mailboxes();
    press(&mut state, Intent::Search(SearchIntent::Open));
    type_text(&mut state, "facture");
    press(&mut state, Intent::Search(SearchIntent::Submit));
    update(&mut state, search_reply("facture", 3));
    press(&mut state, Intent::Archive);
    let commands = update(
        &mut state,
        action_reply("tui-1-act-1", Err(offline_error())),
    );
    assert!(
        commands.contains(&search_command("facture")),
        "{commands:?}"
    );
    assert_eq!(state.search.query(), Some("facture"));
}
