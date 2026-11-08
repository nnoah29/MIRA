//! Trousseau factice qui refuse les valeurs trop longues, comme celui de Windows (2560 octets UTF-16).

use mira_core::ports::{MemorySecretStore, SecretStore, SecretValue};
use mira_vocabulary::error::codes::secrets;
use mira_vocabulary::{ErrorCategory, MiraError, SecretLocation};

/// Emplacement nommé directement, pour regarder ce que le trousseau contient vraiment.
pub struct Raw(pub String);

impl SecretLocation for Raw {
    /// La clé telle quelle
    fn storage_key(&self) -> String {
        self.0.clone()
    }
}

/// Trousseau en mémoire qui refuse un secret de plus de `max_chars` caractères.
#[derive(Debug)]
pub struct LimitedStore {
    pub inner: MemorySecretStore,
    pub max_chars: usize,
}

impl SecretStore for LimitedStore {
    /// Refuse les valeurs trop longues
    fn put(&self, key: &dyn SecretLocation, value: &SecretValue) -> Result<(), MiraError> {
        if value.expose().chars().count() > self.max_chars {
            return Err(MiraError::new(
                ErrorCategory::Secrets,
                secrets::KEYRING_UNAVAILABLE,
                "trousseau indisponible",
            ));
        }
        self.inner.put(key, value)
    }

    /// Relit tel quel
    fn get(&self, key: &dyn SecretLocation) -> Result<Option<SecretValue>, MiraError> {
        self.inner.get(key)
    }

    /// Supprime tel quel
    fn delete(&self, key: &dyn SecretLocation) -> Result<(), MiraError> {
        self.inner.delete(key)
    }
}
