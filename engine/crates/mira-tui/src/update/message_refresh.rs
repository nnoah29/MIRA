//! Rechargement discret de la vue courante : la liste reste affichée et garde tous ses messages chargés.

use mira_assembly::api::{MessageRequest, Request};
use mira_assembly::reading::ListQuery;

use crate::commands::{Command, Tag};
use crate::state::AppState;

const MAX_PAGE: u32 = 500;

/// Redemande la première page de la vue affichée, sauf pendant une recherche ou un chargement en cours.
pub fn refresh_view(state: &mut AppState) -> Vec<Command> {
    let Some(view) = state.messages.view().cloned() else {
        return Vec::new();
    };
    if state.messages.is_loading() || state.search.is_active() {
        return Vec::new();
    }
    state.messages.mark_loading();
    let mut query = ListQuery::of(view.clone());
    query.page.limit = refresh_limit(state.messages.len(), query.page.limit);
    vec![Command::Execute {
        tag: Tag::Messages { view, after: None },
        request: Request::Message(MessageRequest::ListMessages(query)),
    }]
}

/// Taille de la page de rafraîchissement : tout ce qui est chargé (borné à 500 par le moteur), au moins la page normale.
fn refresh_limit(loaded: usize, normal: u32) -> u32 {
    u32::try_from(loaded)
        .unwrap_or(u32::MAX)
        .max(normal)
        .min(MAX_PAGE)
}
