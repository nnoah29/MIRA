//! Commandes et réponses attendues pour les actions et leur annulation.
#![allow(dead_code)]

use mira_assembly::api::{ActionRequest, Request, Response, TargetSelection, WriteResponse};
use mira_assembly::mira_vocabulary::{ActionId, ActionKind, IdempotencyKey, MessageId, MiraError};
use mira_tui::commands::{Command, Tag};
use mira_tui::update::{Msg, Reply};

/// Action de l'utilisateur attendue sur ces messages avec cette clé.
pub fn action_command(kind: ActionKind, ids: &[&str], key: &str) -> Command {
    let messages = ids.iter().map(|id| MessageId::from_raw(*id)).collect();
    Command::Execute {
        tag: Tag::UserAction(IdempotencyKey(key.to_string())),
        request: Request::Action(ActionRequest::ApplyAction {
            kind,
            targets: TargetSelection::Messages { messages },
            idempotency_key: IdempotencyKey(key.to_string()),
        }),
    }
}

/// Réponse du moteur à l'action envoyée avec cette clé.
pub fn action_reply(key: &str, result: Result<&str, MiraError>) -> Msg {
    Msg::Reply(Reply {
        tag: Tag::UserAction(IdempotencyKey(key.to_string())),
        result: result.map(|id| Response::Write(WriteResponse::Action(ActionId::from_raw(id)))),
    })
}

/// Annulation attendue de cette action.
pub fn undo_command(action: &str) -> Command {
    Command::Execute {
        tag: Tag::Undo,
        request: Request::Action(ActionRequest::UndoAction {
            action: ActionId::from_raw(action),
        }),
    }
}

/// Réponse du moteur à une annulation.
pub fn undo_reply(result: Result<(), MiraError>) -> Msg {
    Msg::Reply(Reply {
        tag: Tag::Undo,
        result: result.map(|()| Response::Done),
    })
}
