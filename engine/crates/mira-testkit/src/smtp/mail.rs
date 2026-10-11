//! Message accepté par le faux serveur SMTP.

/// Message reçu : expéditeur, destinataires et données dé-bourrées.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReceivedMail {
    pub from: String,
    pub recipients: Vec<String>,
    pub data: Vec<u8>,
}
