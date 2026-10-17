//! Annulation d'envoi demandée et pas encore confirmée par le moteur : l'objet est gardé pour l'annoncer au succès.

use mira_assembly::mira_vocabulary::ActionId;

use super::QueueState;

impl QueueState {
    /// Retient l'objet de l'envoi dont l'annulation vient d'être demandée.
    pub fn note_cancel(&mut self, action: ActionId, subject: String) {
        self.cancelling = Some((action, subject));
    }

    /// Rend l'objet si cette réponse concerne l'annulation demandée, et l'oublie.
    pub fn take_cancel(&mut self, action: &ActionId) -> Option<String> {
        match self.cancelling.take() {
            Some((pending, subject)) if &pending == action => Some(subject),
            other => {
                self.cancelling = other;
                None
            }
        }
    }
}
