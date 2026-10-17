//! Envoi (Ctrl+Entrée) : destinataires vérifiés, confirmation obligatoire, enregistrement des modifications d'abord.

use super::compose::save::save;
use super::send_dispatch::dispatch_send;
use crate::commands::Command;
use crate::state::compose::{Field, RecipientCheck, SendPhase};
use crate::state::{AppState, Popup};
use crate::view::safe_text::display_safe;

const SEGMENT_CHARS: usize = 40;

/// F5 : ouvre la confirmation si les destinataires sont valides (une fermeture ou sortie demandée avant est annulée).
pub fn request_send(state: &mut AppState) -> Vec<Command> {
    let Some(compose) = state.compose.open_mut() else {
        return Vec::new();
    };
    if compose.send_phase() != SendPhase::Idle {
        return busy(state);
    }
    match compose.check_recipients() {
        RecipientCheck::Ready(_) => {
            compose.set_close_after_save(false);
            compose.set_then_quit(false);
            state.popup = Popup::ConfirmSend;
        }
        RecipientCheck::Empty => {
            state.status.message = Some("Envoi : aucun destinataire".to_string());
        }
        RecipientCheck::Invalid(field, segment) => {
            compose.flag(field);
            state.status.message = Some(invalid_message(field, &segment));
        }
    }
    Vec::new()
}

/// Confirmation reçue : envoie, ou enregistre d'abord (l'envoi suivra la confirmation du moteur).
pub fn confirm_send(state: &mut AppState) -> Vec<Command> {
    let Some(compose) = state.compose.open_mut() else {
        return Vec::new();
    };
    if compose.send_phase() != SendPhase::Idle {
        return busy(state);
    }
    compose.set_close_after_save(false);
    compose.set_then_quit(false);
    if compose.is_saving() {
        compose.set_send_phase(SendPhase::Saving);
        return Vec::new();
    }
    if compose.has_unsaved() {
        return save_then_send(state);
    }
    dispatch_send(state)
}

/// Enregistre les modifications ; l'envoi n'est engagé que si l'enregistrement est vraiment parti.
pub fn save_then_send(state: &mut AppState) -> Vec<Command> {
    let commands = save(state);
    if let Some(compose) = state.compose.open_mut() {
        let phase = if commands.is_empty() {
            SendPhase::Idle
        } else {
            SendPhase::Saving
        };
        compose.set_send_phase(phase);
    }
    commands
}

/// Un envoi est déjà engagé : rien de plus, la barre d'état le dit.
pub fn busy(state: &mut AppState) -> Vec<Command> {
    state.status.message = Some("Envoi en cours : patientez".to_string());
    Vec::new()
}

/// Pourquoi rien ne part : première adresse invalide, neutralisée et raccourcie.
fn invalid_message(field: Field, segment: &str) -> String {
    let shown: String = segment.chars().take(SEGMENT_CHARS).collect();
    format!(
        "Envoi : adresse invalide dans {} : « {} »",
        field.label(),
        display_safe(&shown)
    )
}
