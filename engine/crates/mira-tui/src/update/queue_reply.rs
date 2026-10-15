//! Réponses du moteur aux lectures de la file (compte et tour attendus seulement) et à ses modifications.

use mira_assembly::api::{Response, WriteResponse};
use mira_assembly::mira_vocabulary::{AccountId, ActionId, MiraError};

use super::accounts_reply::{report, unexpected};
use super::queue::open::refresh_queue;
use crate::commands::Command;
use crate::state::AppState;
use crate::state::queue::QueueList;
use crate::view::safe_text::display_safe;

/// Range la boîte d'envoi lue, si c'est la lecture attendue.
pub fn on_outbox(
    state: &mut AppState,
    account: AccountId,
    round: u64,
    result: Result<Response, MiraError>,
) -> Vec<Command> {
    if !state.queue.awaits(QueueList::Outbox, &account, round) {
        return Vec::new();
    }
    match result {
        Ok(Response::Write(WriteResponse::Outbox(entries))) => state.queue.replace_outbox(entries),
        Ok(_) => {
            unexpected(state);
        }
        Err(error) => {
            report(state, error);
        }
    }
    settle(state, QueueList::Outbox)
}

/// Range les actions lues, si c'est la lecture attendue.
pub fn on_queue_actions(
    state: &mut AppState,
    account: AccountId,
    round: u64,
    result: Result<Response, MiraError>,
) -> Vec<Command> {
    if !state.queue.awaits(QueueList::Actions, &account, round) {
        return Vec::new();
    }
    match result {
        Ok(Response::Write(WriteResponse::QueuedActions(actions))) => {
            state.queue.replace_actions(actions);
        }
        Ok(_) => {
            unexpected(state);
        }
        Err(error) => {
            report(state, error);
        }
    }
    settle(state, QueueList::Actions)
}

/// Modification acceptée : la vue est relue (et l'annulation d'envoi annoncée) ; refusée : l'erreur est affichée.
pub fn on_queue_change(
    state: &mut AppState,
    action: ActionId,
    result: Result<Response, MiraError>,
) -> Vec<Command> {
    let cancelled = state.queue.take_cancel(&action);
    match result {
        Ok(_) => {
            if let Some(subject) = cancelled {
                let subject = display_safe(&subject);
                state.status.message = Some(format!(
                    "Envoi « {subject} » annulé : le brouillon revient en édition (rouvrez-le avec B)"
                ));
            }
            refresh_queue(state)
        }
        Err(error) => report(state, error),
    }
}

/// Note l'arrivée de la lecture ; relance un tour si un événement en a demandé un pendant celle-ci.
fn settle(state: &mut AppState, list: QueueList) -> Vec<Command> {
    if state.queue.finish(list) {
        return refresh_queue(state);
    }
    Vec::new()
}
