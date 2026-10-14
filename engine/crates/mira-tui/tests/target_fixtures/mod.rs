//! États et lectures de commandes pour vérifier quel message une action vise.
#![allow(dead_code)]

use crate::action_fixtures::state_with_mailboxes;
use crate::message_fixtures::{inbox_view, page};
use crate::navigation_fixtures::press;
use mira_assembly::api::{ActionRequest, Request, TargetSelection};
use mira_assembly::mira_vocabulary::MessageId;
use mira_tui::commands::Command;
use mira_tui::state::{AppState, Pane};
use mira_tui::update::Intent;

/// Lecteur ouvert sur m1, puis sélection de la liste déplacée sur m2 (Maj+Tab, j, Tab).
pub fn reader_on_m1_list_on_m2() -> AppState {
    let mut state = state_with_mailboxes();
    press(&mut state, Intent::Open);
    press(&mut state, Intent::PrevPane);
    press(&mut state, Intent::MoveDown);
    press(&mut state, Intent::NextPane);
    assert_eq!(state.focus, Pane::Reader);
    state
}

/// Messages visés par les actions de ces commandes.
pub fn action_targets(commands: &[Command]) -> Vec<MessageId> {
    let mut targets = Vec::new();
    for command in commands {
        if let Command::Execute {
            request:
                Request::Action(ActionRequest::ApplyAction {
                    targets: selection, ..
                }),
            ..
        } = command
            && let TargetSelection::Messages { messages } = selection
        {
            targets.extend(messages.iter().cloned());
        }
    }
    targets
}

/// Liste de 60 messages chargés en deux pages (50 puis 10), m59 sélectionné, focus sur la liste.
pub fn state_with_two_pages() -> AppState {
    let mut state = state_with_mailboxes();
    state
        .messages
        .replace_page(inbox_view(), page(0, 50, Some("c1")));
    state
        .messages
        .mark_requested(mira_assembly::mira_vocabulary::Cursor("c1".to_string()));
    state.messages.append_page(page(50, 10, None));
    state.messages.selection.selected = Some(59);
    state
}

/// Identifiant de message brut.
pub fn id(raw: &str) -> MessageId {
    MessageId::from_raw(raw)
}
