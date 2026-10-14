//! Réponses du domaine « synchronisation » : état par compte.

use mira_vocabulary::{AccountId, AccountState, MailboxId, Timestamp};
use serde::{Deserialize, Serialize};

/// État de la synchronisation d'un compte (12.4) : connexion, dernier succès, travail en cours, file d'actions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountSyncStatus {
    pub account: AccountId,
    pub state: AccountState,
    pub syncing: bool,
    pub last_synced_at: Option<Timestamp>,
    pub last_error: Option<String>,
    pub mailboxes_in_progress: Vec<MailboxId>,
    pub pending_actions: u64,
}

/// Réponse à une requête de synchronisation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum SyncResponse {
    Status(Vec<AccountSyncStatus>),
    Paused(AccountSyncStatus),
}
