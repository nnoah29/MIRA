//! Résumé d'un message : en-têtes, drapeaux et état du contenu.

use serde::{Deserialize, Serialize};

use crate::address::entry::AddressEntry;
use crate::ids::kinds::{AccountId, MailboxId, MessageId, ThreadId};
use crate::time::Timestamp;

/// Drapeaux portés par le message lui-même (D-14).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Flags {
    pub seen: bool,
    pub flagged: bool,
    pub answered: bool,
    pub forwarded: bool,
}

/// Ce qui est déjà disponible du contenu du message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentState {
    HeadersOnly,
    BodyPresent,
}

/// Moyens de désabonnement annoncés par l'expéditeur.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Unsubscribe {
    pub https_url: Option<String>,
    pub mailto: Option<String>,
    pub one_click: bool,
}

/// Résumé d'un message, tel que listé ; ne contient pas le corps.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageSummary {
    pub id: MessageId,
    pub account: AccountId,
    pub thread: Option<ThreadId>,
    pub from: Vec<AddressEntry>,
    pub to: Vec<AddressEntry>,
    pub cc: Vec<AddressEntry>,
    pub bcc: Vec<AddressEntry>,
    pub reply_to: Vec<AddressEntry>,
    pub subject: String,
    pub date: Option<Timestamp>,
    pub internal_date: Option<Timestamp>,
    pub message_id_header: Option<String>,
    pub in_reply_to: Option<String>,
    pub references: Vec<String>,
    pub flags: Flags,
    pub size: u64,
    pub preview: String,
    pub has_attachments: bool,
    pub unsubscribe: Option<Unsubscribe>,
    pub content: ContentState,
    pub mailboxes: Vec<MailboxId>,
}
