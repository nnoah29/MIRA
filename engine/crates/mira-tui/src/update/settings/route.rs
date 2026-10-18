//! Routage des intentions : hors vue seule l'ouverture agit ; vue ouverte, elle reçoit tout (modale).

use super::account_actions::on_account_op;
use super::account_nav::move_account;
use super::events_log::on_page_op;
use super::form_input::on_form_op;
use super::identity_actions::on_identity_op;
use super::intent::SettingsIntent;
use super::maintenance::on_maintenance_op;
use super::maintenance_ask::confirm_ask;
use super::open::{back, open_settings, switch_tab};
use crate::commands::Command;
use crate::state::settings::{Dialog, SettingsTab};
use crate::state::{AppState, Selection};
use crate::update::palette::route::route_idle as palette_idle;
use crate::update::{Intent, panes};

/// Intention de Paramètres reçue hors vue : seule l'ouverture agit.
pub fn route_idle(state: &mut AppState, intent: SettingsIntent) -> Vec<Command> {
    match intent {
        SettingsIntent::Show(tab) => open_settings(state, tab),
        SettingsIntent::NextTab
        | SettingsIntent::PrevTab
        | SettingsIntent::Account(_)
        | SettingsIntent::Identity(_)
        | SettingsIntent::Maintenance(_)
        | SettingsIntent::Page(_)
        | SettingsIntent::Form(_) => Vec::new(),
    }
}

/// Vue ouverte : déplacement, pages, opérations, saisie, retour et sortie ; les autres intentions sont ignorées.
#[rustfmt::skip]
pub fn route_open(state: &mut AppState, intent: Intent) -> Vec<Command> {
    match intent {
        Intent::MoveDown => step(state, Selection::down),
        Intent::MoveUp => step(state, Selection::up),
        Intent::Top => step(state, Selection::top),
        Intent::Bottom => step(state, Selection::bottom),
        Intent::NextAccount => step_account(state, Selection::down),
        Intent::PrevAccount => step_account(state, Selection::up),
        Intent::Back | Intent::Cancel => back(state),
        Intent::Confirm => confirm_ask(state),
        Intent::Quit => panes::quit(state),
        Intent::Settings(settings) => on_settings(state, settings),
        Intent::Palette(palette) if state.settings.dialog == Dialog::None => palette_idle(state, palette),
        Intent::Palette(_) | Intent::Open | Intent::NextPane | Intent::PrevPane | Intent::Archive | Intent::Trash
        | Intent::DeletePermanently | Intent::ToggleFlag | Intent::ToggleRead | Intent::OpenMovePicker
        | Intent::Undo | Intent::Search(_) | Intent::Compose(_) | Intent::Queue(_) | Intent::Drafts(_)
        | Intent::StartSync | Intent::Help | Intent::AccountForm(_) => Vec::new(),
    }
}

/// Déplace la sélection de la page affichée, sauf sous une fenêtre.
fn step(state: &mut AppState, move_by: fn(&mut Selection, usize)) -> Vec<Command> {
    if state.settings.dialog != Dialog::None {
        return Vec::new();
    }
    match state.settings.tab {
        SettingsTab::Accounts => return move_account(state, move_by),
        SettingsTab::Identities => state.settings.identities.move_selection(move_by),
        SettingsTab::Events => state.settings.events.move_selection(move_by),
        SettingsTab::Maintenance => {}
    }
    Vec::new()
}

/// Vise le compte voisin, sauf sous une fenêtre.
fn step_account(state: &mut AppState, move_by: fn(&mut Selection, usize)) -> Vec<Command> {
    if state.settings.dialog != Dialog::None {
        return Vec::new();
    }
    move_account(state, move_by)
}

/// Applique une intention propre à la vue ; le changement d'onglet est refusé sous une fenêtre.
fn on_settings(state: &mut AppState, intent: SettingsIntent) -> Vec<Command> {
    let browsing = state.settings.dialog == Dialog::None;
    match intent {
        SettingsIntent::Show(tab) if browsing => switch_tab(state, tab),
        SettingsIntent::NextTab if browsing => switch_tab(state, state.settings.tab.next()),
        SettingsIntent::PrevTab if browsing => switch_tab(state, state.settings.tab.previous()),
        SettingsIntent::Show(_) | SettingsIntent::NextTab | SettingsIntent::PrevTab => Vec::new(),
        SettingsIntent::Account(op) => on_account_op(state, op),
        SettingsIntent::Identity(op) => on_identity_op(state, op),
        SettingsIntent::Maintenance(op) => on_maintenance_op(state, op),
        SettingsIntent::Page(op) => on_page_op(state, op),
        SettingsIntent::Form(op) => on_form_op(state, op),
    }
}
