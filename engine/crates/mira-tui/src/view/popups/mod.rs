//! Fenêtres modales dessinées au-dessus de l'écran principal.

pub mod close_draft;
pub mod confirm;
pub mod help;
pub mod move_target;
pub mod target_label;

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::Clear;

use crate::state::{AppState, Popup};

/// Rectangle centré de cette taille au plus, toujours contenu dans la zone.
pub fn centered(area: Rect, width: u16, height: u16) -> Rect {
    let width = width.min(area.width);
    let height = height.min(area.height);
    Rect {
        x: area.x + (area.width - width) / 2,
        y: area.y + (area.height - height) / 2,
        width,
        height,
    }
}

/// Dessine la fenêtre ouverte, s'il y en a une.
pub fn render_popup(frame: &mut Frame, state: &AppState) {
    match &state.popup {
        Popup::Closed => {}
        Popup::ConfirmDelete { messages } => {
            let area = centered(frame.area(), 64, 7);
            frame.render_widget(Clear, area);
            confirm::render_confirm(frame, area, messages);
        }
        Popup::MovePicker(picker) => {
            let rows = u16::try_from(picker.targets().len()).unwrap_or(u16::MAX);
            let area = centered(frame.area(), 64, rows.saturating_add(2));
            frame.render_widget(Clear, area);
            move_target::render_move_target(frame, area, picker);
        }
        Popup::ConfirmCloseDraft => {
            let area = centered(frame.area(), 64, 6);
            frame.render_widget(Clear, area);
            close_draft::render_close_draft(frame, area);
        }
        Popup::Help => {
            let rows = u16::try_from(help::help_lines(state).len()).unwrap_or(u16::MAX);
            let area = centered(frame.area(), 64, rows.saturating_add(2));
            frame.render_widget(Clear, area);
            help::render_help(frame, area, state);
        }
    }
}
