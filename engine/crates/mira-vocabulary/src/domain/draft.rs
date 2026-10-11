//! Brouillon en cours de rédaction, programmé ou en envoi.

use serde::{Deserialize, Serialize};

use crate::address::entry::AddressEntry;
use crate::ids::kinds::{AccountId, DraftId, IdentityId, MessageId};
use crate::time::Timestamp;

/// État d'un brouillon.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DraftState {
    Editing,
    Scheduled { at: Timestamp },
    Sending,
}

/// Pièce jointe d'un brouillon (métadonnées seulement).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DraftAttachment {
    pub filename: String,
    pub mime_type: String,
    pub size: u64,
}

/// Message en préparation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Draft {
    pub id: DraftId,
    pub account: AccountId,
    pub identity: IdentityId,
    pub to: Vec<AddressEntry>,
    pub cc: Vec<AddressEntry>,
    pub bcc: Vec<AddressEntry>,
    pub subject: String,
    pub body_text: String,
    pub body_html: Option<String>,
    pub attachments: Vec<DraftAttachment>,
    pub in_reply_to: Option<MessageId>,
    pub state: DraftState,
    pub updated_at: Timestamp,
}
