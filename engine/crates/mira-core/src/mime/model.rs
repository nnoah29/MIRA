//! Structures rendues par l'analyse MIME.

use mira_vocabulary::{AddressEntry, Timestamp, Unsubscribe};

/// Résumé des en-têtes utiles d'un message (identifiants sans chevrons).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HeaderSummary {
    pub from: Vec<AddressEntry>,
    pub to: Vec<AddressEntry>,
    pub cc: Vec<AddressEntry>,
    pub bcc: Vec<AddressEntry>,
    pub reply_to: Vec<AddressEntry>,
    pub subject: String,
    pub date: Option<Timestamp>,
    pub message_id: Option<String>,
    pub in_reply_to: Option<String>,
    pub references: Vec<String>,
}

/// Pièce jointe ou ressource incorporée décrite sans son contenu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedAttachment {
    pub part_index: u32,
    pub filename: Option<String>,
    pub mime_type: String,
    pub size: u64,
    pub content_id: Option<String>,
    pub inline: bool,
}

/// Message analysé : en-têtes, corps texte et HTML, pièces jointes, désabonnement.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ParsedMessage {
    pub headers: HeaderSummary,
    pub text_body: String,
    pub html_body: Option<String>,
    pub attachments: Vec<ParsedAttachment>,
    pub unsubscribe: Option<Unsubscribe>,
}
