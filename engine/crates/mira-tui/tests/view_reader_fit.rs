//! Lecteur : un en-tête volumineux ou un terminal minuscule ne prive jamais le corps de place.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod message_fixtures;
mod navigation_fixtures;
mod reader_fixtures;
mod sidebar_fixtures;

use mira_assembly::mira_vocabulary::ThreadId;
use mira_tui::state::AppState;
use mira_tui::update::{Intent, update};
use navigation_fixtures::press;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use reader_fixtures::{
    attachments_reply, body, body_reply, list_state, thread_reply, unread_message,
};

/// Texte rendu par un terminal de cette taille.
fn render_at(state: &AppState, width: u16, height: u16) -> String {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal
        .draw(|frame| mira_tui::view::render(frame, state))
        .unwrap();
    let buffer = terminal.backend().buffer().clone();
    buffer
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect::<Vec<_>>()
        .concat()
}

/// Message ouvert d'un fil, avec ce corps, ces pièces jointes et ce fil.
fn opened(message_subject: &str, files: usize, thread: usize) -> AppState {
    let mut message = unread_message();
    message.subject = message_subject.to_string();
    message.thread = Some(ThreadId::from_raw("t1"));
    message.has_attachments = true;
    let mut state = list_state(vec![message]);
    press(&mut state, Intent::Open);
    update(&mut state, body_reply("m1", Ok(body("DEBUT-CORPS\nsuite"))));
    update(&mut state, attachments_reply(files));
    update(&mut state, thread_reply(thread));
    state
}

/// Un grand fil et beaucoup de pièces jointes laissent le début du corps visible.
#[test]
fn big_thread_and_attachments_keep_body_visible() {
    let state = opened("Bonjour", 10, 30);
    let text = render_at(&state, 100, 20);
    assert!(text.contains("DEBUT-CORPS"));
    assert!(text.contains("… +"));
}

/// Un très long objet sans espace ne fait pas paniquer et laisse le corps visible.
#[test]
fn very_long_subject_keeps_body_visible() {
    let state = opened(&"x".repeat(500), 0, 0);
    let text = render_at(&state, 100, 20);
    assert!(text.contains("DEBUT-CORPS"));
}

/// Des terminaux minuscules ou vides ne font pas paniquer.
#[test]
fn tiny_terminals_do_not_panic() {
    let state = opened("Bonjour", 10, 30);
    for (width, height) in [(10, 3), (0, 0), (1, 1), (100, 4), (45, 5)] {
        render_at(&state, width, height);
    }
}
