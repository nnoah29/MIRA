//! Affichage de diagnostic d'une demande et d'un flux de génération : des comptes, jamais de texte.

use std::fmt;

use super::{ChatStart, ChatStream};

impl fmt::Debug for ChatStart {
    /// Montre la connexion, le modèle et le nombre de messages, jamais leur texte ni la consigne.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ChatStart")
            .field("connection", &self.connection)
            .field("model", &self.request.model)
            .field("has_system", &self.request.system.is_some())
            .field("messages", &self.request.messages.len())
            .finish_non_exhaustive()
    }
}

impl fmt::Debug for ChatStream {
    /// Montre seulement l'identifiant de l'opération.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ChatStream")
            .field("operation", &self.operation)
            .finish_non_exhaustive()
    }
}
