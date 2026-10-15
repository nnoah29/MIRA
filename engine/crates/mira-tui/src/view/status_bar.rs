//! Barre d'état : indicateurs, puis dernier message, dernière erreur de synchro ou touches du contexte.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use super::search_bar::render_search_bar;
use super::status_text::indicators;
use crate::input::hints::{Hint, hints_for};
use crate::input::settings_hints::settings_hints;
use crate::input::{KeyContext, context_of};
use crate::state::AppState;
use crate::theme;
use crate::view::safe_text::display_safe;

/// Affiche la saisie de recherche si elle est ouverte, sinon indicateurs et message.
pub fn render_status(frame: &mut Frame, area: Rect, state: &AppState) {
    if state.search.is_editing() {
        render_search_bar(frame, area, state);
        return;
    }
    let mut spans = Vec::new();
    for indicator in indicators(state) {
        spans.push(Span::styled(format!("{indicator} · "), theme::unread()));
    }
    spans.push(message_span(state));
    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

/// Message de statut, à défaut dernière erreur de synchro, à défaut l'aide courte.
fn message_span(state: &AppState) -> Span<'static> {
    match (&state.status.message, &state.status.sync_error) {
        (Some(message), _) => Span::styled(display_safe(message), theme::status()),
        (None, Some(error)) => Span::styled(display_safe(error), theme::status()),
        (None, None) => Span::styled(key_footer(state), theme::dim()),
    }
}

/// Touches valables dans le contexte courant, précédées du rappel de l'aide.
fn key_footer(state: &AppState) -> String {
    let context = context_of(state);
    let hints = match context {
        KeyContext::Settings => settings_hints(state.settings.tab),
        other => hints_for(other),
    };
    let mut parts = Vec::new();
    if !matches!(
        context,
        KeyContext::Compose
            | KeyContext::Creating
            | KeyContext::Queue
            | KeyContext::Drafts
            | KeyContext::Palette
            | KeyContext::Settings
            | KeyContext::SettingsForm
            | KeyContext::SettingsConfirm
    ) {
        parts.push("? aide".to_string());
    }
    parts.extend(
        hints
            .iter()
            .map(|Hint { keys, action }| format!("{keys} {action}")),
    );
    parts.join(" · ")
}
