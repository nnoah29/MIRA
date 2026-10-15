//! Départ de la requête d'envoi, suite d'un enregistrement préalable, abandon avant le départ.

use mira_assembly::api::{ComposeRequest, Request};

use super::send::save_then_send;
use crate::commands::{Command, Tag};
use crate::state::AppState;
use crate::state::compose::SendPhase;

/// Envoie avec la clé du brouillon : créée au premier envoi, réutilisée à chaque renvoi (le moteur dédoublonne).
pub fn dispatch_send(state: &mut AppState) -> Vec<Command> {
    let needs_key = state.compose.open().is_some_and(|c| c.send_key.is_none());
    let fresh = needs_key.then(|| state.fresh_key("send"));
    let Some(compose) = state.compose.open_mut() else {
        return Vec::new();
    };
    let Some(key) = compose.adopt_send_key(fresh) else {
        return Vec::new();
    };
    compose.set_send_phase(SendPhase::Sending);
    let draft = compose.draft().clone();
    state.status.message = Some("Envoi…".to_string());
    vec![Command::Execute {
        tag: Tag::Sent {
            draft: draft.clone(),
            key: key.clone(),
        },
        request: Request::Compose(ComposeRequest::SendDraft {
            draft,
            at: None,
            idempotency_key: key,
        }),
    }]
}

/// Après un enregistrement réussi : envoie si l'envoi l'attendait, en enregistrant encore ce qui a changé entre-temps.
pub fn continue_send(state: &mut AppState) -> Vec<Command> {
    let Some(compose) = state.compose.open_mut() else {
        return Vec::new();
    };
    if compose.send_phase() != SendPhase::Saving {
        return Vec::new();
    }
    if !compose.has_unsaved() {
        return dispatch_send(state);
    }
    compose.set_send_phase(SendPhase::Idle);
    save_then_send(state)
}

/// Sortie demandée avant le départ de l'envoi (enregistrement en cours) : l'envoi est abandonné, rien ne partira.
pub fn abandon_pending_send(state: &mut AppState) {
    let Some(compose) = state.compose.open_mut() else {
        return;
    };
    if compose.send_phase() == SendPhase::Saving {
        compose.set_send_phase(SendPhase::Idle);
        state.status.message = Some("Envoi annulé".to_string());
    }
}
