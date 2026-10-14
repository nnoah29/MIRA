//! Fabrication de la commande d'une action de l'utilisateur et mise à jour optimiste de l'écran.

use mira_assembly::api::{ActionRequest, Request, TargetSelection};
use mira_assembly::mira_vocabulary::{ActionKind, MessageId, MessageSummary};

use crate::commands::{Command, Tag};
use crate::state::{AppState, Pane, ReaderState};

/// Message sélectionné dans la liste, s'il y en a un.
pub fn selected_message(state: &AppState) -> Option<MessageSummary> {
    let index = state.messages.selection.selected?;
    state.messages.items().get(index).cloned()
}

/// Message visé par une action : celui ouvert si le lecteur a le focus (version de la liste si elle l'a), sinon la sélection.
pub fn target_message(state: &AppState) -> Option<MessageSummary> {
    if state.focus != Pane::Reader {
        return selected_message(state);
    }
    let open = state.reader.message()?;
    let listed = state.messages.items().iter().find(|m| m.id == open.id);
    Some(listed.unwrap_or(open).clone())
}

/// Commande d'action sur ces messages, avec une clé neuve dont la réponse est attendue.
pub fn apply_command(state: &mut AppState, kind: ActionKind, messages: Vec<MessageId>) -> Command {
    let key = state.fresh_key("act");
    state.actions.await_reply(key.clone());
    Command::Execute {
        tag: Tag::UserAction(key.clone()),
        request: Request::Action(ActionRequest::ApplyAction {
            kind,
            targets: TargetSelection::Messages { messages },
            idempotency_key: key,
        }),
    }
}

/// Retire ces messages de la liste et du lecteur, puis rend la commande de l'action.
pub fn send_removing(
    state: &mut AppState,
    kind: ActionKind,
    messages: Vec<MessageId>,
) -> Vec<Command> {
    for id in &messages {
        state.messages.remove(id);
        if state.reader.shows(id) {
            state.reader = ReaderState::default();
        }
    }
    vec![apply_command(state, kind, messages)]
}
