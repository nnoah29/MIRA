//! Liste fenêtrée d'une page de Paramètres : seules les lignes visibles sont construites.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{List, ListItem, ListState};

use crate::theme;
use crate::view::message_list::window::visible_range;

/// Dessine `len` lignes dans la zone ; `text` ne construit que celles de la fenêtre, la sélection toujours dedans.
pub fn render_window(
    frame: &mut Frame,
    area: Rect,
    len: usize,
    selected: Option<usize>,
    text: &dyn Fn(usize) -> String,
) {
    let range = visible_range(len, selected, usize::from(area.height));
    let start = range.start;
    let shown = selected.filter(|index| range.contains(index));
    let rows: Vec<ListItem<'static>> = range.map(|index| ListItem::new(text(index))).collect();
    let mut list_state = ListState::default().with_selected(shown.map(|index| index - start));
    let list = List::new(rows).highlight_style(theme::selected());
    frame.render_stateful_widget(list, area, &mut list_state);
}
