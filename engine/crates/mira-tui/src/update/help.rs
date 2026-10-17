//! Ouverture de l'aide des touches du contexte courant.

use crate::commands::Command;
use crate::state::{AppState, Popup};

/// Ouvre la fenêtre d'aide ; le contexte affiché est celui du panneau sous la fenêtre.
pub fn open_help(state: &mut AppState) -> Vec<Command> {
    state.popup = Popup::Help;
    Vec::new()
}
