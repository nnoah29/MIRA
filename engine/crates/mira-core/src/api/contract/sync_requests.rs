//! Requêtes du domaine « synchronisation » : lancer, mettre en pause, état, remonter la plage, resynchroniser (12.4).

use mira_vocabulary::{AccountId, MailboxId};
use serde::{Deserialize, Serialize};

/// Ce qu'une demande de synchronisation vise.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SyncScope {
    All,
    Account { account: AccountId },
    Mailbox { mailbox: MailboxId },
}

/// Opérations de synchronisation ; les trois tâches longues rendent une opération annulable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum SyncRequest {
    StartSync { scope: SyncScope },
    PauseSync { account: AccountId },
    GetSyncStatus,
    ExtendWindow { mailbox: MailboxId, days: u32 },
    ResyncMailbox { mailbox: MailboxId },
}
