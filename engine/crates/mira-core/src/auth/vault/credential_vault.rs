//! Définition du coffre : trousseau injecté, base du profil et clés de secrets.

use std::sync::Arc;

use mira_vocabulary::{AccountId, ProfileId, SecretKind, SecretRef};

use crate::ports::SecretStore;
use crate::storage::Database;

/// Coffre d'identifiants ; les secrets vivent dans le trousseau, seule l'expiration est en base.
#[derive(Debug, Clone)]
pub struct CredentialVault {
    pub(super) secrets: Arc<dyn SecretStore>,
    pub(super) database: Arc<Database>,
}

impl CredentialVault {
    /// Crée un coffre au-dessus d'un trousseau et de la base du profil
    pub fn new(secrets: Arc<dyn SecretStore>, database: Arc<Database>) -> Self {
        Self { secrets, database }
    }

    /// Référence de trousseau d'un secret de ce compte
    pub(super) fn key(profile: &ProfileId, account: &AccountId, kind: SecretKind) -> SecretRef {
        SecretRef {
            profile: profile.clone(),
            account: account.clone(),
            kind,
        }
    }
}
