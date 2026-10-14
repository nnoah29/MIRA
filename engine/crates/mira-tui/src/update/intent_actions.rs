//! Intentions d'action sur les messages, hors fenêtre modale.

use super::intent::Intent;
use super::{actions, move_picker, undo};
use crate::commands::Command;
use crate::state::AppState;

/// Route une intention d'action ; confirmer ou annuler sans fenêtre ouverte ne fait rien.
pub fn on_message_intent(state: &mut AppState, intent: Intent) -> Vec<Command> {
    match intent {
        Intent::Archive => actions::archive(state),
        Intent::Trash => actions::trash(state),
        Intent::DeletePermanently => actions::delete_permanently(state),
        Intent::ToggleFlag => actions::toggle_flag(state),
        Intent::ToggleRead => actions::toggle_read(state),
        Intent::OpenMovePicker => move_picker::open_move_picker(state),
        Intent::Undo => undo::undo(state),
        Intent::MoveDown
        | Intent::MoveUp
        | Intent::Top
        | Intent::Bottom
        | Intent::Open
        | Intent::Back
        | Intent::NextPane
        | Intent::PrevPane
        | Intent::NextAccount
        | Intent::PrevAccount
        | Intent::Quit
        | Intent::Confirm
        | Intent::Cancel
        | Intent::Search(_)
        | Intent::StartSync => Vec::new(),
    }
}
