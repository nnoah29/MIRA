//! Fabriques d'état partagées par les tests de navigation.
#![allow(dead_code)]

use crate::message_fixtures::numbered;
use mira_tui::commands::Command;
use mira_tui::state::{AppState, Pane};
use mira_tui::update::{Intent, Msg, update};

/// Applique une intention à l'état et rend les commandes produites.
pub fn press(state: &mut AppState, intent: Intent) -> Vec<Command> {
    update(state, Msg::Intent(intent))
}

/// État dont la liste de messages contient `count` éléments et qui a le focus sur elle.
pub fn state_with_messages(count: usize) -> AppState {
    let mut state = AppState::new(80, 24, 1);
    state.messages.replace_items(numbered(count));
    state.focus = Pane::List;
    state
}
