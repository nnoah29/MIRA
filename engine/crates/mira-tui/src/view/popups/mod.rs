//! Fenêtres modales dessinées au-dessus de l'écran principal.

pub mod close_draft;
pub mod confirm;
pub mod confirm_send;
pub mod help;
pub mod move_target;
pub mod target_label;
pub mod uncertain;

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

/// Dessine la fenêtre ouverte, s'il y en a une, centrée sur un fond effacé.
pub fn render_popup(frame: &mut Frame, state: &AppState) {
    let Some((width, height)) = popup_size(state) else {
        return;
    };
    let area = centered(frame.area(), width, height);
    frame.render_widget(Clear, area);
    draw_popup(frame, area, state);
}

/// Taille voulue de la fenêtre ouverte (bordures comprises), ou rien si aucune n'est ouverte.
fn popup_size(state: &AppState) -> Option<(u16, u16)> {
    let rows = |count: usize| u16::try_from(count).unwrap_or(u16::MAX).saturating_add(2);
    match &state.popup {
        Popup::Closed => None,
        Popup::ConfirmDelete { .. } => Some((64, 7)),
        Popup::MovePicker(picker) => Some((64, rows(picker.targets().len()))),
        Popup::ConfirmCloseDraft => Some((64, 6)),
        Popup::ConfirmSend => Some((64, 10)),
        Popup::ResolveUncertain { .. } => Some((64, 8)),
        Popup::Help => Some((64, rows(help::help_lines(state).len()))),
    }
}

/// Dessine le contenu de la fenêtre ouverte dans sa zone.
fn draw_popup(frame: &mut Frame, area: Rect, state: &AppState) {
    match &state.popup {
        Popup::Closed => {}
        Popup::ConfirmDelete { messages } => confirm::render_confirm(frame, area, messages),
        Popup::MovePicker(picker) => move_target::render_move_target(frame, area, picker),
        Popup::ConfirmCloseDraft => close_draft::render_close_draft(frame, area),
        Popup::ConfirmSend => confirm_send::render_confirm_send(frame, area, state.compose.open()),
        Popup::ResolveUncertain { subject, .. } => {
            uncertain::render_uncertain(frame, area, subject)
        }
        Popup::Help => help::render_help(frame, area, state),
    }
}
