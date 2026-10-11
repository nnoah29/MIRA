//! Message à remettre au serveur SMTP : enveloppe et octets MIME complets.

use std::fmt;

/// Enveloppe (expéditeur et destinataires, Bcc compris) et message MIME complet.
#[derive(Clone, PartialEq, Eq)]
pub struct OutgoingMessage {
    pub from: String,
    pub recipients: Vec<String>,
    pub raw: Vec<u8>,
}

impl fmt::Debug for OutgoingMessage {
    /// Affiche le nombre de destinataires et la taille, jamais le contenu
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OutgoingMessage")
            .field("recipients", &self.recipients.len())
            .field("raw_len", &self.raw.len())
            .finish()
    }
}
