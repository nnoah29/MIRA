//! Trousseau de test dont la suppression peut échouer à la demande, le reste passant au trousseau mémoire.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use mira_core::ports::{MemorySecretStore, SecretStore, SecretValue};
use mira_vocabulary::error::codes::secrets;
use mira_vocabulary::{ErrorCategory, MiraError, SecretLocation};

/// Trousseau qui refuse `delete` tant que l'indicateur partagé est levé.
#[derive(Debug)]
pub struct FlakyStore {
    inner: Arc<MemorySecretStore>,
    fail_delete: Arc<AtomicBool>,
}

impl FlakyStore {
    /// Habille ce trousseau mémoire ; `fail_delete` à vrai fait échouer les suppressions
    pub fn new(inner: Arc<MemorySecretStore>, fail_delete: Arc<AtomicBool>) -> Self {
        Self { inner, fail_delete }
    }
}

impl SecretStore for FlakyStore {
    /// Enregistre dans le trousseau mémoire
    fn put(&self, key: &dyn SecretLocation, value: &SecretValue) -> Result<(), MiraError> {
        self.inner.put(key, value)
    }

    /// Relit depuis le trousseau mémoire
    fn get(&self, key: &dyn SecretLocation) -> Result<Option<SecretValue>, MiraError> {
        self.inner.get(key)
    }

    /// Échoue comme un trousseau verrouillé quand l'indicateur est levé, sinon supprime
    fn delete(&self, key: &dyn SecretLocation) -> Result<(), MiraError> {
        if self.fail_delete.load(Ordering::SeqCst) {
            return Err(MiraError::new(
                ErrorCategory::Secrets,
                secrets::KEYRING_UNAVAILABLE,
                "trousseau indisponible",
            ));
        }
        self.inner.delete(key)
    }
}
