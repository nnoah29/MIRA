//! Journal d'événements : lecture par pages, chaque page lue « après le dernier numéro » de la précédente.

use mira_assembly::api::{EventRequest, Request};
use mira_assembly::mira_vocabulary::{EventFilter, EventSeq};

use super::intent::PageOp;
use super::operation::say;
use crate::commands::{Command, SettingsTag, Tag};
use crate::state::AppState;
use crate::state::settings::{Dialog, EVENTS_PAGE, PageMove, SettingsTab};

/// Applique un changement de page ; sur une autre page de la vue ou sous une fenêtre il est ignoré.
pub fn on_page_op(state: &mut AppState, op: PageOp) -> Vec<Command> {
    if state.settings.tab != SettingsTab::Events || state.settings.dialog != Dialog::None {
        return Vec::new();
    }
    match op {
        PageOp::Next => next_page(state),
        PageOp::Previous => previous_page(state),
        PageOp::First => load_first_page(state),
        PageOp::Last => page_command(state, None, PageMove::First, true),
    }
}

/// Lit la première page du journal.
pub fn load_first_page(state: &mut AppState) -> Vec<Command> {
    page_command(state, None, PageMove::First, false)
}

/// Lit la page qui suit la page affichée, s'il y en a une.
fn next_page(state: &mut AppState) -> Vec<Command> {
    if !state.settings.events.is_loaded() {
        return Vec::new();
    }
    if state.settings.events.at_end() {
        return say(state, "Fin du journal");
    }
    let after = state.settings.events.next_after();
    page_command(state, after, PageMove::Next, false)
}

/// Relit la page d'avant, s'il y en a une.
fn previous_page(state: &mut AppState) -> Vec<Command> {
    match state.settings.events.previous_after() {
        Some(after) => page_command(state, after, PageMove::Previous, false),
        None => say(state, "Début du journal"),
    }
}

/// Commande de lecture d'une page après ce numéro, suivie par un jeton neuf.
pub fn page_command(
    state: &mut AppState,
    after: Option<EventSeq>,
    movement: PageMove,
    seeking_end: bool,
) -> Vec<Command> {
    let token = state.settings.begin_events(after, movement, seeking_end);
    let filter = EventFilter {
        after,
        ..EventFilter::default()
    };
    vec![Command::Execute {
        tag: Tag::Settings(SettingsTag::Events(token)),
        request: Request::Event(EventRequest::GetEvents {
            filter,
            limit: EVENTS_PAGE,
        }),
    }]
}
