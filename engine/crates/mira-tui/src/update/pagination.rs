//! Demande de la page suivante de messages quand la sélection approche de la fin.

use mira_assembly::api::{MessageRequest, Request};
use mira_assembly::reading::ListQuery;

use crate::commands::{Command, Tag};
use crate::state::{AppState, Pane};

const PREFETCH_DISTANCE: usize = 5;

/// Commande de chargement de la page suivante, une seule fois par curseur.
pub fn next_page(state: &mut AppState) -> Vec<Command> {
    let Some(selected) = state.messages.selection.selected else {
        return Vec::new();
    };
    let near_end = selected + PREFETCH_DISTANCE >= state.messages.len();
    if state.focus != Pane::List || !near_end {
        return Vec::new();
    }
    let (Some(view), Some(cursor)) = (
        state.messages.view().cloned(),
        state.messages.unrequested_cursor().cloned(),
    ) else {
        return Vec::new();
    };
    let mut query = ListQuery::of(view.clone());
    query.page.cursor = Some(cursor.clone());
    state.messages.mark_requested(cursor);
    vec![Command::Execute {
        tag: Tag::Messages { view, append: true },
        request: Request::Message(MessageRequest::ListMessages(query)),
    }]
}
