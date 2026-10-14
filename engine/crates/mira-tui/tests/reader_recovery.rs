//! Le lecteur survit à l'échec d'une action et suit les bascules lu/étoile du message ouvert.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod action_fixtures;
mod message_fixtures;
mod navigation_fixtures;
mod reader_fixtures;
mod sidebar_fixtures;

use action_fixtures::state_with_mailboxes;
use mira_assembly::api::{ActionRequest, Request};
use mira_assembly::mira_vocabulary::{ActionKind, IdempotencyKey, MessageId};
use mira_tui::commands::{Command, Tag};
use mira_tui::state::AppState;
use mira_tui::update::{Intent, Msg, Reply, update};
use navigation_fixtures::press;
use reader_fixtures::offline_error;

/// Clé et nature de l'action de l'utilisateur parmi ces commandes.
fn user_action(commands: &[Command]) -> (IdempotencyKey, ActionKind) {
    for command in commands {
        if let Command::Execute {
            tag: Tag::UserAction(key),
            request: Request::Action(ActionRequest::ApplyAction { kind, .. }),
        } = command
        {
            return (key.clone(), kind.clone());
        }
    }
    panic!("aucune action : {commands:?}");
}

/// Lecteur ouvert sur m1, focus sur le lecteur.
fn reading_m1() -> AppState {
    let mut state = state_with_mailboxes();
    press(&mut state, Intent::Open);
    state
}

/// Une étoile refusée par le moteur recharge la vue sans fermer le message ouvert.
#[test]
fn failed_action_keeps_open_message_in_reader() {
    let mut state = reading_m1();
    let (key, _) = user_action(&press(&mut state, Intent::ToggleFlag));
    let reply = Reply {
        tag: Tag::UserAction(key),
        result: Err(offline_error()),
    };
    update(&mut state, Msg::Reply(reply));
    assert!(state.reader.shows(&MessageId::from_raw("m1")));
}

/// Message ouvert absent de la liste : deux appuis sur `u` puis `s` donnent des valeurs alternées.
#[test]
fn toggles_alternate_for_open_message_missing_from_list() {
    let mut state = reading_m1();
    state.messages.remove(&MessageId::from_raw("m1"));
    let (_, first) = user_action(&press(&mut state, Intent::ToggleRead));
    let (_, second) = user_action(&press(&mut state, Intent::ToggleRead));
    assert_ne!(first, second, "{first:?}");
    let (_, first) = user_action(&press(&mut state, Intent::ToggleFlag));
    let (_, second) = user_action(&press(&mut state, Intent::ToggleFlag));
    assert_ne!(first, second, "{first:?}");
}
