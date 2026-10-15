//! Vue « boîte d'envoi et file d'actions » : une ligne par fiche, seules les lignes visibles sont construites.

pub mod actions;
pub mod outbox;

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, List, ListItem, ListState, Paragraph};

use super::message_list::window::visible_range;
use crate::state::AppState;
use crate::theme;

/// Dessine la vue : fiches, ou « chargement », ou « aucune action en file ».
pub fn render_queue(frame: &mut Frame, area: Rect, state: &AppState) {
    let count = state.queue.entries().len();
    let block = Block::bordered()
        .title(format!("File d'envoi et d'actions ({count})"))
        .border_style(theme::border(true));
    if count == 0 {
        let text = if state.queue.is_loaded() {
            "Aucune action en file"
        } else {
            "Chargement…"
        };
        frame.render_widget(Paragraph::new(text).block(block), area);
        return;
    }
    let (rows, selected) = queue_rows(state, area.width, area.height);
    let mut list_state = ListState::default().with_selected(selected);
    let list = List::new(rows)
        .block(block)
        .highlight_style(theme::selected());
    frame.render_stateful_widget(list, area, &mut list_state);
}

/// Lignes construites pour un cadre de cette taille (bordures comprises) et position de la sélection parmi elles.
pub fn queue_rows(
    state: &AppState,
    _width: u16,
    height: u16,
) -> (Vec<ListItem<'static>>, Option<usize>) {
    let capacity = usize::from(height.saturating_sub(2));
    let entries = state.queue.entries();
    let selected = state.queue.selection.selected;
    let range = visible_range(entries.len(), selected, capacity);
    let start = range.start;
    let rows = entries
        .get(range.clone())
        .unwrap_or(&[])
        .iter()
        .map(|entry| ListItem::new(outbox::entry_text(entry)))
        .collect();
    let shown = selected.filter(|index| range.contains(index));
    (rows, shown.map(|index| index - start))
}
