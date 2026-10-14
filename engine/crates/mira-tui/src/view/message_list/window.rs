//! Fenêtre de la liste : seules les lignes visibles sont construites, la sélection toujours dedans (D-32).

use std::ops::Range;

use ratatui::widgets::ListItem;

use super::row::message_item;
use crate::state::AppState;

const ROW_HEIGHT: u16 = 2;

/// Indices affichables : au plus `capacity` éléments, la sélection en dernière ligne une fois la première page dépassée.
pub fn visible_range(len: usize, selected: Option<usize>, capacity: usize) -> Range<usize> {
    if len == 0 || capacity == 0 {
        return 0..0;
    }
    let selected = selected.unwrap_or(0).min(len - 1);
    let start = (selected + 1).saturating_sub(capacity);
    start..(start + capacity).min(len)
}

/// Lignes construites pour un cadre de cette taille (bordures comprises) et position de la sélection parmi elles.
pub fn list_rows(
    state: &AppState,
    width: u16,
    height: u16,
) -> (Vec<ListItem<'static>>, Option<usize>) {
    let inner_width = usize::from(width.saturating_sub(2));
    let capacity = usize::from(height.saturating_sub(2) / ROW_HEIGHT);
    let selected = state.messages.selection.selected;
    let range = visible_range(state.messages.len(), selected, capacity);
    let start = range.start;
    let visible = state.messages.items().get(range.clone()).unwrap_or(&[]);
    let rows = visible
        .iter()
        .map(|message| message_item(message, inner_width))
        .collect();
    let shown = selected.filter(|index| range.contains(index));
    (rows, shown.map(|index| index - start))
}
