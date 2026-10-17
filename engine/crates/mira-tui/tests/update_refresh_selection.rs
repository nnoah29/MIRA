//! Rafraîchissement discret d'une vue : la sélection suit le message, pas l'indice.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod action_fixtures;
mod engine_fixtures;
mod message_fixtures;
mod navigation_fixtures;
mod reader_fixtures;
mod search_fixtures;
mod sidebar_fixtures;

use action_fixtures::{action_command, state_with_mailboxes};
use engine_fixtures::sync_finished;
use message_fixtures::{inbox_view, summary};
use mira_assembly::api::Response;
use mira_assembly::mira_vocabulary::{ActionKind, MessageSummary, Page};
use mira_tui::commands::Tag;
use mira_tui::state::AppState;
use mira_tui::update::{Intent, Msg, Reply, update};
use navigation_fixtures::press;

/// Fait arriver la réponse d'un rafraîchissement de la boîte avec ces messages.
fn refresh_with(state: &mut AppState, items: Vec<MessageSummary>) {
    update(state, sync_finished("a1"));
    let tag = Tag::Messages {
        view: inbox_view(),
        after: None,
    };
    let page = Page {
        items,
        next_cursor: None,
    };
    let result = Ok(Response::Messages(page));
    update(state, Msg::Reply(Reply { tag, result }));
}

/// Du courrier inséré au-dessus du curseur ne déplace pas la sélection : l'archivage vise le même message.
#[test]
fn insertion_above_cursor_keeps_selected_message() {
    let mut state = state_with_mailboxes();
    press(&mut state, Intent::MoveDown);
    let items = vec![
        summary("n1", "Neuf", "Eve"),
        summary("m1", "Bonjour", "Bob"),
        summary("m2", "Salut", "Ann"),
    ];
    refresh_with(&mut state, items);
    assert_eq!(state.messages.selection.selected, Some(2));
    let commands = press(&mut state, Intent::Archive);
    assert_eq!(
        commands,
        vec![action_command(ActionKind::Archive, &["m2"], "tui-1-act-1")]
    );
}

/// Le message sélectionné disparaît : recadrage par indice, sélection valide.
#[test]
fn vanished_selection_falls_back_to_clamped_index() {
    let mut state = state_with_mailboxes();
    press(&mut state, Intent::MoveDown);
    refresh_with(&mut state, vec![summary("m1", "Bonjour", "Bob")]);
    assert_eq!(state.messages.selection.selected, Some(0));
}

/// Une liste rafraîchie vide n'a plus de sélection.
#[test]
fn emptied_refresh_clears_selection() {
    let mut state = state_with_mailboxes();
    refresh_with(&mut state, Vec::new());
    assert_eq!(state.messages.selection.selected, None);
}

/// Le message ouvert dans le lecteur reste ouvert tant qu'il est dans la liste.
#[test]
fn reader_stays_open_while_message_remains() {
    let mut state = state_with_mailboxes();
    press(&mut state, Intent::Open);
    let items = vec![
        summary("n1", "Neuf", "Eve"),
        summary("m1", "Bonjour", "Bob"),
    ];
    refresh_with(&mut state, items);
    assert_eq!(state.reader.message().map(|m| m.id.as_str()), Some("m1"));
}
