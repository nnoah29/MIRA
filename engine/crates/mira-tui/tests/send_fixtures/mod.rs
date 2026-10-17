//! Commandes et réponses de l'envoi d'un brouillon, et envoi confirmé par intentions.
#![allow(dead_code)]

use mira_assembly::api::{ComposeRequest, Request, Response, WriteResponse};
use mira_assembly::mira_vocabulary::error::codes::internal::BUG;
use mira_assembly::mira_vocabulary::{ActionId, DraftId, ErrorCategory, IdempotencyKey, MiraError};
use mira_tui::commands::{Command, Tag};
use mira_tui::state::AppState;
use mira_tui::update::{ComposeIntent, Intent, Msg, Reply, update};

/// Envoi attendu de ce brouillon avec cette clé, sans date choisie.
pub fn send_command(draft: &str, key: &str) -> Command {
    Command::Execute {
        tag: Tag::Sent {
            draft: DraftId::from_raw(draft),
            key: IdempotencyKey(key.to_string()),
        },
        request: Request::Compose(ComposeRequest::SendDraft {
            draft: DraftId::from_raw(draft),
            at: None,
            idempotency_key: IdempotencyKey(key.to_string()),
        }),
    }
}

/// Réponse du moteur à l'envoi de ce brouillon : l'action d'envoi en file, ou une erreur.
pub fn sent_reply(draft: &str, key: &str, result: Result<&str, MiraError>) -> Msg {
    Msg::Reply(Reply {
        tag: Tag::Sent {
            draft: DraftId::from_raw(draft),
            key: IdempotencyKey(key.to_string()),
        },
        result: result
            .map(|action| Response::Write(WriteResponse::Action(ActionId::from_raw(action)))),
    })
}

/// Réponse d'échec du moteur à l'envoi de ce brouillon avec cette clé.
pub fn failed_send(draft: &str, key: &str) -> Msg {
    let error = MiraError::new(ErrorCategory::Network, BUG, "serveur injoignable");
    sent_reply(draft, key, Err(error))
}

/// Ctrl+Entrée puis confirmation ; rend les commandes de la confirmation.
pub fn send_confirmed(state: &mut AppState) -> Vec<Command> {
    update(state, Msg::Intent(Intent::Compose(ComposeIntent::Send)));
    update(state, Msg::Intent(Intent::Confirm))
}

/// Nombre de commandes `SendDraft` dans la liste.
pub fn count_sends(commands: &[Command]) -> usize {
    commands
        .iter()
        .filter(|command| {
            matches!(
                command,
                Command::Execute {
                    request: Request::Compose(ComposeRequest::SendDraft { .. }),
                    ..
                }
            )
        })
        .count()
}
