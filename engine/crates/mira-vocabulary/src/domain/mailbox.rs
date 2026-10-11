//! Boîte (dossier ou libellé) d'un compte et son marqueur de synchronisation.

use serde::{Deserialize, Serialize};

use crate::ids::kinds::{AccountId, MailboxId};

/// Rôle d'une boîte.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MailboxRole {
    Inbox,
    Sent,
    Drafts,
    Trash,
    Junk,
    Archive,
    All,
    Flagged,
    Other,
}

impl MailboxRole {
    /// Dit si la boîte est une boîte système, qu'on ne renomme ni ne supprime (9.9).
    pub fn is_system(&self) -> bool {
        matches!(
            self,
            Self::Inbox | Self::Sent | Self::Drafts | Self::Trash | Self::Junk
        )
    }
}

/// Repère de synchronisation IMAP d'une boîte.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SyncMarker {
    pub uid_validity: u32,
    pub uid_next: u32,
    pub highest_modseq: Option<u64>,
}

/// Boîte d'un compte, avec ses compteurs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mailbox {
    pub id: MailboxId,
    pub account: AccountId,
    pub name: String,
    pub display_name: String,
    pub delimiter: Option<String>,
    pub parent: Option<MailboxId>,
    pub role: MailboxRole,
    pub sync_enabled: bool,
    pub marker: Option<SyncMarker>,
    pub total: u64,
    pub unread: u64,
}
