//! Requêtes du domaine des boîtes en lecture (12.4) ; les écritures passent par la file d'actions.

use mira_vocabulary::{AccountId, MailboxId};
use serde::{Deserialize, Serialize};

/// Lecture des boîtes d'un compte et choix des boîtes synchronisées.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum MailboxRequest {
    ListMailboxes {
        account: AccountId,
    },
    SetSyncedMailboxes {
        account: AccountId,
        mailboxes: Vec<MailboxId>,
    },
}
