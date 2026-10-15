//! Envoi du brouillon ouvert : phase en cours (enregistrement préalable, envoi en vol) et clé d'envoi unique.

use mira_assembly::mira_vocabulary::IdempotencyKey;

use super::field::Field;
use super::state::ComposeState;

/// Où en est l'envoi du brouillon : rien, enregistrement qui doit le précéder, ou requête d'envoi en vol.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SendPhase {
    #[default]
    Idle,
    Saving,
    Sending,
}

impl ComposeState {
    /// Phase d'envoi en cours.
    pub fn send_phase(&self) -> SendPhase {
        self.send_phase
    }

    /// Change la phase d'envoi.
    pub fn set_send_phase(&mut self, phase: SendPhase) {
        self.send_phase = phase;
    }

    /// Clé d'envoi du brouillon : celle déjà créée, sinon `fresh` devient la clé définitive du brouillon.
    pub fn adopt_send_key(&mut self, fresh: Option<IdempotencyKey>) -> Option<IdempotencyKey> {
        if self.send_key.is_none() {
            self.send_key = fresh;
        }
        self.send_key.clone()
    }

    /// Signale la saisie refusée de ce champ.
    pub fn flag(&mut self, field: Field) {
        self.fields[field.index()].flagged = true;
    }
}
