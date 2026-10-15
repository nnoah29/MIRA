//! Vue Paramètres : barre d'onglets, page affichée et fenêtre ouverte.

pub mod accounts;
pub mod dialog;
pub mod events;
pub mod form_view;
pub mod identities;
pub mod maintenance;
pub mod rows;

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};

use crate::state::AppState;
use crate::state::settings::SettingsTab;
use crate::theme;

/// Barre des onglets, celui qui est affiché mis en valeur.
fn tab_bar(current: SettingsTab) -> Line<'static> {
    let mut spans = Vec::new();
    for (index, tab) in SettingsTab::all().into_iter().enumerate() {
        let label = format!(" {} {} ", index + 1, tab.title());
        let style = if tab == current { theme::selected() } else { theme::dim() };
        spans.push(Span::styled(label, style));
    }
    Line::from(spans)
}

/// Dessine la page affichée.
fn render_page(frame: &mut Frame, area: Rect, state: &AppState) {
    match state.settings.tab {
        SettingsTab::Accounts => accounts::render_accounts(frame, area, state),
        SettingsTab::Identities => identities::render_identities(frame, area, state),
        SettingsTab::Maintenance => maintenance::render_maintenance(frame, area, state),
        SettingsTab::Events => events::render_events(frame, area, state),
    }
}

/// Dessine la vue : onglets, page, puis la fenêtre ouverte par-dessus.
pub fn render_settings(frame: &mut Frame, area: Rect, state: &AppState) {
    let block = Block::bordered()
        .title("Paramètres")
        .border_style(theme::border(true));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let [tabs, page] = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).areas(inner);
    frame.render_widget(Paragraph::new(tab_bar(state.settings.tab)), tabs);
    render_page(frame, page, state);
    dialog::render_dialog(frame, area, state);
}
