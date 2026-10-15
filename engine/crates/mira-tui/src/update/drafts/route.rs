//! Routage des intentions : hors vue seule l'ouverture agit ; vue ouverte, elle reçoit tout (modale).

use super::delete::ask_delete;
use super::intent::DraftsIntent;
use super::open::{close_drafts, open_drafts};
use super::reopen::reopen_selected;
use crate::commands::Command;
use crate::state::{AppState, Selection};
use crate::update::{Intent, panes};

/// Intention de brouillons reçue hors vue : seule l'ouverture agit.
pub fn route_idle(state: &mut AppState, intent: DraftsIntent) -> Vec<Command> {
    match intent {
        DraftsIntent::Open => open_drafts(state),
        DraftsIntent::AskDelete | DraftsIntent::ConfirmDelete => Vec::new(),
    }
}

/// Vue ouverte : déplacement, réouverture, suppression, retour et sortie ; les autres intentions sont ignorées.
#[rustfmt::skip]
pub fn route_open(state: &mut AppState, intent: Intent) -> Vec<Command> {
    match intent {
        Intent::MoveDown => step(state, Selection::down),
        Intent::MoveUp => step(state, Selection::up),
        Intent::Top => step(state, Selection::top),
        Intent::Bottom => step(state, Selection::bottom),
        Intent::Open => reopen_selected(state),
        Intent::Back | Intent::Cancel => close_drafts(state),
        Intent::Quit => panes::quit(state),
        Intent::Drafts(DraftsIntent::AskDelete) => ask_delete(state),
        Intent::Drafts(DraftsIntent::Open | DraftsIntent::ConfirmDelete) | Intent::NextPane | Intent::PrevPane
        | Intent::NextAccount | Intent::PrevAccount | Intent::Archive | Intent::Trash | Intent::DeletePermanently
        | Intent::ToggleFlag | Intent::ToggleRead | Intent::OpenMovePicker | Intent::Undo | Intent::Confirm
        | Intent::Search(_) | Intent::Compose(_) | Intent::Queue(_) | Intent::StartSync | Intent::Help => Vec::new(),
    }
}

/// Déplace la sélection de la vue.
fn step(state: &mut AppState, move_by: fn(&mut Selection, usize)) -> Vec<Command> {
    state.drafts.move_selection(move_by);
    Vec::new()
}
