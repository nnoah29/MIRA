//! Référence de test et trousseau plafonné enveloppé par le découpage.

use mira_core::ports::{ChunkedSecretStore, MemorySecretStore};
use mira_vocabulary::{AccountId, ProfileId, SecretKind, SecretRef};

use super::limited::LimitedStore;

/// Référence de secret de test
pub fn reference() -> SecretRef {
    SecretRef {
        profile: ProfileId::from_raw("default"),
        account: AccountId::from_raw("acc-1"),
        kind: SecretKind::AccessToken,
    }
}

/// Clé de stockage de la référence de test.
pub fn key() -> String {
    mira_vocabulary::SecretLocation::storage_key(&reference())
}

/// Trousseau plafonné à 1000 caractères, enveloppé par le découpage en morceaux de 800.
pub fn chunked() -> ChunkedSecretStore<LimitedStore> {
    let limited = LimitedStore {
        inner: MemorySecretStore::new(),
        max_chars: 1000,
    };
    ChunkedSecretStore::new(limited, 800)
}
