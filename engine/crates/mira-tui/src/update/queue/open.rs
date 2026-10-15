//! Ouverture, fermeture et relecture de la vue « file » du compte sélectionné.

use mira_assembly::api::{ComposeRequest, QueueRequest, Request};
use mira_assembly::mira_vocabulary::{AccountId, ActionState};

use crate::commands::{Command, Tag};
use crate::state::AppState;
use crate::update::account_switch::selected_id;

/// États lus dans la file : ce qui attend, avance, a échoué ou attend une décision.
pub const QUEUE_STATES: [ActionState; 4] = [
    ActionState::Pending,
    ActionState::InProgress,
    ActionState::Failed,
    ActionState::Uncertain,
];

/// Ouvre la vue sur le compte sélectionné et lit sa boîte d'envoi et sa file.
pub fn open_queue(state: &mut AppState) -> Vec<Command> {
    let Some(account) = selected_id(state) else {
        state.status.message = Some("Aucun compte sélectionné".to_string());
        return Vec::new();
    };
    state.queue.open(account);
    refresh_queue(state)
}

/// Referme la vue ; les lectures encore attendues seront ignorées.
pub fn close_queue(state: &mut AppState) -> Vec<Command> {
    state.queue.close();
    Vec::new()
}

/// Relit la vue ouverte ; pendant une lecture, un seul tour de plus est noté pour la fin.
pub fn refresh_queue(state: &mut AppState) -> Vec<Command> {
    let Some(round) = state.queue.start_round() else {
        return Vec::new();
    };
    match state.queue.account() {
        Some(account) => list_commands(account, round),
        None => Vec::new(),
    }
}

/// Lectures d'un tour : boîte d'envoi et actions en file du compte.
fn list_commands(account: &AccountId, round: u64) -> Vec<Command> {
    vec![
        Command::Execute {
            tag: Tag::Outbox {
                account: account.clone(),
                round,
            },
            request: Request::Compose(ComposeRequest::ListOutbox {
                account: account.clone(),
            }),
        },
        Command::Execute {
            tag: Tag::QueueActions {
                account: account.clone(),
                round,
            },
            request: Request::Queue(QueueRequest::ListActions {
                account: account.clone(),
                states: QUEUE_STATES.to_vec(),
            }),
        },
    ]
}
