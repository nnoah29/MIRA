//! Colonne du milieu : messages de la boîte sélectionnée, ou « aucun message ».

pub mod row;
pub mod short_date;
pub mod title;

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, List, ListState, Paragraph};

use crate::state::{AppState, Pane};
use crate::theme;

/// Dessine la liste de messages avec la ligne sélectionnée mise en valeur.
pub fn render_message_list(frame: &mut Frame, area: Rect, state: &AppState) {
    let focused = state.focus == Pane::List;
    let block = Block::bordered()
        .title(title::list_title(state))
        .border_style(theme::border(focused));
    if state.messages.is_empty() {
        frame.render_widget(Paragraph::new("aucun message").block(block), area);
        return;
    }
    let width = usize::from(area.width.saturating_sub(2));
    let rows = state
        .messages
        .items()
        .iter()
        .map(|m| row::message_item(m, width));
    let mut list_state = ListState::default().with_selected(state.messages.selection.selected);
    let list = List::new(rows).block(block).highlight_style(if focused {
        theme::selected()
    } else {
        theme::dim()
    });
    frame.render_stateful_widget(list, area, &mut list_state);
}
