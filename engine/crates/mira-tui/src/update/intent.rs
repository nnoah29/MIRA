//! Intentions de l'utilisateur, indépendantes des touches qui les produisent.

use super::account_form::AccountFormIntent;
use super::compose::ComposeIntent;
use super::compose::route::{route_creating, route_editing, route_idle};
use super::drafts::DraftsIntent;
use super::intent_actions::on_message_intent;
use super::queue::QueueIntent;
use super::queue::route::route_open;
use super::search_intent::SearchIntent;
use super::{
    account_switch, help, navigation, pagination, panes, popup_intent, search_intent, sync_status,
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
    Compose(ComposeIntent),
    Queue(QueueIntent),
    Drafts(DraftsIntent),
    StartSync,
    Help,
    AccountForm(AccountFormIntent),
}

/// Route une intention vers son gestionnaire ; une fenêtre ouverte la reçoit en premier.
#[rustfmt::skip]
pub fn on_intent(state: &mut AppState, intent: Intent) -> Vec<Command> {
    if let Some(commands) = route_modal(state, intent) {
        return commands;
    }
    match intent {
        Intent::MoveDown | Intent::MoveUp | Intent::Top | Intent::Bottom | Intent::Open | Intent::Back
        | Intent::NextPane | Intent::PrevPane | Intent::NextAccount | Intent::PrevAccount
        | Intent::Quit => on_navigation_intent(state, intent),
        Intent::Archive | Intent::Trash | Intent::DeletePermanently | Intent::ToggleFlag | Intent::ToggleRead
        | Intent::OpenMovePicker | Intent::Undo | Intent::Confirm | Intent::Cancel => on_message_intent(state, intent),
        Intent::Search(search) => search_intent::route_idle(state, search),
        Intent::Compose(compose) => route_idle(state, compose),
        Intent::Queue(queue) => super::queue::route::route_idle(state, queue),
        Intent::Drafts(drafts) => super::drafts::route::route_idle(state, drafts),
        Intent::StartSync => sync_status::start_sync(state),
        Intent::Help => help::open_help(state),
        Intent::AccountForm(form) => super::account_form::route::route_idle(state, form),
    }
}

/// Route l'intention vers le formulaire d'ajout de compte, la fenêtre, l'éditeur, la vue « file », la vue « Brouillons » ou la saisie de recherche ouverte, qui la reçoit avant tout.
fn route_modal(state: &mut AppState, intent: Intent) -> Option<Vec<Command>> {
    if state.account_form.is_open() {
        return Some(super::account_form::route::route_open(state, intent));
    }
    if state.compose.is_creating() {
        return Some(route_creating(state, intent));
    }
    if state.popup.is_open() {
        return Some(popup_intent::route(state, intent));
    }
    if state.compose.open().is_some() {
        return Some(route_editing(state, intent));
    }
    if state.queue.is_open() {
        return Some(route_open(state, intent));
    }
    if state.drafts.is_open() {
        return Some(super::drafts::route::route_open(state, intent));
    }
    if state.search.is_editing() {
        return Some(search_intent::route_editing(state, intent));
    }
    None
}

/// Intentions de navigation entre panneaux, comptes et messages.
#[rustfmt::skip]
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
        Intent::Archive | Intent::Trash | Intent::DeletePermanently | Intent::ToggleFlag | Intent::ToggleRead
        | Intent::OpenMovePicker | Intent::Undo | Intent::Confirm | Intent::Cancel | Intent::Search(_)
        | Intent::Compose(_) | Intent::Queue(_) | Intent::Drafts(_) | Intent::StartSync | Intent::Help | Intent::AccountForm(_) => {}
    }
    Vec::new()
}

/// Déplace la sélection puis demande la page suivante si la fin de liste approche.
fn scroll(state: &mut AppState, step: fn(&mut AppState)) -> Vec<Command> {
    step(state);
    pagination::next_page(state)
}
