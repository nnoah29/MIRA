//! Événements d'action et d'envoi : la vue ouverte est relue, l'avancement d'un envoi est annoncé.

use mira_assembly::mira_vocabulary::{ActionId, EventCategory, EventKind};

use super::open::refresh_queue;
use crate::commands::Command;
use crate::state::AppState;
use crate::state::queue::QueueItem;
use crate::view::safe_text::display_safe;

/// Annonce l'avancement d'un envoi (sans effacer l'échec d'envoi de l'éditeur ouvert), puis relit la vue si besoin.
pub fn on_queue_event(state: &mut AppState, kind: &EventKind) -> Vec<Command> {
    let failure_shown = state.compose.open().is_some_and(|c| c.send_failed());
    if let Some(message) = send_progress(state, kind).filter(|_| !failure_shown) {
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

/// Désignation d'un envoi : son objet s'il est connu (session ou vue « file »), neutralisé.
fn named(state: &AppState, action: &ActionId) -> String {
    let from_queue = state
        .queue
        .entries()
        .iter()
        .find_map(|entry| match &entry.item {
            QueueItem::Send { subject } if &entry.action == action => Some(subject.as_str()),
            QueueItem::Send { .. } | QueueItem::Action { .. } => None,
        });
    match state.sent_subjects.subject_of(action).or(from_queue) {
        Some(subject) => format!("Message « {} »", display_safe(subject)),
        None => "Un message de la file".to_string(),
    }
}

/// Message pour un envoi parti, échoué ou incertain ; jamais de renvoi automatique d'un envoi incertain.
fn send_progress(state: &AppState, kind: &EventKind) -> Option<String> {
    if let EventKind::SendDeparted { action, .. } = kind {
        return Some(format!("{} envoyé", named(state, action)));
    }
    if let EventKind::SendUncertain { action, .. } = kind {
        let name = named(state, action);
        return Some(format!(
            "{name} : envoi incertain, ouvrez la file (Q) pour décider"
        ));
    }
    if let EventKind::SendFailed { action, code, .. } = kind {
        let name = named(state, action);
        return Some(format!(
            "{name} : échec d'envoi ({}), voir la file (Q)",
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
