//! Ouverture du sélecteur de déplacement et déplacement vers la boîte choisie.

use mira_assembly::mira_vocabulary::{ActionKind, Mailbox, MailboxId, MessageSummary, ViewRef};

use super::action_command::{selected_message, send_removing};
use crate::commands::Command;
use crate::state::{AppState, MovePicker, Popup};

/// Boîte dont la liste affiche les messages.
fn current_mailbox(state: &AppState) -> Option<&MailboxId> {
    match state.messages.view()? {
        ViewRef::Mailbox { mailbox } => Some(mailbox),
        _ => None,
    }
}

/// Autres boîtes du compte du message.
fn other_mailboxes(state: &AppState, message: &MessageSummary) -> Vec<Mailbox> {
    let current = current_mailbox(state);
    let same_account = |mailbox: &&Mailbox| mailbox.account == message.account;
    let elsewhere = |mailbox: &&Mailbox| Some(&mailbox.id) != current;
    let items = state.mailboxes.items().iter();
    items
        .filter(same_account)
        .filter(elsewhere)
        .cloned()
        .collect()
}

/// Ouvre le sélecteur sur les autres boîtes du compte, ou le dit s'il n'y en a pas.
pub fn open_move_picker(state: &mut AppState) -> Vec<Command> {
    let Some(message) = selected_message(state) else {
        return Vec::new();
    };
    let targets = other_mailboxes(state, &message);
    if targets.is_empty() {
        state.status.message = Some("Aucune autre boîte vers laquelle déplacer".to_string());
    } else {
        state.popup = Popup::MovePicker(MovePicker::new(vec![message.id], targets));
    }
    Vec::new()
}

/// Déplace les messages vers la boîte choisie du sélecteur.
pub fn confirm_move(state: &mut AppState, picker: &MovePicker) -> Vec<Command> {
    match picker.chosen() {
        Some(target) => {
            let kind = ActionKind::Move {
                to: target.id.clone(),
            };
            send_removing(state, kind, picker.messages.clone())
        }
        None => Vec::new(),
    }
}
