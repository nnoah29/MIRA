//! Réponse à la lecture d'une page du journal : rangée sans jamais réagir aux événements lus (AG-41).

use mira_assembly::api::Response;
use mira_assembly::mira_vocabulary::{Event, MiraError};

use super::events_log::page_command;
use super::operation::say;
use crate::commands::Command;
use crate::state::AppState;
use crate::state::settings::{PageMove, PendingPage};
use crate::update::accounts_reply::{report, unexpected};

/// Range la page reçue si c'est la lecture attendue de la vue ouverte ; les événements lus ne déclenchent rien.
pub fn on_events(
    state: &mut AppState,
    token: u64,
    result: Result<Response, MiraError>,
) -> Vec<Command> {
    let Some(page) = state.settings.take_events(token) else {
        return Vec::new();
    };
    match result {
        Ok(Response::Events(events)) => store_page(state, page, events),
        Ok(_) => unexpected(state),
        Err(error) => report(state, error),
    }
}

/// Range la page ; lit la suivante tant qu'on cherche la fin du journal et que la page est pleine.
fn store_page(state: &mut AppState, page: PendingPage, events: Vec<Event>) -> Vec<Command> {
    let empty = events.is_empty();
    if state.settings.events.apply(page, events) {
        let after = state.settings.events.next_after();
        return page_command(state, after, PageMove::Next, true);
    }
    if empty && page.movement == PageMove::Next && !page.seeking_end {
        return say(state, "Fin du journal");
    }
    Vec::new()
}
