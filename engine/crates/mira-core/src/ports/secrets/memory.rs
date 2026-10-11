//! Trousseau en mémoire, réservé aux tests.

use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard};

use mira_vocabulary::error::codes::secrets;
use mira_vocabulary::{ErrorCategory, MiraError, SecretRef};

use super::store::SecretStore;
use super::value::SecretValue;

/// Trousseau fictif ; `unavailable()` simule un trousseau inaccessible.
#[derive(Debug, Default)]
pub struct MemorySecretStore {
    pub(super) entries: Mutex<HashMap<String, SecretValue>>,
    broken: bool,
}

impl MemorySecretStore {
    /// Crée un trousseau vide et disponible
    pub fn new() -> Self {
        Self::default()
    }

    /// Crée un trousseau dont chaque opération échoue comme un trousseau indisponible
    pub fn unavailable() -> Self {
        Self {
            broken: true,
            ..Self::default()
        }
    }

    /// Ouvre l'accès aux secrets, ou échoue si le trousseau est indisponible
    pub(super) fn open(&self) -> Result<MutexGuard<'_, HashMap<String, SecretValue>>, MiraError> {
        if self.broken {
            return Err(MiraError::new(
                ErrorCategory::Secrets,
                secrets::KEYRING_UNAVAILABLE,
                "trousseau indisponible",
            ));
        }
        Ok(self
            .entries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()))
    }
}

impl SecretStore for MemorySecretStore {
    /// Enregistre ou remplace le secret en mémoire
    fn put(&self, key: &SecretRef, value: &SecretValue) -> Result<(), MiraError> {
        self.open()?.insert(key.storage_key(), value.clone());
        Ok(())
    }

    /// Relit le secret enregistré, s'il existe
    fn get(&self, key: &SecretRef) -> Result<Option<SecretValue>, MiraError> {
        Ok(self.open()?.get(&key.storage_key()).cloned())
    }

    /// Supprime le secret s'il existe
    fn delete(&self, key: &SecretRef) -> Result<(), MiraError> {
        self.open()?.remove(&key.storage_key());
        Ok(())
    }
}
