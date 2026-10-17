//! Commandes et réponses attendues autour du lecteur.
#![allow(dead_code)]

use mira_assembly::api::{ActionRequest, MessageRequest, Request, Response, TargetSelection};
use mira_assembly::mira_vocabulary::{ActionKind, IdempotencyKey, MessageId, MiraError};
use mira_assembly::reading::BodyView;
use mira_tui::commands::{Command, Tag};
use mira_tui::update::{Msg, Reply};

/// Commande attendue pour lire le corps local du message.
pub fn get_body_command(id: &str) -> Command {
    Command::Execute {
        tag: Tag::Body(MessageId::from_raw(id)),
        request: Request::Message(MessageRequest::GetMessageBody {
            message: MessageId::from_raw(id),
        }),
    }
}

/// Commande attendue pour télécharger le corps depuis le serveur.
pub fn fetch_body_command(id: &str) -> Command {
    Command::Execute {
        tag: Tag::Body(MessageId::from_raw(id)),
        request: Request::Message(MessageRequest::FetchMessageBody {
            message: MessageId::from_raw(id),
        }),
    }
}

/// Réponse du moteur étiquetée pour le corps de ce message.
pub fn body_reply(id: &str, result: Result<BodyView, MiraError>) -> Msg {
    Msg::Reply(Reply {
        tag: Tag::Body(MessageId::from_raw(id)),
        result: result.map(Response::MessageBody),
    })
}

/// Action « marquer lu » attendue pour ce message, avec la clé donnée.
pub fn mark_read_command(id: &str, key: &str) -> Command {
    Command::Execute {
        tag: Tag::Action,
        request: Request::Action(ActionRequest::ApplyAction {
            kind: ActionKind::MarkRead { read: true },
            targets: TargetSelection::Messages {
                messages: vec![MessageId::from_raw(id)],
            },
            idempotency_key: IdempotencyKey(key.to_string()),
        }),
    }
}
