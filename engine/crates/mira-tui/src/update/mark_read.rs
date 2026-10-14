//! Marquage « lu » d'un message par une action du moteur.

use mira_assembly::api::{ActionRequest, Request, TargetSelection};
use mira_assembly::mira_vocabulary::{ActionKind, MessageId};

use crate::commands::{Command, Tag};
use crate::state::AppState;

/// Action « marquer lu » pour ce message, avec une clé d'idempotence tirée du compteur de l'état.
pub fn mark_read_command(state: &mut AppState, id: &MessageId) -> Command {
    let idempotency_key = state.fresh_key(&format!("read-{id}"));
    Command::Execute {
        tag: Tag::Action,
        request: Request::Action(ActionRequest::ApplyAction {
            kind: ActionKind::MarkRead { read: true },
            targets: TargetSelection::Messages {
                messages: vec![id.clone()],
            },
            idempotency_key,
        }),
    }
}
