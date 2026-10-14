//! Liste de messages paginée : pages reçues, pages périmées, erreurs et bornes de la sélection.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod message_fixtures;
mod navigation_fixtures;
mod paging_fixtures;

use message_fixtures::{numbered, page};
use mira_assembly::api::Response;
use mira_assembly::mira_vocabulary::{Cursor, ErrorCategory, MiraError};
use mira_tui::commands::Tag;
use mira_tui::update::{Intent, Msg, Reply, update};
use navigation_fixtures::press;
use paging_fixtures::{messages_reply, next_page_command, paged_state};

/// La page reçue s'ajoute à la suite, garde la sélection et permet le curseur suivant.
#[test]
fn appended_page_extends_list_and_allows_next_cursor() {
    let mut state = paged_state();
    press(&mut state, Intent::Bottom);
    let reply = messages_reply(Some("c1"), Ok(page(20, 20, Some("c2"))));
    assert!(update(&mut state, reply).is_empty());
    assert_eq!(state.messages.len(), 40);
    assert_eq!(state.messages.selection.selected, Some(19));
    let commands = press(&mut state, Intent::Bottom);
    assert_eq!(commands, vec![next_page_command("c2")]);
}

/// Une page ajoutée pour une autre vue est ignorée.
#[test]
fn stale_appended_page_is_ignored() {
    let mut state = paged_state();
    press(&mut state, Intent::Bottom);
    let other = Tag::Messages {
        view: mira_assembly::mira_vocabulary::ViewRef::Standard {
            view: mira_assembly::mira_vocabulary::StandardView::Unread,
        },
        after: Some(Cursor("c1".to_string())),
    };
    let reply = Reply {
        tag: other,
        result: Ok(Response::Messages(page(20, 5, None))),
    };
    update(&mut state, Msg::Reply(reply));
    assert_eq!(state.messages.len(), 20);
}

/// Une erreur sur la page suivante est signalée et autorise une nouvelle tentative.
#[test]
fn failed_next_page_reports_and_allows_retry() {
    let mut state = paged_state();
    press(&mut state, Intent::Bottom);
    let error = MiraError::new(
        ErrorCategory::Network,
        mira_assembly::mira_vocabulary::error::codes::network::UNREACHABLE,
        "serveur injoignable",
    );
    update(&mut state, messages_reply(Some("c1"), Err(error)));
    assert!(
        state
            .status
            .message
            .clone()
            .unwrap()
            .contains("serveur injoignable")
    );
    assert_eq!(state.messages.len(), 20);
    let commands = press(&mut state, Intent::MoveUp);
    assert_eq!(commands, vec![next_page_command("c1")]);
}

/// Remplacer la page par une plus courte ramène la sélection dans les bornes.
#[test]
fn selection_stays_in_bounds_after_page_replaced() {
    let mut state = paged_state();
    press(&mut state, Intent::Bottom);
    update(&mut state, messages_reply(None, Ok(page(0, 3, None))));
    assert_eq!(state.messages.selection.selected, Some(2));
    update(&mut state, messages_reply(None, Ok(page(0, 0, None))));
    assert_eq!(state.messages.selection.selected, None);
    state.messages.replace_items(numbered(4));
    assert_eq!(state.messages.selection.selected, Some(0));
}

/// Une page 2 tardive, arrivée après le remplacement de la liste, est ignorée.
#[test]
fn late_append_after_replace_is_ignored() {
    let mut state = paged_state();
    press(&mut state, Intent::Bottom);
    update(
        &mut state,
        messages_reply(None, Ok(page(0, 20, Some("d1")))),
    );
    update(
        &mut state,
        messages_reply(Some("c1"), Ok(page(20, 20, Some("c2")))),
    );
    assert_eq!(state.messages.len(), 20);
    assert_eq!(state.messages.unrequested_cursor().unwrap().0, "d1");
}
