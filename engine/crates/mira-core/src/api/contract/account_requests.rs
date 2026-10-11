//! Requêtes du domaine comptes (12.4, 7.5–7.9) et leurs paramètres.

use mira_vocabulary::{
    AccountId, AuthMethodKind, IdempotencyKey, MailboxId, ProviderId, ServerSettings,
};
use serde::{Deserialize, Serialize};

use super::credential_input::CredentialInput;

/// Opérations sur les comptes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum AccountRequest {
    PrepareAccount {
        address: String,
        provider: Option<ProviderId>,
    },
    TestConnection {
        draft: AccountDraft,
        credential: CredentialInput,
    },
    AddAccount {
        draft: AccountDraft,
        credential: CredentialInput,
        idempotency_key: IdempotencyKey,
    },
    ListAccounts,
    UpdateAccount {
        account: AccountId,
        changes: AccountChanges,
    },
    PauseAccount {
        account: AccountId,
    },
    ResumeAccount {
        account: AccountId,
    },
    DeleteAccount {
        account: AccountId,
    },
}

/// Compte tel que l'utilisateur veut l'ajouter ; les serveurs ne sont saisis que pour « Autre ».
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountDraft {
    pub address: String,
    pub display_name: String,
    pub provider: Option<ProviderId>,
    pub servers: Option<ServerSettings>,
    pub auth: AuthMethodKind,
}

/// Modifications d'un compte ; un champ absent n'est pas modifié.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AccountChanges {
    pub display_name: Option<String>,
    pub servers: Option<ServerSettings>,
    pub window_days: Option<u32>,
    pub synced_mailboxes: Option<Vec<MailboxId>>,
}
