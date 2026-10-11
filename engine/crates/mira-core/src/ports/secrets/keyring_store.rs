//! Trousseau du système (SecretService sous Linux) via la bibliothèque `keyring`.

use mira_vocabulary::error::codes::secrets;
use mira_vocabulary::{ErrorCategory, ErrorCode, MiraError, SecretRef};

use super::store::SecretStore;
use super::value::SecretValue;

const SERVICE_NAME: &str = "mira";

/// Trousseau réel ; n'écrit jamais un secret ailleurs que dans le trousseau système.
#[derive(Debug, Clone, Copy, Default)]
pub struct KeyringSecretStore;

impl KeyringSecretStore {
    /// Ouvre l'entrée du trousseau qui correspond à la référence
    fn entry(key: &SecretRef) -> Result<keyring::Entry, MiraError> {
        keyring::Entry::new(SERVICE_NAME, &key.storage_key()).map_err(|error| translate(&error))
    }
}

impl SecretStore for KeyringSecretStore {
    /// Enregistre le secret dans le trousseau système
    fn put(&self, key: &SecretRef, value: &SecretValue) -> Result<(), MiraError> {
        Self::entry(key)?
            .set_password(value.expose())
            .map_err(|error| translate(&error))
    }

    /// Relit le secret, ou rien si le trousseau n'a pas d'entrée
    fn get(&self, key: &SecretRef) -> Result<Option<SecretValue>, MiraError> {
        match Self::entry(key)?.get_password() {
            Ok(value) => Ok(Some(SecretValue::new(value))),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(error) => Err(translate(&error)),
        }
    }

    /// Supprime l'entrée du trousseau ; une entrée absente n'est pas une erreur
    fn delete(&self, key: &SecretRef) -> Result<(), MiraError> {
        match Self::entry(key)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(error) => Err(translate(&error)),
        }
    }
}

/// Convertit une erreur du trousseau en erreur du contrat, sans jamais citer de valeur
fn translate(error: &keyring::Error) -> MiraError {
    let (code, message): (ErrorCode, &str) = match error {
        keyring::Error::NoStorageAccess(_) => (secrets::KEYRING_LOCKED, "trousseau verrouillé"),
        _ => (secrets::KEYRING_UNAVAILABLE, "trousseau indisponible"),
    };
    MiraError::new(ErrorCategory::Secrets, code, message)
}
