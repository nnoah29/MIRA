//! Actions sur le message sélectionné : archiver, supprimer, marquer.

use mira_assembly::mira_vocabulary::ActionKind;

use super::action_command::{apply_command, selected_message, send_removing};
use crate::commands::Command;
use crate::state::{AppState, Popup};

/// Archive le message sélectionné.
pub fn archive(state: &mut AppState) -> Vec<Command> {
    send_selected(state, ActionKind::Archive)
}

/// Met le message sélectionné à la corbeille.
pub fn trash(state: &mut AppState) -> Vec<Command> {
    send_selected(state, ActionKind::Trash)
}

/// Demande confirmation avant de supprimer définitivement le message sélectionné.
pub fn delete_permanently(state: &mut AppState) -> Vec<Command> {
    if let Some(message) = selected_message(state) {
        state.popup = Popup::ConfirmDelete {
            messages: vec![message.id],
        };
    }
    Vec::new()
}

/// Envoie cette action sur le message sélectionné et le retire de la liste.
fn send_selected(state: &mut AppState, kind: ActionKind) -> Vec<Command> {
    match selected_message(state) {
        Some(message) => send_removing(state, kind, vec![message.id]),
        None => Vec::new(),
    }
}

/// Inverse l'étoile du message sélectionné.
pub fn toggle_flag(state: &mut AppState) -> Vec<Command> {
    let Some(message) = selected_message(state) else {
        return Vec::new();
    };
    let flagged = !message.flags.flagged;
    state.messages.set_flagged(&message.id, flagged);
    let kind = ActionKind::MarkFlagged { flagged };
    vec![apply_command(state, kind, vec![message.id])]
}

/// Inverse l'état lu du message sélectionné.
pub fn toggle_read(state: &mut AppState) -> Vec<Command> {
    let Some(message) = selected_message(state) else {
        return Vec::new();
    };
    let read = !message.flags.seen;
    state.messages.set_seen(&message.id, read);
    let kind = ActionKind::MarkRead { read };
    vec![apply_command(state, kind, vec![message.id])]
}
