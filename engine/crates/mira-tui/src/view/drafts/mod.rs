//! Vue « Brouillons » : une ligne par brouillon, seules les lignes visibles sont construites.

pub mod row;

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, List, ListItem, ListState, Paragraph};

use super::message_list::window::visible_range;
use crate::state::AppState;
use crate::theme;

/// Dessine la vue : brouillons, ou « chargement », ou « aucun brouillon ».
pub fn render_drafts(frame: &mut Frame, area: Rect, state: &AppState) {
    let count = state.drafts.items().len();
    let block = Block::bordered()
        .title(format!("Brouillons ({count})"))
        .border_style(theme::border(true));
    if count == 0 {
        let text = if state.drafts.is_loaded() {
            "Aucun brouillon"
        } else {
            "Chargement…"
        };
        frame.render_widget(Paragraph::new(text).block(block), area);
        return;
    }
    let (rows, selected) = drafts_rows(state, area.width, area.height);
    let mut list_state = ListState::default().with_selected(selected);
    let list = List::new(rows)
        .block(block)
        .highlight_style(theme::selected());
    frame.render_stateful_widget(list, area, &mut list_state);
}

/// Lignes construites pour un cadre de cette taille (bordures comprises) et position de la sélection parmi elles.
pub fn drafts_rows(
    state: &AppState,
    _width: u16,
    height: u16,
) -> (Vec<ListItem<'static>>, Option<usize>) {
    let capacity = usize::from(height.saturating_sub(2));
    let items = state.drafts.items();
    let selected = state.drafts.selection.selected;
    let range = visible_range(items.len(), selected, capacity);
    let start = range.start;
    let rows = items
        .get(range.clone())
        .unwrap_or(&[])
        .iter()
        .map(|draft| ListItem::new(row::draft_text(draft)))
        .collect();
    let shown = selected.filter(|index| range.contains(index));
    (rows, shown.map(|index| index - start))
}
