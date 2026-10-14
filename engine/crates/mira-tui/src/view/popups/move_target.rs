//! Sélecteur de la boîte de destination d'un déplacement.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, List, ListItem, ListState};

use crate::state::MovePicker;
use crate::theme;
use crate::view::safe_text::display_safe;

/// Liste des boîtes proposées, la boîte choisie en surbrillance.
pub fn render_move_target(frame: &mut Frame, area: Rect, picker: &MovePicker) {
    let items: Vec<ListItem> = picker
        .targets()
        .iter()
        .map(|mailbox| ListItem::new(display_safe(&mailbox.display_name)))
        .collect();
    let block = Block::bordered()
        .title("Déplacer vers")
        .border_style(theme::border(true));
    let list = List::new(items)
        .block(block)
        .highlight_style(theme::selected());
    let mut list_state = ListState::default().with_selected(Some(picker.selected()));
    frame.render_stateful_widget(list, area, &mut list_state);
}
