//! Réponse du moteur à une page de messages.

use mira_assembly::api::Response;
use mira_assembly::mira_vocabulary::{Cursor, MessageSummary, MiraError, Page, ViewRef};

use super::accounts_reply::{report, unexpected};
use crate::commands::Command;
use crate::state::AppState;

/// Range la page reçue : première page d'une vue voulue ou page suivante attendue ; le reste est périmé.
pub fn on_messages(
    state: &mut AppState,
    view: ViewRef,
    after: Option<Cursor>,
    result: Result<Response, MiraError>,
) -> Vec<Command> {
    if after.is_none() && state.messages.view() == Some(&view) {
        state.messages.finish_load();
    }
    match result {
        Ok(Response::Messages(page)) => store_page(state, view, after, page),
        Ok(_) => return unexpected(state),
        Err(error) => return fail(state, &view, after, error),
    }
    Vec::new()
}

/// Remplace la liste ou l'allonge, sauf si la réponse ne correspond plus à la demande en cours.
fn store_page(
    state: &mut AppState,
    view: ViewRef,
    after: Option<Cursor>,
    page: Page<MessageSummary>,
) {
    match after {
        None if state.messages.view() == Some(&view) => state.messages.replace_page(view, page),
        Some(cursor) if state.messages.awaits(&view, &cursor) => state.messages.append_page(page),
        None | Some(_) => {}
    }
}

/// Signale l'erreur et, pour une page suivante attendue, autorise une nouvelle tentative.
fn fail(
    state: &mut AppState,
    view: &ViewRef,
    after: Option<Cursor>,
    error: MiraError,
) -> Vec<Command> {
    if after.is_some_and(|cursor| state.messages.awaits(view, &cursor)) {
        state.messages.release_request();
    }
    report(state, error)
}
