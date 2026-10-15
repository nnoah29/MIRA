//! Intentions de recherche : hors saisie elles ouvrent ou ferment, en saisie elles tapent.

use super::intent::Intent;
use super::{panes, search};
use crate::commands::Command;
use crate::state::AppState;

/// Ce que l'utilisateur fait avec la recherche.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchIntent {
    Open,
    Char(char),
    Backspace,
    Submit,
    Leave,
}

/// Route une intention de recherche reçue hors saisie.
pub fn route_idle(state: &mut AppState, intent: SearchIntent) -> Vec<Command> {
    match intent {
        SearchIntent::Open => search::open_search(state),
        SearchIntent::Leave => search::leave_search(state),
        SearchIntent::Submit | SearchIntent::Char(_) | SearchIntent::Backspace => Vec::new(),
    }
}

/// Pendant la saisie, la recherche reçoit toutes les intentions ; seules celles qui la concernent agissent.
#[rustfmt::skip]
pub fn route_editing(state: &mut AppState, intent: Intent) -> Vec<Command> {
    match intent {
        Intent::Search(search) => type_into_search(state, search),
        Intent::Open | Intent::Confirm => search::submit_search(state),
        Intent::Back | Intent::Cancel => search::leave_search(state),
        Intent::Quit => panes::quit(state),
        Intent::MoveDown | Intent::MoveUp | Intent::Top | Intent::Bottom | Intent::NextPane | Intent::PrevPane
        | Intent::NextAccount | Intent::PrevAccount | Intent::Archive | Intent::Trash | Intent::DeletePermanently
        | Intent::ToggleFlag | Intent::ToggleRead | Intent::OpenMovePicker | Intent::Undo | Intent::StartSync
        | Intent::Help | Intent::Compose(_) | Intent::Queue(_) | Intent::Drafts(_) | Intent::Palette(_)
        | Intent::Settings(_) => Vec::new(),
    }
}

/// Applique une intention de recherche à la saisie ouverte.
fn type_into_search(state: &mut AppState, intent: SearchIntent) -> Vec<Command> {
    match intent {
        SearchIntent::Char(c) => state.search.push(c),
        SearchIntent::Backspace => state.search.pop(),
        SearchIntent::Submit => return search::submit_search(state),
        SearchIntent::Leave => return search::leave_search(state),
        SearchIntent::Open => {}
    }
    Vec::new()
}
