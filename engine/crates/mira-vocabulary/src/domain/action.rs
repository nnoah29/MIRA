//! Action utilisateur mise en file pour être rejouée vers le serveur.

use serde::{Deserialize, Serialize};

use crate::ids::keys::IdempotencyKey;
use crate::ids::kinds::{AccountId, ActionId, DraftId, MailboxId, MessageId};
use crate::time::Timestamp;

/// Nature d'une action en file, avec ses paramètres.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ActionKind {
    MarkRead {
        read: bool,
    },
    MarkFlagged {
        flagged: bool,
    },
    Move {
        to: MailboxId,
    },
    AddToMailbox {
        mailbox: MailboxId,
    },
    RemoveFromMailbox {
        mailbox: MailboxId,
    },
    Archive,
    Trash,
    DeletePermanently,
    MarkSpam {
        spam: bool,
    },
    CreateMailbox {
        name: String,
        parent: Option<MailboxId>,
    },
    RenameMailbox {
        mailbox: MailboxId,
        name: String,
    },
    DeleteMailbox {
        mailbox: MailboxId,
    },
    EmptyMailbox {
        mailbox: MailboxId,
    },
    SaveDraft {
        draft: DraftId,
    },
    Send {
        draft: DraftId,
    },
}

/// Avancement d'une action en file ; `Uncertain` désigne un envoi interrompu.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionState {
    Pending,
    InProgress,
    Succeeded,
    Failed,
    Cancelled,
    Uncertain,
}

/// Action en attente d'application, avec sa clé d'idempotence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PendingAction {
    pub id: ActionId,
    pub account: AccountId,
    pub kind: ActionKind,
    pub targets: Vec<MessageId>,
    pub idempotency_key: IdempotencyKey,
    pub state: ActionState,
    pub attempts: u32,
    pub last_error: Option<String>,
    pub created_at: Timestamp,
    pub not_before: Timestamp,
}
