//! Intentions de l'utilisateur, indépendantes des touches qui les produisent.

use super::{account_switch, navigation, panes};
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
}

/// Route une intention vers son gestionnaire.
pub fn on_intent(state: &mut AppState, intent: Intent) -> Vec<Command> {
    match intent {
        Intent::MoveDown => navigation::move_down(state),
        Intent::MoveUp => navigation::move_up(state),
        Intent::Top => navigation::top(state),
        Intent::Bottom => navigation::bottom(state),
        Intent::Open => panes::open(state),
        Intent::Back => panes::back(state),
        Intent::NextPane => panes::next_pane(state),
        Intent::PrevPane => panes::prev_pane(state),
        Intent::NextAccount => return account_switch::next_account(state),
        Intent::PrevAccount => return account_switch::prev_account(state),
        Intent::Quit => return panes::quit(state),
    }
    Vec::new()
}
