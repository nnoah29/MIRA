//! Recherche locale : saisie, envoi, sortie et réponses tardives.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod action_fixtures;
mod loading_fixtures;
mod message_fixtures;
mod navigation_fixtures;
mod reader_fixtures;
mod search_fixtures;
mod sidebar_fixtures;

use loading_fixtures::{load_command, loaded};
use mira_tui::update::{Intent, SearchIntent, update};
use navigation_fixtures::press;
use search_fixtures::{search_command, search_reply, type_text};

/// `/` ouvre la saisie, `Entrée` envoie la requête tapée et le résultat remplace la liste.
#[test]
fn slash_opens_search_and_enter_submits_query() {
    let mut state = loaded();
    press(&mut state, Intent::Search(SearchIntent::Open));
    assert!(state.search.is_editing());
    type_text(&mut state, "facture");
    let commands = press(&mut state, Intent::Search(SearchIntent::Submit));
    assert_eq!(commands, vec![search_command("facture")]);
    update(&mut state, search_reply("facture", 3));
    assert_eq!(state.messages.len(), 3);
    assert_eq!(state.search.query(), Some("facture"));
}

/// `Échap` quitte la recherche et recharge la boîte qui était affichée avant.
#[test]
fn escape_leaves_search_and_restores_mailbox_view() {
    let mut state = loaded();
    press(&mut state, Intent::Search(SearchIntent::Open));
    type_text(&mut state, "x");
    press(&mut state, Intent::Search(SearchIntent::Submit));
    update(&mut state, search_reply("x", 2));
    let commands = press(&mut state, Intent::Search(SearchIntent::Leave));
    assert_eq!(commands, vec![load_command("a1-INBOX")]);
    assert_eq!(state.search.query(), None);
    assert!(!state.search.is_editing());
}

/// Une requête vide ou blanche n'est jamais envoyée et la saisie reste ouverte.
#[test]
fn empty_query_is_not_sent() {
    let mut state = loaded();
    press(&mut state, Intent::Search(SearchIntent::Open));
    assert!(press(&mut state, Intent::Search(SearchIntent::Submit)).is_empty());
    type_text(&mut state, "   ");
    assert!(press(&mut state, Intent::Search(SearchIntent::Submit)).is_empty());
    assert!(state.search.is_editing());
}

/// La réponse d'une ancienne requête, arrivée après une nouvelle, est ignorée.
#[test]
fn late_reply_of_previous_query_is_ignored() {
    let mut state = loaded();
    press(&mut state, Intent::Search(SearchIntent::Open));
    type_text(&mut state, "ancien");
    press(&mut state, Intent::Search(SearchIntent::Submit));
    press(&mut state, Intent::Search(SearchIntent::Open));
    type_text(&mut state, "neuf");
    press(&mut state, Intent::Search(SearchIntent::Submit));
    update(&mut state, search_reply("ancien", 5));
    assert_eq!(state.messages.len(), 0);
    update(&mut state, search_reply("neuf", 2));
    assert_eq!(state.messages.len(), 2);
}

/// Après la sortie de la recherche, la réponse tardive de la recherche ne revient pas dans la liste.
#[test]
fn search_reply_after_leaving_is_ignored() {
    let mut state = loaded();
    press(&mut state, Intent::Search(SearchIntent::Open));
    type_text(&mut state, "x");
    press(&mut state, Intent::Search(SearchIntent::Submit));
    press(&mut state, Intent::Search(SearchIntent::Leave));
    update(&mut state, search_reply("x", 4));
    assert_eq!(state.messages.len(), 0);
}
