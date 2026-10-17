//! Requêtes du domaine « rédaction » : brouillons, pièces jointes, envoi et boîte d'envoi (12.4).

use mira_vocabulary::{
    AccountId, ActionId, DraftId, IdempotencyKey, IdentityId, MessageId, Timestamp,
};
use serde::{Deserialize, Serialize};

use super::draft_changes::DraftChanges;

/// Façon de commencer un brouillon.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DraftMode {
    New,
    Reply { message: MessageId },
    ReplyAll { message: MessageId },
    Forward { message: MessageId },
}

/// Opérations de rédaction et d'envoi.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum ComposeRequest {
    CreateDraft {
        account: AccountId,
        identity: Option<IdentityId>,
        mode: DraftMode,
    },
    UpdateDraft {
        draft: DraftId,
        changes: DraftChanges,
    },
    AttachFile {
        draft: DraftId,
        path: String,
    },
    RemoveAttachment {
        draft: DraftId,
        position: u32,
    },
    DeleteDraft {
        draft: DraftId,
    },
    GetDraft {
        draft: DraftId,
    },
    ListDrafts {
        account: AccountId,
    },
    SendDraft {
        draft: DraftId,
        at: Option<Timestamp>,
        idempotency_key: IdempotencyKey,
    },
    CancelSend {
        action: ActionId,
    },
    ListOutbox {
        account: AccountId,
    },
    ResolveUncertainSend {
        action: ActionId,
        resend: bool,
    },
}
