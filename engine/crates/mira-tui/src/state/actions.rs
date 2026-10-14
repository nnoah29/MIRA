//! Mémoire des actions de l'utilisateur : celle à annuler et celle dont la réponse est attendue.

use mira_assembly::mira_vocabulary::{ActionId, IdempotencyKey};

/// Dernière action confirmée par le moteur et clé de la dernière action envoyée.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ActionsState {
    pub last_action: Option<ActionId>,
    awaited: Option<IdempotencyKey>,
}

impl ActionsState {
    /// Note l'envoi d'une action : plus rien d'ancien à annuler, et seule sa réponse compte.
    pub fn await_reply(&mut self, key: IdempotencyKey) {
        self.last_action = None;
        self.awaited = Some(key);
    }

    /// Vrai tant que la réponse de la dernière action envoyée n'est pas arrivée.
    pub fn in_flight(&self) -> bool {
        self.awaited.is_some()
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
