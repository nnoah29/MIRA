//! Réponse du moteur à l'envoi (succès, réponse tardive) et sortie pendant un envoi.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod action_fixtures;
mod compose_fixtures;
mod message_fixtures;
mod navigation_fixtures;
mod reader_fixtures;
mod send_fixtures;
mod sidebar_fixtures;

use compose_fixtures::{compose, open_reply, saved_reply};
use mira_tui::commands::Command;
use mira_tui::state::Popup;
use mira_tui::state::compose::SendPhase;
use mira_tui::update::{ComposeIntent, Intent, update};
use navigation_fixtures::press;
use send_fixtures::{count_sends, send_confirmed, sent_reply};

/// Succès : l'éditeur se ferme sans supprimer le brouillon, la barre d'état dit que le message est en file.
#[test]
fn accepted_send_closes_the_editor() {
    let mut state = open_reply();
    send_confirmed(&mut state);
    let after = update(&mut state, sent_reply("d1", "tui-1-send-2", Ok("act-9")));
    assert!(after.is_empty(), "{after:?}");
    assert!(state.compose.is_closed());
    let shown = state.status.message.clone().unwrap();
    assert!(shown.contains("En file d'envoi"), "{shown}");
}

/// Réponse d'un autre brouillon ou d'une autre clé : ignorée, l'envoi reste en vol.
#[test]
fn late_send_reply_for_another_draft_is_ignored() {
    let mut state = open_reply();
    send_confirmed(&mut state);
    update(&mut state, sent_reply("d2", "tui-1-send-2", Ok("act-9")));
    update(&mut state, sent_reply("d1", "tui-9-send-7", Ok("act-9")));
    let open = state.compose.open().unwrap();
    assert_eq!(open.send_phase(), SendPhase::Sending);
    let mut idle = open_reply();
    update(&mut idle, sent_reply("d1", "tui-1-send-2", Ok("act-9")));
    assert!(idle.compose.open().is_some());
}

/// Quitter pendant l'envoi en vol quitte tout de suite : la requête est déjà partie, aucun nouvel envoi.
#[test]
fn quit_while_send_in_flight_quits_without_new_send() {
    let mut state = open_reply();
    send_confirmed(&mut state);
    let commands = press(&mut state, Intent::Quit);
    assert_eq!(commands, vec![Command::Quit]);
    assert!(state.should_quit);
}

/// Quitter pendant l'enregistrement qui précède l'envoi annule l'envoi : rien ne partira ensuite.
#[test]
fn quit_while_saving_before_send_cancels_the_send() {
    let mut state = compose_fixtures::open_modified();
    send_confirmed(&mut state);
    press(&mut state, Intent::Quit);
    assert_eq!(state.popup, Popup::ConfirmCloseDraft);
    assert_eq!(state.compose.open().unwrap().send_phase(), SendPhase::Idle);
    let saved = compose_fixtures::draft("d1", &["bob@example.org"], "Re: Bonjour!", "Corps");
    let after = update(&mut state, saved_reply("d1", Ok(saved)));
    assert_eq!(count_sends(&after), 0);
}

/// Enregistrement réussi avant l'envoi : l'envoi part ensuite, dans cet ordre, avec la clé neuve du brouillon.
#[test]
fn successful_save_is_followed_by_the_send() {
    let mut state = compose_fixtures::open_modified();
    assert_eq!(count_sends(&send_confirmed(&mut state)), 0);
    assert!(compose(&mut state, ComposeIntent::Send).is_empty());
    let saved = compose_fixtures::draft("d1", &["bob@example.org"], "Re: Bonjour!", "Corps");
    let after = update(&mut state, saved_reply("d1", Ok(saved)));
    assert_eq!(
        after,
        vec![send_fixtures::send_command("d1", "tui-1-send-2")]
    );
    assert_eq!(
        state.compose.open().unwrap().send_phase(),
        SendPhase::Sending
    );
}
