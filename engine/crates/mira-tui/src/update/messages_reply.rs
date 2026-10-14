//! Réponse du moteur à une page de messages.

use mira_assembly::api::Response;
use mira_assembly::mira_vocabulary::{MiraError, ViewRef};

use super::accounts_reply::{report, unexpected};
use crate::commands::Command;
use crate::state::AppState;

/// Range la page reçue : remplace la liste ou l'allonge, sauf si la vue a changé entre-temps.
pub fn on_messages(
    state: &mut AppState,
    view: ViewRef,
    append: bool,
    result: Result<Response, MiraError>,
) -> Vec<Command> {
    let current = state.messages.view() == Some(&view);
    match result {
        Ok(Response::Messages(page)) if append && current => state.messages.append_page(page),
        Ok(Response::Messages(page)) if !append => state.messages.replace_page(view, page),
        Ok(Response::Messages(_)) => {}
        Ok(_) => return unexpected(state),
        Err(error) => return fail(state, current && append, error),
    }
    Vec::new()
}

/// Signale l'erreur et, pour une page suivante, autorise une nouvelle tentative.
fn fail(state: &mut AppState, release: bool, error: MiraError) -> Vec<Command> {
    if release {
        state.messages.release_request();
    }
    report(state, error)
}
