//! Requêtes des domaines comptes, identités et authentification (12.4, 7.5–7.9).

use mira_vocabulary::{AccountId, IdentityId, MailboxId, ProviderId, ServerSettings};
use serde::{Deserialize, Serialize};

use super::secret_input::SecretInput;

/// Opérations sur les comptes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum AccountRequest {
    PrepareAccount { address: String, provider: Option<ProviderId> },
    ListAccounts,
    UpdateAccount { account: AccountId, changes: AccountChanges },
    PauseAccount { account: AccountId },
    ResumeAccount { account: AccountId },
    DeleteAccount { account: AccountId },
}

/// Modifications d'un compte ; un champ absent n'est pas modifié.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AccountChanges {
    pub display_name: Option<String>,
    pub servers: Option<ServerSettings>,
    pub window_days: Option<u32>,
    pub synced_mailboxes: Option<Vec<MailboxId>>,
}

/// Opérations sur les identités d'envoi.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum IdentityRequest {
    ListIdentities { account: AccountId },
    AddIdentity { account: AccountId, address: String, display_name: String, signature: Option<String>, is_default: bool },
    UpdateIdentity { identity: IdentityId, display_name: Option<String>, signature: Option<String> },
    SetDefaultIdentity { identity: IdentityId },
}

/// Opérations d'authentification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum AuthRequest {
    StoreAppPassword { account: AccountId, password: SecretInput },
}
