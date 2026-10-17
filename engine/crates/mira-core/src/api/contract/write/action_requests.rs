//! Requêtes du domaine « actions » : actions sur les messages, annulation, écriture sur les boîtes (12.4).

use mira_vocabulary::{AccountId, ActionId, ActionKind, IdempotencyKey, MailboxId};
use serde::{Deserialize, Serialize};

use super::target_selection::TargetSelection;

/// Commandes d'intention : l'`ActionId` rendu est immédiat, le serveur suit plus tard (invariant 28).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum ActionRequest {
    ApplyAction {
        kind: ActionKind,
        targets: TargetSelection,
        idempotency_key: IdempotencyKey,
    },
    UndoAction {
        action: ActionId,
    },
    CreateMailbox {
        account: AccountId,
        name: String,
        parent: Option<MailboxId>,
        idempotency_key: IdempotencyKey,
    },
    RenameMailbox {
        mailbox: MailboxId,
        name: String,
        idempotency_key: IdempotencyKey,
    },
    DeleteMailbox {
        mailbox: MailboxId,
        idempotency_key: IdempotencyKey,
    },
    EmptyMailbox {
        mailbox: MailboxId,
        idempotency_key: IdempotencyKey,
    },
}
