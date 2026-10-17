//! Annonces d'envoi : l'objet est cité, et un ancien succès n'efface jamais l'échec d'envoi de l'éditeur ouvert.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod action_fixtures;
mod compose_fixtures;
mod message_fixtures;
mod navigation_fixtures;
mod reader_fixtures;
mod send_fixtures;
mod sidebar_fixtures;

use compose_fixtures::open_reply;
use mira_assembly::mira_vocabulary::{AccountId, ActionId, Event, EventKind, EventSeq, Timestamp};
use mira_tui::update::{Msg, update};
use send_fixtures::{failed_send, send_confirmed, sent_reply};

/// Événement « envoi parti » de cette action du compte a1.
fn departed(action: &str) -> Msg {
    Msg::Event(Event {
        seq: EventSeq(5),
        at: Timestamp::from_millis(0),
        kind: EventKind::SendDeparted {
            account: AccountId::from_raw("a1"),
            action: ActionId::from_raw(action),
        },
    })
}

/// Le départ d'un envoi de la session cite son objet.
#[test]
fn departed_send_names_its_subject() {
    let mut state = open_reply();
    send_confirmed(&mut state);
    update(&mut state, sent_reply("d1", "tui-1-send-2", Ok("act-9")));
    update(&mut state, departed("act-9"));
    let shown = state.status.message.clone().unwrap();
    assert!(shown.contains("« Re: Bonjour » envoyé"), "{shown}");
}

/// Un envoi plus ancien qui part n'efface pas l'échec de l'envoi en cours d'édition.
#[test]
fn older_departure_never_hides_a_send_failure() {
    let mut state = open_reply();
    send_confirmed(&mut state);
    update(&mut state, failed_send("d1", "tui-1-send-2"));
    update(&mut state, departed("ancien"));
    let shown = state.status.message.clone().unwrap();
    assert!(shown.contains("Échec de l'envoi"), "{shown}");
}

/// L'échec d'envoi dit comment réessayer sans risque de doublon.
#[test]
fn send_failure_explains_the_safe_retry() {
    let mut state = open_reply();
    send_confirmed(&mut state);
    update(&mut state, failed_send("d1", "tui-1-send-2"));
    let shown = state.status.message.clone().unwrap();
    assert!(shown.contains("F5"), "{shown}");
}
