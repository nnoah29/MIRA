//! Ouverture d'un message : corps demandé, hors ligne non fatal, texte neutralisé, marquage lu.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod message_fixtures;
mod navigation_fixtures;
mod reader_fixtures;
mod sidebar_fixtures;

use mira_tui::state::Pane;
use mira_tui::state::reader::ReaderContent;
use mira_tui::update::{Intent, update};
use navigation_fixtures::press;
use reader_fixtures::{
    body, body_reply, get_body_command, list_state, mark_read_command, offline_error,
    unread_message,
};

/// Ouvrir un message de la liste demande son corps local et donne le focus au lecteur.
#[test]
fn opening_message_requests_body() {
    let mut state = list_state(vec![unread_message()]);
    let commands = press(&mut state, Intent::Open);
    assert!(commands.contains(&get_body_command("m1")));
    assert_eq!(state.focus, Pane::Reader);
}

/// Un corps absent hors ligne donne une invite claire, sans erreur fatale ni perte de sélection.
#[test]
fn offline_body_error_shows_download_hint_not_fatal() {
    let mut state = list_state(vec![unread_message()]);
    press(&mut state, Intent::Open);
    let commands = update(&mut state, body_reply("m1", Err(offline_error())));
    assert!(commands.is_empty());
    assert_eq!(state.reader.content, ReaderContent::OfflineMissing);
    assert_eq!(state.messages.selection.selected, Some(0));
    assert!(state.status.message.is_none());
    assert!(!state.should_quit);
}

/// Le corps reçu est rangé tel quel dans le lecteur (la neutralisation se fait à l'affichage).
#[test]
fn body_reply_is_stored_for_the_open_message() {
    let mut state = list_state(vec![unread_message()]);
    press(&mut state, Intent::Open);
    update(&mut state, body_reply("m1", Ok(body("salut"))));
    assert_eq!(state.reader.content, ReaderContent::Body(body("salut")));
}

/// Ouvrir un message non lu émet l'action « marquer lu » avec une clé déterministe.
#[test]
fn open_marks_message_read_via_action() {
    let mut state = list_state(vec![unread_message()]);
    let commands = press(&mut state, Intent::Open);
    let wanted = mark_read_command("m1", "tui-1-read-m1-1");
    assert!(commands.contains(&wanted), "{commands:?}");
    assert!(state.messages.items()[0].flags.seen);
}

/// Un message déjà lu n'émet pas d'action de marquage.
#[test]
fn open_read_message_emits_no_action() {
    let mut read = unread_message();
    read.flags.seen = true;
    let mut state = list_state(vec![read]);
    let commands = press(&mut state, Intent::Open);
    assert_eq!(commands, vec![get_body_command("m1")]);
}

/// Les clés d'idempotence successives sont distinctes.
#[test]
fn successive_mark_read_keys_differ() {
    let mut state = list_state(vec![
        unread_message(),
        message_fixtures::summary("m2", "Autre", "Ann"),
    ]);
    press(&mut state, Intent::Open);
    press(&mut state, Intent::Back);
    press(&mut state, Intent::MoveDown);
    let commands = press(&mut state, Intent::Open);
    assert!(commands.contains(&mark_read_command("m2", "tui-1-read-m2-2")));
}
