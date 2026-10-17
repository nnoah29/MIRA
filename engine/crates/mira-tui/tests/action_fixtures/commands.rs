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
        tag: Tag::Undo(ActionId::from_raw(action)),
        request: Request::Action(ActionRequest::UndoAction {
            action: ActionId::from_raw(action),
        }),
    }
}

/// Réponse du moteur à une annulation.
pub fn undo_reply(action: &str, result: Result<(), MiraError>) -> Msg {
    Msg::Reply(Reply {
        tag: Tag::Undo(ActionId::from_raw(action)),
        result: result.map(|()| Response::Done),
    })
}

/// Liste des messages de la boîte de réception attendue pour recharger la vue.
pub fn reload_command() -> Command {
    let view = crate::message_fixtures::inbox_view();
    Command::Execute {
        tag: Tag::Messages {
            view: view.clone(),
            after: None,
        },
        request: Request::Message(mira_assembly::api::MessageRequest::ListMessages(
            mira_assembly::reading::ListQuery::of(view),
        )),
    }
}

/// Réponse du moteur à ce rechargement, avec ces messages.
pub fn reload_reply(count: usize) -> Msg {
    Msg::Reply(Reply {
        tag: Tag::Messages {
            view: crate::message_fixtures::inbox_view(),
            after: None,
        },
        result: Ok(Response::Messages(crate::message_fixtures::page(
            0, count, None,
        ))),
    })
}
