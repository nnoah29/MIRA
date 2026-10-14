//! Découpage de l'écran en trois colonnes et une barre d'état.

use ratatui::layout::{Constraint, Layout, Rect};

use crate::state::Pane;

const NARROW_WIDTH: u16 = 100;

/// Zones occupées par chaque panneau ; une zone vide n'est pas dessinée.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Areas {
    pub sidebar: Rect,
    pub list: Rect,
    pub reader: Rect,
    pub status: Rect,
}

/// Découpe l'écran : trois colonnes si large, sinon seul le panneau actif.
pub fn split(area: Rect, focus: Pane) -> Areas {
    let [body, status] = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(area);
    if area.width < NARROW_WIDTH {
        return single_pane(body, status, focus);
    }
    let [sidebar, list, reader] = Layout::horizontal([
        Constraint::Percentage(22),
        Constraint::Percentage(33),
        Constraint::Percentage(45),
    ])
    .areas(body);
    Areas {
        sidebar,
        list,
        reader,
        status,
    }
}

/// Donne tout le corps au panneau qui a le focus.
fn single_pane(body: Rect, status: Rect, focus: Pane) -> Areas {
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

/// Zone au-dessus de la barre d'état.
pub fn body(area: Rect) -> Rect {
    let [body, _] = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(area);
    body
}
