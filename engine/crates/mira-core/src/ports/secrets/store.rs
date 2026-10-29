//! Port du trousseau : stocke et relit les secrets par référence.

use mira_vocabulary::{MiraError, SecretLocation};

use super::value::SecretValue;

/// Trousseau de secrets ; ne se replie jamais sur un stockage en clair (7.7).
pub trait SecretStore: Send + Sync + std::fmt::Debug {
    /// Enregistre ou remplace un secret
    fn put(&self, key: &dyn SecretLocation, value: &SecretValue) -> Result<(), MiraError>;

    /// Relit un secret, ou rien s'il n'existe pas
    fn get(&self, key: &dyn SecretLocation) -> Result<Option<SecretValue>, MiraError>;

    /// Supprime un secret ; un secret absent n'est pas une erreur
    fn delete(&self, key: &dyn SecretLocation) -> Result<(), MiraError>;
}
