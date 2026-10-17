//! Compte de messagerie, son état et ses réglages de synchronisation.

use serde::{Deserialize, Serialize};

use super::auth::AuthMethodKind;
use super::servers::ServerSettings;
use crate::address::EmailAddress;
use crate::ids::keys::ProviderId;
use crate::ids::kinds::{AccountId, MailboxId, ProfileId};
use crate::time::Timestamp;

/// État de connexion d'un compte.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AccountState {
    Connected,
    Offline,
    AuthRequired,
    Paused,
    Error { code: String },
}

/// Réglages de synchronisation d'un compte (D-21 : fenêtre de 90 jours par défaut).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncSettings {
    pub window_days: u32,
    pub prefetch_bodies: u32,
    pub synced_mailboxes: Option<Vec<MailboxId>>,
}

impl Default for SyncSettings {
    /// Donne les réglages par défaut : 90 jours, 50 corps préchargés, toutes les boîtes.
    fn default() -> Self {
        Self {
            window_days: 90,
            prefetch_bodies: 50,
            synced_mailboxes: None,
        }
    }
}

/// Compte de messagerie d'un profil ; ne contient aucun secret.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Account {
    pub id: AccountId,
    pub profile: ProfileId,
    pub address: EmailAddress,
    pub display_name: String,
    pub provider: ProviderId,
    pub auth: AuthMethodKind,
    pub servers: Option<ServerSettings>,
    pub sync: SyncSettings,
    pub state: AccountState,
    pub created_at: Timestamp,
}
