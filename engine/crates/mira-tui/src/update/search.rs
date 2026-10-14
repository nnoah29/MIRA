//! Ouverture, envoi et sortie de la recherche locale.

use mira_assembly::api::{Request, SearchRequest};
use mira_assembly::mira_vocabulary::PageRequest;
use mira_assembly::reading::SearchFilters;

use super::message_load::{load_selected_mailbox, load_view};
use crate::commands::{Command, Tag};
use crate::state::{AppState, Pane};

/// Ouvre la saisie en mémorisant la vue à restaurer.
pub fn open_search(state: &mut AppState) -> Vec<Command> {
    let current = state.messages.view().cloned();
    state.search.open(current);
    Vec::new()
}

/// Envoie la requête tapée ; une requête vide n'envoie rien.
pub fn submit_search(state: &mut AppState) -> Vec<Command> {
    let Some(query) = state.search.submit() else {
        return Vec::new();
    };
    state.messages.reset();
    state.focus = Pane::List;
    let request = SearchRequest::Search {
        query: query.clone(),
        filters: SearchFilters::default(),
        page: PageRequest::default(),
    };
    vec![Command::Execute {
        tag: Tag::Search(query),
        request: Request::Search(request),
    }]
}

/// Quitte la recherche et recharge la vue d'avant, ou la boîte sélectionnée à défaut.
pub fn leave_search(state: &mut AppState) -> Vec<Command> {
    if !state.search.leave() {
        return Vec::new();
    }
    match state.search.take_previous() {
        Some(view) => load_view(state, view),
        None => load_selected_mailbox(state),
    }
}
