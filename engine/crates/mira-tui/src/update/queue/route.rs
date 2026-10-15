//! Routage des intentions : hors vue seule l'ouverture agit ; vue ouverte, elle reçoit tout (modale).

use super::change::{cancel, resolve, retry};
use super::intent::QueueIntent;
use super::open::{close_queue, open_queue};
use crate::commands::Command;
use crate::state::{AppState, Selection};
use crate::update::{Intent, panes};

/// Intention de file reçue hors vue : seule l'ouverture agit.
pub fn route_idle(state: &mut AppState, intent: QueueIntent) -> Vec<Command> {
    match intent {
        QueueIntent::Open => open_queue(state),
        QueueIntent::Retry
        | QueueIntent::Cancel
        | QueueIntent::Resolve
        | QueueIntent::Resend
        | QueueIntent::DontResend => Vec::new(),
    }
}

/// Vue ouverte : déplacement, actions sur la fiche, retour et sortie ; les autres intentions sont ignorées.
#[rustfmt::skip]
pub fn route_open(state: &mut AppState, intent: Intent) -> Vec<Command> {
    match intent {
        Intent::MoveDown => step(state, Selection::down),
        Intent::MoveUp => step(state, Selection::up),
        Intent::Top => step(state, Selection::top),
        Intent::Bottom => step(state, Selection::bottom),
        Intent::Open => resolve(state),
        Intent::Back | Intent::Cancel => close_queue(state),
        Intent::Quit => panes::quit(state),
        Intent::Queue(queue) => on_queue(state, queue),
        Intent::NextPane | Intent::PrevPane | Intent::NextAccount | Intent::PrevAccount | Intent::Archive
        | Intent::Trash | Intent::DeletePermanently | Intent::ToggleFlag | Intent::ToggleRead
        | Intent::OpenMovePicker | Intent::Undo | Intent::Confirm | Intent::Search(_) | Intent::Compose(_)
        | Intent::Drafts(_) | Intent::StartSync | Intent::Help
        | Intent::Palette(_)
        | Intent::Settings(_) => Vec::new(),
    }
}

/// Applique une intention de file à la fiche sélectionnée.
fn on_queue(state: &mut AppState, intent: QueueIntent) -> Vec<Command> {
    match intent {
        QueueIntent::Retry => retry(state),
        QueueIntent::Cancel => cancel(state),
        QueueIntent::Resolve => resolve(state),
        QueueIntent::Open | QueueIntent::Resend | QueueIntent::DontResend => Vec::new(),
    }
}

/// Déplace la sélection de la vue.
fn step(state: &mut AppState, move_by: fn(&mut Selection, usize)) -> Vec<Command> {
    state.queue.move_selection(move_by);
    Vec::new()
}
