//! Trousseau de test dont l'écriture échoue.

use std::sync::Arc;

use mira_core::ports::{MemorySecretStore, SecretStore, SecretValue};
use mira_vocabulary::{MiraError, SecretLocation};

/// Trousseau dont l'écriture échoue : la lecture reste celle du trousseau mémoire.
#[derive(Debug)]
pub struct FailingPut(pub Arc<MemorySecretStore>);

impl SecretStore for FailingPut {
    /// Refuse toute écriture
    fn put(&self, _key: &dyn SecretLocation, _value: &SecretValue) -> Result<(), MiraError> {
        Err(MiraError::new(
            mira_vocabulary::ErrorCategory::Internal,
            mira_vocabulary::error::codes::internal::BUG,
            "écriture refusée",
        ))
    }

    /// Relit dans le trousseau mémoire
    fn get(&self, key: &dyn SecretLocation) -> Result<Option<SecretValue>, MiraError> {
        self.0.get(key)
    }

    /// Supprime dans le trousseau mémoire
    fn delete(&self, key: &dyn SecretLocation) -> Result<(), MiraError> {
        self.0.delete(key)
    }
}
