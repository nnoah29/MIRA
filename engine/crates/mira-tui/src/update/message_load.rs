//! Chargement de la première page de messages d'une boîte.

use mira_assembly::api::{MessageRequest, Request};
use mira_assembly::mira_vocabulary::ViewRef;
use mira_assembly::reading::ListQuery;

use crate::commands::{Command, Tag};
use crate::state::AppState;

/// Vide la liste, mémorise la vue voulue et demande sa première page.
pub fn load_view(state: &mut AppState, view: ViewRef) -> Vec<Command> {
    state.messages.begin_load(view.clone());
    vec![Command::Execute {
        tag: Tag::Messages {
            view: view.clone(),
            after: None,
        },
        request: Request::Message(MessageRequest::ListMessages(ListQuery::of(view))),
    }]
}

/// Charge la boîte sélectionnée, s'il y en a une.
pub fn load_selected_mailbox(state: &mut AppState) -> Vec<Command> {
    let Some(mailbox) = state.mailboxes.selected() else {
        return Vec::new();
    };
    let view = ViewRef::Mailbox {
        mailbox: mailbox.id.clone(),
    };
    load_view(state, view)
}
