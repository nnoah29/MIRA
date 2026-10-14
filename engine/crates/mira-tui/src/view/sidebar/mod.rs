//! Colonne de gauche : comptes et boîtes du compte sélectionné.

pub mod accounts_tree;
pub mod role_label;

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, List, ListState};

use crate::state::{AppState, Pane};
use crate::theme;

/// Dessine l'arbre des comptes avec la ligne sélectionnée mise en valeur.
pub fn render_sidebar(frame: &mut Frame, area: Rect, state: &AppState) {
    let focused = state.focus == Pane::Sidebar;
    let block = Block::bordered()
        .title("Comptes")
        .border_style(theme::border(focused));
    let rows = accounts_tree::rows(state);
    let mut list_state = ListState::default();
    if focused {
        list_state.select(accounts_tree::selected_row(state));
    }
    let list = List::new(rows)
        .block(block)
        .highlight_style(theme::selected());
    frame.render_stateful_widget(list, area, &mut list_state);
}
