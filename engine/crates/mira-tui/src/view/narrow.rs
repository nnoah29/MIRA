//! Mode étroit : un seul panneau à la fois, celui qui a le focus ; Entrée descend, Échap remonte.

use ratatui::layout::Rect;

use super::layout::Areas;
use crate::state::Pane;

/// Donne tout le corps au panneau qui a le focus ; les autres ont une zone vide, donc ne sont pas dessinés.
pub fn single_pane(body: Rect, status: Rect, focus: Pane) -> Areas {
    let empty = Rect::default();
    let (sidebar, list, reader) = match focus {
        Pane::Sidebar => (body, empty, empty),
        Pane::List => (empty, body, empty),
        Pane::Reader => (empty, empty, body),
    };
    Areas {
        sidebar,
        list,
        reader,
        status,
    }
}
