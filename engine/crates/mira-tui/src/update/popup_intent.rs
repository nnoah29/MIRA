//! Intentions pendant qu'une fenêtre modale est ouverte : elle seule les reçoit.

use mira_assembly::mira_vocabulary::ActionKind;

use super::action_command::send_removing;
use super::intent::Intent;
use super::move_picker::confirm_move;
use super::panes;
use crate::commands::Command;
use crate::state::{AppState, MovePicker, Popup};

/// Route une intention vers la fenêtre ouverte ; celles qui ne la concernent pas sont ignorées.
pub fn route(state: &mut AppState, intent: Intent) -> Vec<Command> {
    match intent {
        Intent::Confirm | Intent::Open => confirm(state),
        Intent::Cancel | Intent::Back => cancel(state),
        Intent::MoveDown => step_picker(state, MovePicker::down),
        Intent::MoveUp => step_picker(state, MovePicker::up),
        Intent::Quit => panes::quit(state),
        Intent::Top
        | Intent::Bottom
        | Intent::NextPane
        | Intent::PrevPane
        | Intent::NextAccount
        | Intent::PrevAccount
        | Intent::Archive
        | Intent::Trash
        | Intent::DeletePermanently
        | Intent::ToggleFlag
        | Intent::ToggleRead
        | Intent::OpenMovePicker
        | Intent::Undo
        | Intent::Search(_)
        | Intent::StartSync => Vec::new(),
    }
}

/// Ferme la fenêtre sans rien envoyer.
fn cancel(state: &mut AppState) -> Vec<Command> {
    state.popup = Popup::Closed;
    Vec::new()
}

/// Déplace le choix du sélecteur ; sans effet sur une confirmation.
fn step_picker(state: &mut AppState, step: fn(&mut MovePicker)) -> Vec<Command> {
    if let Popup::MovePicker(picker) = &mut state.popup {
        step(picker);
    }
    Vec::new()
}

/// Ferme la fenêtre et envoie l'action qu'elle confirmait.
fn confirm(state: &mut AppState) -> Vec<Command> {
    match std::mem::take(&mut state.popup) {
        Popup::ConfirmDelete { messages } => {
            send_removing(state, ActionKind::DeletePermanently, messages)
        }
        Popup::MovePicker(picker) => confirm_move(state, &picker),
        Popup::Closed => Vec::new(),
    }
}
