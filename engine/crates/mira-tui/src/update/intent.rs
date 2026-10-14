//! Intentions de l'utilisateur, indépendantes des touches qui les produisent.

use super::intent_actions::on_message_intent;
use super::search_intent::SearchIntent;
use super::{
    account_switch, navigation, pagination, panes, popup_intent, search_intent, sync_status,
};
use crate::commands::Command;
use crate::state::AppState;

/// Ce que l'utilisateur veut faire.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Intent {
    MoveDown,
    MoveUp,
    Top,
    Bottom,
    Open,
    Back,
    NextPane,
    PrevPane,
    NextAccount,
    PrevAccount,
    Quit,
    Archive,
    Trash,
    DeletePermanently,
    ToggleFlag,
    ToggleRead,
    OpenMovePicker,
    Undo,
    Confirm,
    Cancel,
    Search(SearchIntent),
    StartSync,
}

/// Route une intention vers son gestionnaire ; une fenêtre ouverte la reçoit en premier.
pub fn on_intent(state: &mut AppState, intent: Intent) -> Vec<Command> {
    if let Some(commands) = route_modal(state, intent) {
        return commands;
    }
    match intent {
        Intent::MoveDown
        | Intent::MoveUp
        | Intent::Top
        | Intent::Bottom
        | Intent::Open
        | Intent::Back
        | Intent::NextPane
        | Intent::PrevPane
        | Intent::NextAccount
        | Intent::PrevAccount
        | Intent::Quit => on_navigation_intent(state, intent),
        Intent::Archive
        | Intent::Trash
        | Intent::DeletePermanently
        | Intent::ToggleFlag
        | Intent::ToggleRead
        | Intent::OpenMovePicker
        | Intent::Undo
        | Intent::Confirm
        | Intent::Cancel => on_message_intent(state, intent),
        Intent::Search(search) => search_intent::route_idle(state, search),
        Intent::StartSync => sync_status::start_sync(state),
    }
}

/// Route l'intention vers la fenêtre ou la saisie de recherche ouverte, qui la reçoit avant tout.
fn route_modal(state: &mut AppState, intent: Intent) -> Option<Vec<Command>> {
    if state.popup.is_open() {
        return Some(popup_intent::route(state, intent));
    }
    if state.search.is_editing() {
        return Some(search_intent::route_editing(state, intent));
    }
    None
}

/// Intentions de navigation entre panneaux, comptes et messages.
fn on_navigation_intent(state: &mut AppState, intent: Intent) -> Vec<Command> {
    match intent {
        Intent::MoveDown => return scroll(state, navigation::move_down),
        Intent::MoveUp => return scroll(state, navigation::move_up),
        Intent::Top => return scroll(state, navigation::top),
        Intent::Bottom => return scroll(state, navigation::bottom),
        Intent::Open => return panes::open(state),
        Intent::Back => panes::back(state),
        Intent::NextPane => panes::next_pane(state),
        Intent::PrevPane => panes::prev_pane(state),
        Intent::NextAccount => return account_switch::next_account(state),
        Intent::PrevAccount => return account_switch::prev_account(state),
        Intent::Quit => return panes::quit(state),
        Intent::Archive
        | Intent::Trash
        | Intent::DeletePermanently
        | Intent::ToggleFlag
        | Intent::ToggleRead
        | Intent::OpenMovePicker
        | Intent::Undo
        | Intent::Confirm
        | Intent::Cancel
        | Intent::Search(_)
        | Intent::StartSync => {}
    }
    Vec::new()
}

/// Déplace la sélection puis demande la page suivante si la fin de liste approche.
fn scroll(state: &mut AppState, step: fn(&mut AppState)) -> Vec<Command> {
    step(state);
    pagination::next_page(state)
}
