//! Données d'entrée de la composition : identité d'envoi, destinataires, corps et pièces jointes.

use mira_vocabulary::{AddressEntry, Timestamp};

/// Pièce jointe à inclure dans le message : nom, type MIME et contenu binaire.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComposeAttachment {
    pub filename: String,
    pub mime_type: String,
    pub data: Vec<u8>,
}

/// Tout ce qu'il faut pour construire un message ; `message_id` est fourni par le moteur.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComposeInput {
    pub from: AddressEntry,
    pub to: Vec<AddressEntry>,
    pub cc: Vec<AddressEntry>,
    pub bcc: Vec<AddressEntry>,
    pub subject: String,
    pub date: Timestamp,
    pub message_id: String,
    pub in_reply_to: Option<String>,
    pub references: Vec<String>,
    pub body_text: String,
    pub body_html: Option<String>,
    pub attachments: Vec<ComposeAttachment>,
}
