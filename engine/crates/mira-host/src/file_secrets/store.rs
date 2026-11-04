//! Le trousseau fichier comme port de secrets : enregistrer, relire, supprimer sous verrou.

use std::collections::BTreeMap;

use mira_assembly::mira_vocabulary::{MiraError, SecretLocation};
use mira_assembly::ports::{SecretStore, SecretValue};

use super::FileSecretStore;
use super::sealed_file::{load, save, unavailable};

impl FileSecretStore {
    /// Applique une modification aux entrées sous le verrou, puis réécrit le fichier
    fn update(&self, change: impl FnOnce(&mut BTreeMap<String, String>)) -> Result<(), MiraError> {
        let _guard = self.lock.lock().map_err(|_| unavailable("trousseau"))?;
        let mut entries = load(&self.path, &self.key)?;
        change(&mut entries);
        save(&self.path, &self.key, &entries)
    }
}

impl SecretStore for FileSecretStore {
    /// Enregistre ou remplace le secret
    fn put(&self, key: &dyn SecretLocation, value: &SecretValue) -> Result<(), MiraError> {
        let value = value.expose().to_string();
        self.update(|entries| {
            entries.insert(key.storage_key(), value);
        })
    }

    /// Relit le secret, ou rien s'il n'existe pas
    fn get(&self, key: &dyn SecretLocation) -> Result<Option<SecretValue>, MiraError> {
        let _guard = self.lock.lock().map_err(|_| unavailable("trousseau"))?;
        Ok(load(&self.path, &self.key)?
            .remove(&key.storage_key())
            .map(SecretValue::new))
    }

    /// Supprime le secret ; absent, ce n'est pas une erreur
    fn delete(&self, key: &dyn SecretLocation) -> Result<(), MiraError> {
        self.update(|entries| {
            entries.remove(&key.storage_key());
        })
    }
}
