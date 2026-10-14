//! Liste de messages paginée : demande de la page suivante au défilement.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod message_fixtures;
mod navigation_fixtures;
mod paging_fixtures;

use message_fixtures::{inbox_view, page};
use mira_tui::state::{AppState, Pane};
use mira_tui::update::Intent;
use navigation_fixtures::press;
use paging_fixtures::{next_page_command, paged_state};

/// Approcher la fin de la liste demande la page suivante avec son curseur.
#[test]
fn scrolling_near_end_requests_next_page_with_cursor() {
    let mut state = paged_state();
    state.messages.selection.selected = Some(13);
    assert!(press(&mut state, Intent::MoveDown).is_empty());
    let commands = press(&mut state, Intent::MoveDown);
    assert_eq!(commands, vec![next_page_command("c1")]);
}

/// Un même curseur n'est demandé qu'une fois tant que la page n'est pas arrivée.
#[test]
fn next_page_is_requested_once_per_cursor() {
    let mut state = paged_state();
    assert_eq!(press(&mut state, Intent::Bottom).len(), 1);
    assert!(press(&mut state, Intent::MoveUp).is_empty());
    assert!(press(&mut state, Intent::MoveDown).is_empty());
    assert!(press(&mut state, Intent::Bottom).is_empty());
}

/// Sans curseur suivant, rien n'est demandé même en fin de liste.
#[test]
fn no_cursor_means_no_request() {
    let mut state = AppState::new(100, 30, 1);
    state.messages.replace_page(inbox_view(), page(0, 20, None));
    state.focus = Pane::List;
    assert!(press(&mut state, Intent::Bottom).is_empty());
}

/// Hors du panneau liste, le défilement ne demande rien.
#[test]
fn other_panes_do_not_request_pages() {
    let mut state = paged_state();
    state.focus = Pane::Reader;
    assert!(press(&mut state, Intent::Bottom).is_empty());
}
