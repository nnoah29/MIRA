//! Aide contextuelle, pied de page des touches et titre des résultats de recherche tronqués.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod action_fixtures;
mod input_fixtures;
mod message_fixtures;
mod navigation_fixtures;
mod reader_fixtures;
mod screen_fixtures;
mod search_fixtures;
mod sidebar_fixtures;

use action_fixtures::state_with_mailboxes;
use input_fixtures::{key, map};
use mira_assembly::api::Response;
use mira_assembly::mira_vocabulary::{Cursor, Page};
use mira_tui::commands::Tag;
use mira_tui::state::{Pane, Popup};
use mira_tui::update::{Intent, Msg, Reply, SearchIntent, update};
use navigation_fixtures::press;
use screen_fixtures::text_in;
use search_fixtures::type_text;

/// `?` ouvre l'aide, qui liste les touches du panneau courant et pas celles des autres.
#[test]
fn question_mark_opens_help_with_contextual_keys() {
    let mut list = state_with_mailboxes();
    let intent = map(&list, key('?')).unwrap();
    press(&mut list, intent);
    assert_eq!(list.popup, Popup::Help);
    let text = text_in(&list, 120, 40);
    assert!(text.contains("Aide"), "{text}");
    assert!(
        text.contains("archiver") && text.contains("défaire"),
        "{text}"
    );
    assert!(!text.contains("compte suivant"), "{text}");
    let mut sidebar = state_with_mailboxes();
    sidebar.focus = Pane::Sidebar;
    press(&mut sidebar, Intent::Help);
    let text = text_in(&sidebar, 120, 40);
    assert!(text.contains("compte suivant"), "{text}");
    assert!(!text.contains("archiver"), "{text}");
    assert!(
        text.contains("quitter") && text.contains("recherche"),
        "{text}"
    );
}

/// Le pied de page montre les touches valables dans le contexte courant.
#[test]
fn footer_shows_keys_of_current_context() {
    let mut state = state_with_mailboxes();
    let list = text_in(&state, 200, 20);
    let footer = list.lines().last().unwrap().to_string();
    assert!(footer.contains("e archiver"), "{footer}");
    assert!(footer.contains("? aide"), "{footer}");
    state.focus = Pane::Sidebar;
    let sidebar = text_in(&state, 200, 20);
    let footer = sidebar.lines().last().unwrap().to_string();
    assert!(!footer.contains("archiver"), "{footer}");
    assert!(footer.contains("Entrée ouvrir"), "{footer}");
}

/// Des résultats avec une page suivante disent qu'ils ne sont que les premiers.
#[test]
fn truncated_search_results_say_first_results_only() {
    let mut state = state_with_mailboxes();
    press(&mut state, Intent::Search(SearchIntent::Open));
    type_text(&mut state, "facture");
    press(&mut state, Intent::Search(SearchIntent::Submit));
    let page = Page {
        items: message_fixtures::numbered(50),
        next_cursor: Some(Cursor("suite".to_string())),
    };
    let reply = Reply {
        tag: Tag::Search("facture".to_string()),
        result: Ok(Response::Messages(page)),
    };
    update(&mut state, Msg::Reply(reply));
    let text = text_in(&state, 160, 30);
    assert!(text.contains("50 premiers résultats"), "{text}");
}

/// Des résultats complets n'affichent pas la mention de troncature.
#[test]
fn complete_search_results_do_not_say_truncated() {
    let mut state = state_with_mailboxes();
    press(&mut state, Intent::Search(SearchIntent::Open));
    type_text(&mut state, "facture");
    press(&mut state, Intent::Search(SearchIntent::Submit));
    update(&mut state, search_fixtures::search_reply("facture", 3));
    let text = text_in(&state, 160, 30);
    assert!(text.contains("Recherche : facture"), "{text}");
    assert!(!text.contains("premiers résultats"), "{text}");
}
