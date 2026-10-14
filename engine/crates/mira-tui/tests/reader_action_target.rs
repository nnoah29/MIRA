//! Une action lancée depuis le lecteur vise le message ouvert, jamais une autre ligne de la liste.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod action_fixtures;
mod engine_fixtures;
mod message_fixtures;
mod navigation_fixtures;
mod reader_fixtures;
mod sidebar_fixtures;
mod target_fixtures;

use engine_fixtures::sync_finished;
use message_fixtures::{inbox_view, page};
use mira_assembly::api::{MessageRequest, Request, Response};
use mira_tui::commands::{Command, Tag};
use mira_tui::state::Popup;
use mira_tui::update::{Intent, Msg, Reply, update};
use navigation_fixtures::press;
use target_fixtures::{action_targets, id, reader_on_m1_list_on_m2, state_with_two_pages};

/// Archiver, corbeille, étoile et lu depuis le lecteur visent m1 (ouvert), pas m2 (sélection de la liste).
#[test]
fn reader_actions_target_open_message_not_list_selection() {
    for intent in [
        Intent::Archive,
        Intent::Trash,
        Intent::ToggleFlag,
        Intent::ToggleRead,
    ] {
        let mut state = reader_on_m1_list_on_m2();
        let commands = press(&mut state, intent);
        assert_eq!(action_targets(&commands), vec![id("m1")], "{intent:?}");
    }
}

/// Suppression définitive et déplacement depuis le lecteur visent aussi le message ouvert.
#[test]
fn reader_delete_and_move_target_open_message() {
    let mut state = reader_on_m1_list_on_m2();
    press(&mut state, Intent::DeletePermanently);
    let commands = press(&mut state, Intent::Confirm);
    assert_eq!(action_targets(&commands), vec![id("m1")]);
    let mut state = reader_on_m1_list_on_m2();
    press(&mut state, Intent::OpenMovePicker);
    assert!(matches!(state.popup, Popup::MovePicker(_)));
    let commands = press(&mut state, Intent::Confirm);
    assert_eq!(action_targets(&commands), vec![id("m1")]);
}

/// Depuis la liste, l'action vise toujours la ligne sélectionnée.
#[test]
fn list_actions_still_target_list_selection() {
    let mut state = reader_on_m1_list_on_m2();
    press(&mut state, Intent::PrevPane);
    let commands = press(&mut state, Intent::Archive);
    assert_eq!(action_targets(&commands), vec![id("m2")]);
}

/// Un rafraîchissement qui ne ramène que la 1re page ne fait pas supprimer un autre message que celui ouvert.
#[test]
fn reader_delete_targets_open_message_after_refresh_truncates_list() {
    let mut state = state_with_two_pages();
    press(&mut state, Intent::Open);
    update(&mut state, sync_finished("a1"));
    let reply = Reply {
        tag: Tag::Messages {
            view: inbox_view(),
            after: None,
        },
        result: Ok(Response::Messages(page(0, 50, Some("c1")))),
    };
    update(&mut state, Msg::Reply(reply));
    press(&mut state, Intent::DeletePermanently);
    let commands = press(&mut state, Intent::Confirm);
    assert_eq!(action_targets(&commands), vec![id("m59")]);
}

/// Le rafraîchissement redemande au moins autant de messages qu'il y en a de chargés.
#[test]
fn refresh_asks_for_all_loaded_messages() {
    let mut state = state_with_two_pages();
    let commands = update(&mut state, sync_finished("a1"));
    let limits: Vec<u32> = commands
        .iter()
        .filter_map(|command| {
            if let Command::Execute {
                request: Request::Message(MessageRequest::ListMessages(query)),
                ..
            } = command
            {
                return Some(query.page.limit);
            }
            None
        })
        .collect();
    assert_eq!(limits, vec![60]);
}
