//! Rechargement discret de la vue courante : la liste reste affichée pendant l'attente.

use mira_assembly::api::{MessageRequest, Request};
use mira_assembly::reading::ListQuery;

use crate::commands::{Command, Tag};
use crate::state::AppState;

/// Redemande la première page de la vue affichée, sauf pendant une recherche ou un chargement en cours.
pub fn refresh_view(state: &mut AppState) -> Vec<Command> {
    let Some(view) = state.messages.view().cloned() else {
        return Vec::new();
    };
    if state.messages.is_loading() || state.search.is_active() {
        return Vec::new();
    }
    state.messages.mark_loading();
    vec![Command::Execute {
        tag: Tag::Messages {
            view: view.clone(),
            after: None,
        },
        request: Request::Message(MessageRequest::ListMessages(ListQuery::of(view))),
    }]
}
