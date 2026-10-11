//! Réponse construite par un gestionnaire de commande.

use super::continuation::Continuation;

/// Octets à envoyer au client et suite éventuelle de l'échange.
#[derive(Debug, Default)]
pub(crate) struct Reply {
    pub out: Vec<u8>,
    pub continuation: Option<Continuation>,
    pub close: bool,
}

impl Reply {
    /// Ajoute une ligne non étiquetée
    pub(crate) fn untagged(&mut self, text: &str) {
        self.out
            .extend_from_slice(format!("* {text}\r\n").as_bytes());
    }

    /// Ajoute des octets bruts à la réponse
    pub(crate) fn raw(&mut self, bytes: &[u8]) {
        self.out.extend_from_slice(bytes);
    }

    /// Termine la réponse par une ligne de statut étiquetée
    pub(crate) fn tagged(mut self, tag: &str, status: &str, text: &str) -> Self {
        self.out
            .extend_from_slice(format!("{tag} {status} {text}\r\n").as_bytes());
        self
    }

    /// Termine la réponse par un statut OK
    pub(crate) fn ok(self, tag: &str, text: &str) -> Self {
        self.tagged(tag, "OK", text)
    }

    /// Demande une suite d'échange après l'envoi de la réponse
    pub(crate) fn with_continuation(mut self, continuation: Continuation) -> Self {
        self.continuation = Some(continuation);
        self
    }

    /// Demande la fermeture de la connexion après l'envoi
    pub(crate) fn closing(mut self) -> Self {
        self.close = true;
        self
    }
}
