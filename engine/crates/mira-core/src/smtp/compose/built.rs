//! Message MIME prêt à envoyer, avec son enveloppe.

use std::fmt;

use crate::smtp::OutgoingMessage;

/// Résultat de la composition : octets MIME complets, identifiant et enveloppe SMTP.
#[derive(Clone, PartialEq, Eq)]
pub struct BuiltMessage {
    pub raw: Vec<u8>,
    pub message_id: String,
    pub envelope_from: String,
    pub envelope_recipients: Vec<String>,
}

impl BuiltMessage {
    /// Convertit en message prêt pour le client SMTP
    pub fn into_outgoing(self) -> OutgoingMessage {
        OutgoingMessage {
            from: self.envelope_from,
            recipients: self.envelope_recipients,
            raw: self.raw,
        }
    }
}

impl fmt::Debug for BuiltMessage {
    /// Affiche l'identifiant et la taille, jamais le contenu du message
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BuiltMessage")
            .field("message_id", &self.message_id)
            .field("raw_len", &self.raw.len())
            .field("recipients", &self.envelope_recipients.len())
            .finish()
    }
}
