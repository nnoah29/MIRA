//! Suite d'un échange après la réponse initiale d'une commande.

/// Dialogue à poursuivre après l'envoi de la réponse d'une commande.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Continuation {
    Authenticate { tag: String, mechanism: String },
    Idle { tag: String },
}
