//! Rendu de la liste de messages : lignes, état vide et neutralisation du texte.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod message_fixtures;
mod sidebar_fixtures;

use message_fixtures::{inbox_view, page, summary};
use mira_assembly::mira_vocabulary::Page;
use mira_tui::state::{AppState, Pane};
use sidebar_fixtures::{account, rendered_text};

/// État avec un compte et la liste de messages donnée.
fn state_with(items: Vec<mira_assembly::mira_vocabulary::MessageSummary>) -> AppState {
    let mut state = AppState::new(100, 30, 1);
    state.accounts.replace_items(vec![account("a1")]);
    let page = Page {
        items,
        next_cursor: None,
    };
    state.messages.replace_page(inbox_view(), page);
    state
}

/// Une boîte vide affiche « aucun message ».
#[test]
fn empty_mailbox_shows_no_messages_text() {
    let text = rendered_text(&state_with(Vec::new()));
    assert!(text.contains("aucun message"));
}

/// Une ligne montre l'expéditeur, l'objet et la date courte.
#[test]
fn list_row_shows_sender_subject_and_short_date() {
    let text = rendered_text(&state_with(vec![summary("m1", "Bonjour", "Bob")]));
    assert!(text.contains("Bob"));
    assert!(text.contains("Bonjour"));
    assert!(text.contains("21/01 14:00"));
    assert!(!text.contains("aucun message"));
}

/// Les messages non lus portent une pastille, pas les messages lus.
#[test]
fn unread_message_has_dot_and_read_has_none() {
    let mut read = summary("m2", "Lu", "Ann");
    read.flags.seen = true;
    let text = rendered_text(&state_with(vec![summary("m1", "Neuf", "Bob"), read]));
    let unread_line = text.lines().find(|line| line.contains("Bob")).unwrap();
    let read_line = text.lines().find(|line| line.contains("Ann")).unwrap();
    assert!(unread_line.contains('●'));
    assert!(!read_line.contains('●'));
}

/// L'objet et l'expéditeur passent par le texte neutralisé.
#[test]
fn list_row_uses_safe_text_for_subject_and_sender() {
    let hostile = summary("m1", "Obj\u{1b}[2Jet\u{7}", "Bo\u{1b}]0;x\u{7}b\r");
    let text = rendered_text(&state_with(vec![hostile]));
    assert!(text.contains("Objet"));
    assert!(text.contains("Bob"));
    assert!(!text.contains('\u{1b}'));
    assert!(!text.contains("[2J"));
}

/// Beaucoup de messages ne font pas déborder le rendu et la sélection reste visible.
#[test]
fn long_list_renders_selected_row() {
    let mut state = state_with(Vec::new());
    state
        .messages
        .replace_page(inbox_view(), page(0, 200, None));
    state.focus = Pane::List;
    state.messages.selection.selected = Some(150);
    let text = rendered_text(&state);
    assert!(text.contains("objet 150"));
}
