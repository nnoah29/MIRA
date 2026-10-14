//! Marquage « lu » d'un message par une action du moteur.

use mira_assembly::api::{ActionRequest, Request, TargetSelection};
use mira_assembly::mira_vocabulary::{ActionKind, MessageId};

use crate::commands::idempotency::next_key;
use crate::commands::{Command, Tag};
use crate::state::AppState;

/// Action « marquer lu » pour ce message, avec une clé d'idempotence tirée du compteur de l'état.
pub fn mark_read_command(state: &mut AppState, id: &MessageId) -> Command {
    let prefix = format!("{}-read-{}", state.key_prefix, id);
    let idempotency_key = next_key(&mut state.key_counter, &prefix);
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
