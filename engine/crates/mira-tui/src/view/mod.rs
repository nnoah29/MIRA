//! Dessin de l'état à l'écran ; aucune modification de l'état.

pub mod ansi_skip;
pub mod layout;
pub mod message_list;
pub mod popups;
pub mod reader;
pub mod safe_text;
pub mod search_bar;
pub mod sidebar;
pub mod status_bar;
pub mod status_text;
pub mod welcome;

use ratatui::Frame;

use crate::state::AppState;

/// Dessine l'état complet dans le cadre donné.
pub fn render(frame: &mut Frame, state: &AppState) {
    let areas = layout::split(frame.area(), state.focus);
    if state.accounts.is_empty() {
        welcome::render_welcome(frame, layout::body(frame.area()));
    } else {
        sidebar::render_sidebar(frame, areas.sidebar, state);
        message_list::render_message_list(frame, areas.list, state);
        reader::render_reader(frame, areas.reader, state);
    }
    status_bar::render_status(frame, areas.status, state);
    popups::render_popup(frame, state);
}
