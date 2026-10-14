//! La corbeille est réversible : aucune confirmation.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod action_fixtures;
mod message_fixtures;
mod navigation_fixtures;
mod reader_fixtures;
mod sidebar_fixtures;

use action_fixtures::{action_command, state_with_mailboxes};
use mira_assembly::mira_vocabulary::ActionKind;
use mira_tui::state::Popup;
use mira_tui::update::Intent;
use navigation_fixtures::press;

/// La corbeille est réversible : l'action part sans fenêtre de confirmation.
#[test]
fn trash_sends_trash_action_without_confirmation() {
    let mut state = state_with_mailboxes();
    let commands = press(&mut state, Intent::Trash);
    let wanted = action_command(ActionKind::Trash, &["m1"], "tui-1-act-1");
    assert_eq!(commands, vec![wanted]);
    assert_eq!(state.popup, Popup::Closed);
}
