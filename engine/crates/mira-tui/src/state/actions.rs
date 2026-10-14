//! Mémoire des actions de l'utilisateur : celle à annuler et celle dont la réponse est attendue.

use mira_assembly::mira_vocabulary::{ActionId, IdempotencyKey};

/// Dernière action confirmée par le moteur et clé de la dernière action envoyée.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ActionsState {
    pub last_action: Option<ActionId>,
    awaited: Option<IdempotencyKey>,
}

impl ActionsState {
    /// Note que la réponse de l'action envoyée avec cette clé est attendue ; les plus anciennes sont périmées.
    pub fn await_reply(&mut self, key: IdempotencyKey) {
        self.awaited = Some(key);
    }

    /// Vrai si la réponse pour cette clé est celle de la dernière action envoyée.
    pub fn awaits(&self, key: &IdempotencyKey) -> bool {
        self.awaited.as_ref() == Some(key)
    }

    /// Mémorise l'action confirmée pour pouvoir l'annuler.
    pub fn confirm(&mut self, action: ActionId) {
        self.last_action = Some(action);
        self.awaited = None;
    }

    /// Oublie l'attente après un échec de la dernière action envoyée.
    pub fn forget_awaited(&mut self) {
        self.awaited = None;
    }
}
