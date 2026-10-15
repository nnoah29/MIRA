//! Événements d'action et d'envoi : la vue ouverte est relue, l'avancement d'un envoi est annoncé.

use mira_assembly::mira_vocabulary::{EventCategory, EventKind};

use super::open::refresh_queue;
use crate::commands::Command;
use crate::state::AppState;
use crate::view::safe_text::display_safe;

/// Annonce l'avancement d'un envoi, puis relit la vue si l'événement concerne son compte.
pub fn on_queue_event(state: &mut AppState, kind: &EventKind) -> Vec<Command> {
    if let Some(message) = send_progress(kind) {
        state.status.message = Some(message);
    }
    let concerns = state
        .queue
        .account()
        .is_some_and(|account| kind.account() == Some(account));
    if concerns && is_queue_category(kind.category()) {
        return refresh_queue(state);
    }
    Vec::new()
}

/// Message pour un envoi parti, échoué ou incertain ; jamais de renvoi automatique d'un envoi incertain.
fn send_progress(kind: &EventKind) -> Option<String> {
    if let EventKind::SendDeparted { .. } = kind {
        return Some("Message envoyé".to_string());
    }
    if let EventKind::SendUncertain { .. } = kind {
        return Some("Envoi incertain : ouvrez la file (Q) pour décider".to_string());
    }
    if let EventKind::SendFailed { code, .. } = kind {
        return Some(format!(
            "Échec d'un envoi ({}) : voir la file (Q)",
            display_safe(code)
        ));
    }
    None
}

/// Vrai pour les familles d'événements qui changent la file d'actions.
fn is_queue_category(category: EventCategory) -> bool {
    match category {
        EventCategory::Action | EventCategory::Send => true,
        EventCategory::Engine
        | EventCategory::Account
        | EventCategory::Sync
        | EventCategory::Mailbox
        | EventCategory::Message
        | EventCategory::Thread
        | EventCategory::Alert
        | EventCategory::Log => false,
    }
}
