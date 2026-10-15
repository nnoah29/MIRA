//! Page Événements : une page du journal, numéro, date, nature et compte de chaque événement.

use mira_assembly::mira_vocabulary::Event;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::widgets::Paragraph;

use super::rows::render_window;
use crate::state::AppState;
use crate::view::message_list::short_date::short_date;
use crate::view::safe_text::display_safe;

/// Nom de la nature d'un événement : le début de sa représentation de débogage, sans ses données.
fn kind_name(event: &Event) -> String {
    let debug = format!("{:?}", event.kind);
    debug.chars().take_while(|c| c.is_alphanumeric()).collect()
}

/// Ligne d'un événement : numéro, date courte, nature et compte concerné.
fn event_line(event: &Event) -> String {
    let account = event
        .kind
        .account()
        .map(|account| format!("  [{}]", display_safe(account.as_str())))
        .unwrap_or_default();
    format!("{:>6}  {}  {}{account}", event.seq.0, short_date(event.at), kind_name(event))
}

/// Dessine la page du journal, ou « chargement » / « journal vide ».
pub fn render_events(frame: &mut Frame, area: Rect, state: &AppState) {
    let events = &state.settings.events;
    let [head, body] = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).areas(area);
    let end = if events.at_end() { " (dernière page)" } else { "" };
    let title = format!("Page {} — {} événements{end}", events.page_number(), events.entries().len());
    frame.render_widget(Paragraph::new(title), head);
    if events.entries().is_empty() {
        let text = if events.is_loaded() { "Journal vide" } else { "Chargement…" };
        frame.render_widget(Paragraph::new(text), body);
        return;
    }
    let entries = events.entries();
    let line = |index: usize| entries.get(index).map(event_line).unwrap_or_default();
    render_window(frame, body, entries.len(), events.selection.selected, &line);
}
