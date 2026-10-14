//! Fenêtres modales : confirmation de suppression et sélecteur de déplacement.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod action_fixtures;
mod message_fixtures;
mod navigation_fixtures;
mod reader_fixtures;
mod sidebar_fixtures;

use action_fixtures::state_with_mailboxes;
use mira_tui::update::Intent;
use mira_tui::view::render;
use navigation_fixtures::press;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use sidebar_fixtures::rendered_text;

/// Rend l'état dans un terminal de cette taille et rend le texte affiché.
fn rendered_in(state: &mira_tui::state::AppState, width: u16, height: u16) -> String {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal.draw(|frame| render(frame, state)).unwrap();
    let buffer = terminal.backend().buffer().clone();
    let symbols: Vec<&str> = buffer.content().iter().map(|cell| cell.symbol()).collect();
    symbols
        .chunks(usize::from(width))
        .map(|row| row.concat())
        .collect::<Vec<_>>()
        .join("\n")
}

/// La confirmation de suppression définitive est visible et dit comment répondre.
#[test]
fn confirm_popup_is_displayed() {
    let mut state = state_with_mailboxes();
    press(&mut state, Intent::DeletePermanently);
    let text = rendered_text(&state);
    assert!(text.contains("Supprimer définitivement"), "{text}");
    assert!(text.contains("Entrée"), "{text}");
}

/// Le sélecteur liste les boîtes proposées avec son titre.
#[test]
fn move_popup_lists_targets() {
    let mut state = state_with_mailboxes();
    press(&mut state, Intent::OpenMovePicker);
    let text = rendered_text(&state);
    assert!(text.contains("Déplacer vers"), "{text}");
    assert!(
        text.contains("Archive") && text.contains("Projets"),
        "{text}"
    );
}

/// Un nom de boîte hostile est neutralisé dans le sélecteur.
#[test]
fn move_popup_names_are_safe() {
    let mut state = state_with_mailboxes();
    state.mailboxes.replace_items(vec![
        sidebar_fixtures::mailbox(
            "a1",
            "INBOX",
            mira_assembly::mira_vocabulary::MailboxRole::Inbox,
            0,
        ),
        sidebar_fixtures::mailbox(
            "a1",
            "Mal\u{1b}[31mveillant",
            mira_assembly::mira_vocabulary::MailboxRole::Other,
            0,
        ),
    ]);
    press(&mut state, Intent::OpenMovePicker);
    let text = rendered_text(&state);
    assert!(!text.contains('\u{1b}'), "{text:?}");
    assert!(text.contains("Malveillant"), "{text}");
}

/// Les fenêtres modales ne font pas paniquer sur de très petits terminaux.
#[test]
fn popups_survive_tiny_terminals() {
    for (width, height) in [(1, 1), (5, 3), (20, 4), (30, 8)] {
        let mut state = state_with_mailboxes();
        press(&mut state, Intent::OpenMovePicker);
        rendered_in(&state, width, height);
        press(&mut state, Intent::Cancel);
        press(&mut state, Intent::DeletePermanently);
        rendered_in(&state, width, height);
    }
}
