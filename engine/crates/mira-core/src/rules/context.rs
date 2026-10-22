//! Contexte d'évaluation d'un message : résumé, adresses du compte et état du fil.

use mira_vocabulary::{EmailAddress, MessageSummary, Timestamp};

/// Pièce jointe connue du message, telle que vue par les conditions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttachmentInfo {
    pub name: String,
    pub media_type: String,
}

/// Tout ce que l'évaluateur sait du message ; fourni par l'appelant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageContext {
    pub summary: MessageSummary,
    pub own_addresses: Vec<EmailAddress>,
    pub attachments: Vec<AttachmentInfo>,
    pub in_existing_thread: bool,
    pub now: Timestamp,
}
