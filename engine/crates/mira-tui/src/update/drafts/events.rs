//! Événements d'action et d'envoi du compte de la vue : un brouillon peut changer d'état, la vue est relue.

use mira_assembly::mira_vocabulary::{EventCategory, EventKind};

use super::open::refresh_drafts;
use crate::commands::Command;
use crate::state::AppState;

/// Relit la vue ouverte si l'événement touche la file ou les envois de son compte.
pub fn on_drafts_event(state: &mut AppState, kind: &EventKind) -> Vec<Command> {
    let concerns = state
        .drafts
        .account()
        .is_some_and(|account| kind.account() == Some(account));
    if concerns && changes_drafts(kind.category()) {
        return refresh_drafts(state);
    }
    Vec::new()
}

/// Vrai pour les familles d'événements qui peuvent changer l'état d'un brouillon.
fn changes_drafts(category: EventCategory) -> bool {
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
